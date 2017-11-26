
use errors::*;
use modules::*;

use tokio_core;
use hyper;
use hyper::Client;
use hyper_tls::HttpsConnector;
use futures::Future;
use futures::future;
use url;
use serde_json;
use serde_json::Value;

use futures::Stream;
use std::str::FromStr;
use std::io;

#[derive(Debug)]
pub struct OpenIdAuthentication {
    pub openid_configuration: hyper::Uri,

    // Information retrieved from openid configuration
    pub authorization_endpoint: hyper::Uri,
    pub token_endpoint: hyper::Uri,
    pub userinfo_endpoint: hyper::Uri,
}

impl OpenIdAuthentication {
    pub fn new(openid_configuration: &str) -> OpenIdAuthentication {

        // Get the endpoints that should be used with OAuth2
        let mut core = tokio_core::reactor::Core::new().unwrap();
        let handle = core.handle();

        let openid_config_url = hyper::Uri::from_str(openid_configuration).expect("Invalid OpenID URL");

        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle).expect("Failed to setup HTTPS"))
            .build(&handle);
        let work = client.get(openid_config_url.clone())
            .and_then(|res| {
                res.body().concat2().and_then(move |body| {
                    let json: Value = serde_json::from_slice(&body).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            e
                        )
                    })?;
                    Ok(json)
                })
            });
        let openid_config = core.run(work).expect("Failed to get openid configuration");

        let auth_endpoint = hyper::Uri::from_str(openid_config["authorization_endpoint"].as_str().unwrap()).expect("Invalid authorization_endpoint received");
        let token_endpoint = hyper::Uri::from_str(openid_config["token_endpoint"].as_str().unwrap()).expect("Invalid token_endpoint received");
        let userinfo_endpoint = hyper::Uri::from_str(openid_config["userinfo_endpoint"].as_str().unwrap()).expect("Invalid userinfo_endpoint received");

        OpenIdAuthentication {
            openid_configuration: openid_config_url,
            authorization_endpoint: auth_endpoint,
            token_endpoint: token_endpoint,
            userinfo_endpoint: userinfo_endpoint,
        }
    }
}

impl authentication::Authentication for OpenIdAuthentication {
    // Authenticate using the token provided by cargo publish.
    fn authenticate(&self, token: &str) -> Result<()> {
        Ok(())
    }
}
