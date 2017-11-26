
use ::modules;

use std;
use std::io::Read;

use toml;

#[derive(Debug, Deserialize)]
pub struct CaesiumConfig {
    pub registry: CeasiumRegistryConfig,
    pub storage: CaesiumStorageConfig,
    pub authentication: Option<CaesiumAuthenticationConfig>,
    pub server: Option<CaesiumServerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct CeasiumRegistryConfig {
    pub index: String,
}

#[derive(Debug, Deserialize)]
pub struct CaesiumStorageConfig {
    pub file: Option<CaesiumFileStorageConfig>,
    pub artifactory: Option<CaesiumArtifactoryStorageConfig>,
}

#[derive(Debug, Deserialize)]
pub struct CaesiumFileStorageConfig {
    pub location: String,
}

#[derive(Debug, Deserialize)]
pub struct CaesiumArtifactoryStorageConfig {
    pub base_url: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct CaesiumAuthenticationConfig {
    pub oauth2: Option<CaesiumOAuth2Config>,
}

#[derive(Debug, Deserialize)]
pub struct CaesiumOAuth2Config {
    pub client_id: String,
    pub client_secret: String,
    pub authorization_url: String,
    pub token_url: String,
    pub scope: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CaesiumServerConfig {
    pub port: Option<u16>,
}

impl CaesiumConfig {
    pub fn new(config_file: &str) -> CaesiumConfig {
        let mut toml = String::new();
        std::fs::File::open(config_file).and_then(|mut f| f.read_to_string(&mut toml)).expect("Failed to read file");

        toml::from_str(&toml).unwrap()
    }

    pub fn create_storage_module(&self) -> Box<modules::storage::CrateStorage> {
        if let Some(ref file) = self.storage.file {
            Box::new(modules::storage::file::FileCrateStorage::new(&file.location))
        } else if let Some(ref artifactory) = self.storage.artifactory {
            Box::new(modules::storage::artifactory::ArtifactoryCrateStorage::new(&artifactory.base_url, &artifactory.api_key))
        } else {
            panic!("No storage config present");
        }
    }

    pub fn create_authentication_module(&self) -> Option<Box<modules::authentication::Authentication>> {
        if let Some(ref auth) = self.authentication {
            if let Some(ref oauth) = auth.oauth2 {
                Some(Box::new(modules::authentication::oauth2::OAuth2Authentication::new()))
            } else {
                None
            }
        } else {
            None
        }
    }
}
