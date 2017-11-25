
use errors::*;
use modules::*;

use std::str::FromStr;

use tokio_core;
use hyper;
use hyper::Client;
use futures::Future;
use url;

header! { (XJFrogArtApi, "X-JFrog-Art-Api") => [String] }

pub struct ArtifactoryCrateStorage {
    base_url: url::Url,
    api_key: String,
}

impl ArtifactoryCrateStorage {
    pub fn new(url: &String, api_key: &String) -> ArtifactoryCrateStorage {

        // May be worth ensuring that we can authenticate using the provided
        // credentials.

        ArtifactoryCrateStorage {
            base_url: url::Url::parse(url).expect("Invalid Artifactory URL in config"),
            api_key: api_key.clone(),
        }
    }
}

/// Artifactory based storage
impl storage::CrateStorage for ArtifactoryCrateStorage {
    fn upload(&self, manifest: &::registry::CargoManifest, tar: &[u8]) -> Result<()> {

        let mut core = tokio_core::reactor::Core::new().unwrap();
        let handle = core.handle();

        let client = Client::new(&handle);
        let body = tar.to_vec();

        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap()
                               .push(&manifest.name)
                               .push(&manifest.vers)
                               .push("download");
        let hyper_uri = hyper::Uri::from_str(url.as_str())?;

        let mut request = hyper::Request::new(hyper::Method::Put, hyper_uri);
        request.set_body(body);
        request.headers_mut().set(XJFrogArtApi(self.api_key.clone()));

        let work = client.request(request).and_then(|res| {
            if res.status() != hyper::StatusCode::Created {
                panic!("Received invalid error code: {}", res.status());
            }

            Ok(())
        });

        core.run(work)?;

        Ok(())
    }
}
