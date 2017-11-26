
use errors::*;

pub trait Authentication {
    // Authenticate using the token provided by cargo publish.
    fn authenticate(&self, token: &str) -> Result<()>;
}

pub mod oauth2;
