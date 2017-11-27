
use errors::*;

#[derive(Debug, Deserialize)]
pub struct AuthenticationUserInfo {
    pub sub: String,
    pub name: Option<String>,
}

pub trait Authentication {
    // Authenticate using the token provided by cargo publish.
    fn authenticate(&self, token: &str) -> Result<AuthenticationUserInfo>;
}

pub mod openid;
