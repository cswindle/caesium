#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate nom;
// extern crate router;
extern crate url;
#[macro_use]
extern crate hyper;
extern crate hyper_tls;
extern crate futures;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate crypto;
extern crate git2;
extern crate toml;
extern crate tokio_core;

mod config;
mod errors;
mod parser;
mod modules;
mod registry;

use errors::*;

use hyper::{Put, StatusCode};
use hyper::server::{Http, Service, Request, Response};

use futures::Stream;
use futures::Future;

use std::sync::Arc;


struct Caesium {
    registry: registry::Registry,

    // Config
    config: config::CaesiumConfig,

    authentication: Option<Box<modules::authentication::Authentication>>,

    // authorization: Option<modules::authorization::Authorisor>,

    storage: Box<modules::storage::CrateStorage>,
}

impl Caesium {
    pub fn new() -> Caesium {

        let config = config::CaesiumConfig::new("registry.toml");
        let storage = config.create_storage_module();
        let authentication = config.create_authentication_module();

        Caesium {
            registry: registry::Registry::new(&config.registry.index.clone()),
            config: config,
            storage: storage,
            authentication: authentication,
        }
    }

    fn publish(&self, manifest: &str, crate_tar: &[u8]) -> Result<()> {

        let manifest: registry::CargoManifest = serde_json::from_str(&manifest).unwrap();

        // Authenticate
        //  - OAuth
        //    - Github
        //    - Google
        //    - Gitlab
        if let Some(ref authentication) = self.authentication {
            authentication.authenticate("test")?;
        }

        // Authorize

        // Now call into the storage driver to store the crate
        self.storage.upload(&manifest, crate_tar)?;

        // Now that everything is stored, we need to update the index file so
        // that the crate is available.
        self.registry.add_crate(&manifest, crate_tar)?;

        Ok(())
    }
}

struct CaesiumService {
    caesium: Arc<Caesium>
}

impl CaesiumService {
    pub fn new(caesium: Arc<Caesium>) -> CaesiumService {
        CaesiumService {
            caesium: caesium,
        }
    }
}

impl Service for CaesiumService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<futures::Future<Item = Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        match (req.method(), req.path()) {
            (&Put, "/api/v1/crates/new") => {

                println!("Handling new");

                let mut caesium = self.caesium.clone();

                Box::new(req.body()
                    .fold(Vec::new(), |mut acc, chunk| {
                        acc.extend_from_slice(&*chunk);
                        futures::future::ok::<_, Self::Error>(acc)
                    })
                    .map(move |body| {
                        let (manifest, tar) = parser::parse_crate_upload(body.as_slice()).unwrap();

                        match caesium.publish(manifest, tar) {
                            Ok(_) => Response::new().with_status(StatusCode::Ok),
                            Err(_) => Response::new().with_status(StatusCode::InternalServerError),
                        }
                    }))
            },
            _ => {
                Box::new(futures::future::ok(Response::new().with_status(StatusCode::NotFound)))
            }
        }
    }
}

fn main() {
    let caesium = Arc::new(Caesium::new());

    let port = match caesium.config.server {
        Some(ref server) => server.port,
        None => None
    };

    let port = if let Some(port) = port {
        port
    } else {
        3000
    };

    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    let mut server = Http::new().bind(&addr, move || Ok(CaesiumService::new(caesium.clone()))).unwrap();
    server.no_proto();
    println!("Listening on http://{} with 1 thread.", server.local_addr().unwrap());
    server.run().unwrap();
}
