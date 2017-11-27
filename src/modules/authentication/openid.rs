
use errors::*;
use modules::*;

use tokio_core;
use hyper;
use hyper::Client;
use hyper_tls::HttpsConnector;
use hyper::header::{Authorization, Bearer};
use futures::Future;
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
    fn authenticate(&self, token: &str) -> Result<authentication::AuthenticationUserInfo> {

        // We need to try and get the userinfo_data here, if it fails then we
        // are not authenticated.
        let mut core = tokio_core::reactor::Core::new().unwrap();
        let handle = core.handle();

        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle).unwrap())
            .build(&handle);

        let mut request = hyper::Request::new(hyper::Method::Get, self.userinfo_endpoint.clone());
        request.headers_mut().set(Authorization(
            Bearer {
                token: token.to_string(),
           }
        ));

        let work = client.request(request)
            .and_then(|res| {
                if res.status() != hyper::StatusCode::Ok {
                    bail!(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Invalid status code: {}", res.status()),
                    ));
                }
                Ok(res)
            }).and_then(|res| {
                res.body().concat2().and_then(move |body| {
                    let json: authentication::AuthenticationUserInfo = serde_json::from_slice(&body).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            e
                        )
                    })?;
                    Ok(json)
                })
            });
        let user_info = core.run(work).map_err(|_| ErrorKind::AuthenticationError("Failed to authenticate".to_string()))?;

        Ok(user_info)
    }
}
