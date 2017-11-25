
use errors::*;

pub trait CrateStorage {
    // Uploads the crate of the tar file and returns the URL that it is
    // available at.
    fn upload(&self, manifest: &::registry::CargoManifest, tar: &[u8]) -> Result<()>;
}

pub mod file;
pub mod artifactory;
