
use errors::*;
use modules::*;

pub struct OpenIdAuthentication {
    pub openid_configuration: String,
}

impl OpenIdAuthentication {
    pub fn new(openid_configuration: &str) -> OpenIdAuthentication {
        OpenIdAuthentication {
            openid_configuration: openid_configuration.to_string(),
        }
    }
}

impl authentication::Authentication for OpenIdAuthentication {
    // Authenticate using the token provided by cargo publish.
    fn authenticate(&self, token: &str) -> Result<()> {
        Ok(())
    }
}
