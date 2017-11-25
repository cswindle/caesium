
use errors::*;
use modules::*;

use std;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub struct FileCrateStorage {
    pub location: PathBuf,
}

impl FileCrateStorage {
    pub fn new(location: &String) -> FileCrateStorage {
        FileCrateStorage {
            location: PathBuf::from(location),
        }
    }
}

/// File based storage
impl storage::CrateStorage for FileCrateStorage {
    fn upload(&self, manifest: &::registry::CargoManifest, tar: &[u8]) -> Result<()> {
        let upload_file = self.location.join(manifest.name.clone()).join(manifest.vers.clone()).join("download");

        std::fs::create_dir_all(upload_file.parent().unwrap()).expect("Failed to create dir");

        let mut uploaded_crate = File::create(upload_file.clone()).unwrap();
        uploaded_crate.write_all(tar).unwrap();

        Ok(())
    }
}
