use std;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serde_json;
use git2;
use git2::build::RepoBuilder;
use git2::{FetchOptions, Repository};

use errors::*;

#[derive(Debug, Deserialize)]
pub struct CargoManifest {
    pub name: String,
    pub vers: String,
    pub deps: Vec<CargoManifestDependency>,
    pub features: HashMap<String, Vec<String>>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub readme: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<String>,
    pub badges: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct CargoManifestDependency {
    pub optional: bool,
    pub default_features: bool,
    pub name: String,
    pub features: Vec<String>,
    pub version_req: String,
    pub target: Option<String>,
    pub kind: String,
    pub registry: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegistryIndexEntry {
    pub name: String,
    pub vers: String,
    pub deps: Vec<RegistryIndexDependency>,
    pub cksum: String,
    pub features: HashMap<String, Vec<String>>,
    pub yanked: bool,
}

#[derive(Debug, Serialize)]
pub struct RegistryIndexDependency {
    pub name: String,
    pub vers: String,
    pub registry: Option<String>,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: String,
}

impl RegistryIndexEntry {
    pub fn new(manifest: &CargoManifest, cksum: String) -> RegistryIndexEntry {
        RegistryIndexEntry {
            name: manifest.name.clone(),
            vers: manifest.vers.clone(),
            deps: manifest.deps.iter().map(|dep| RegistryIndexDependency::new(dep)).collect(),
            cksum: cksum,
            features: manifest.features.clone(),
            yanked: false,
        }
    }
}

impl RegistryIndexDependency {
    pub fn new(dep: &CargoManifestDependency) -> RegistryIndexDependency {
        RegistryIndexDependency {
            name: dep.name.clone(),
            vers: dep.version_req.clone(),
            registry: dep.registry.clone(),
            features: dep.features.clone(),
            optional: dep.optional,
            default_features: dep.default_features,
            target: dep.target.clone(),
            kind: dep.kind.clone(),
        }
    }
}

impl From<CargoManifestDependency> for RegistryIndexDependency {
    fn from(dep: CargoManifestDependency) -> Self {
        RegistryIndexDependency {
            name: dep.name.clone(),
            vers: dep.version_req.clone(),
            registry: dep.registry.clone(),
            features: dep.features.clone(),
            optional: dep.optional,
            default_features: dep.default_features,
            target: dep.target.clone(),
            kind: dep.kind.clone(),
        }
    }
}

pub struct Registry {
    index_repo: Repository,
}

impl Registry {
    pub fn new(registry_index: &str) -> Registry {
        let mut cb = git2::RemoteCallbacks::new();
        cb.credentials(|_url, username, _allowed| {
            git2::Cred::ssh_key_from_agent(username.unwrap())
        });

        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);

        let registry_path = Path::new("./repo");

        // Try and remove the repo directory before we clone
        let _ = std::fs::remove_dir_all(registry_path);

        let repo = match RepoBuilder::new().fetch_options(fo)
                                           .clone(registry_index, registry_path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };

        Registry {
            index_repo: repo,
        }
    }

    fn index_file(&self, name: &str) -> PathBuf {
        let base = self.index_repo.workdir().unwrap();

        let name = name.chars()
            .flat_map(|c| c.to_lowercase())
            .collect::<String>();
        match name.len() {
            1 => base.join("1").join(&name),
            2 => base.join("2").join(&name),
            3 => base.join("3").join(&name[..1]).join(&name),
            _ => base.join(&name[0..2]).join(&name[2..4]).join(&name),
        }
    }

    fn update_crate_index(&self, dst: &PathBuf, entry: &RegistryIndexEntry) -> Result<()> {
        std::fs::create_dir_all(dst.parent().unwrap())?;
        let mut prev = String::new();
        if std::fs::metadata(&dst).is_ok() {
            std::fs::File::open(&dst).and_then(|mut f| f.read_to_string(&mut prev))?;
        }
        let s = serde_json::to_string(&entry)?;
        let new = prev + &s;
        let mut f = std::fs::File::create(&dst)?;
        f.write_all(new.as_bytes())?;
        f.write_all(b"\n")?;

        Ok(())
    }

    fn commit(&self, index_file: &PathBuf, message: String) -> Result<()> {
        let mut index = self.index_repo.index()?;
        let mut repo_path = self.index_repo.workdir().unwrap().iter();
        let dst = index_file.iter()
            .skip_while(|s| Some(*s) == repo_path.next())
            .collect::<PathBuf>();
        index.add_path(&dst)?;
        index.write().unwrap();
        let tree_id = index.write_tree()?;
        let tree = self.index_repo.find_tree(tree_id)?;

        let head = self.index_repo.head()?;
        let parent = self.index_repo.find_commit(head.target().unwrap())?;
        let signature = self.index_repo.signature()?;

        self.index_repo.commit(Some("HEAD"), // point HEAD to our new commit
                               &signature,   // author
                               &signature,   // committer
                               &message,     // commit message
                               &tree,        // tree
                               &[&parent])?; // parents

        Ok(())
    }

    fn push(&self) -> Result<()> {
        let mut ref_status = None;
        let mut origin = self.index_repo.find_remote("origin")?;
        let res = {
            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(|_url, username, _allowed| {
                git2::Cred::ssh_key_from_agent(username.unwrap())
            });
            callbacks.push_update_reference(|refname, status| {
                assert_eq!(refname, "refs/heads/master");
                ref_status = status.map(|s| s.to_string());
                Ok(())
            });
            let mut opts = git2::PushOptions::new();
            opts.remote_callbacks(callbacks);
            origin.push(&["refs/heads/master"], Some(&mut opts))
        };
        match res {
            Ok(()) if ref_status.is_none() => Ok(()),
            Ok(()) => bail!("failed to push a ref: {:?}", ref_status),
            Err(e) => bail!("failure to push: {}", e),
        }
    }

    pub fn add_crate(&self, manifest: &CargoManifest, crate_tar: &[u8]) -> Result<()> {

        let mut sha = Sha256::new();
        sha.input(crate_tar);

        // Convert the manifest into the registry index
        let entry = RegistryIndexEntry::new(manifest, sha.result_str());

        let index_file = self.index_file(&entry.name);

        self.update_crate_index(&index_file, &entry)?;

        self.commit(&index_file, format!("Adding {} {}", manifest.name, manifest.vers))?;

        self.push()?;

        Ok(())
    }
}
