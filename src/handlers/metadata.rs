use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use log::{debug, warn};
use serde::{Deserialize, Serialize};

use super::handler::Handler;

const METADATA_FILENAME: &str = "metadata.yaml";

#[derive(Serialize, Deserialize)]
pub struct MetadataRecord {
    last_mod: u64,
}

pub struct Metadata {
    path: PathBuf,
    records: HashMap<PathBuf, MetadataRecord>,
}

impl Metadata {
    pub fn new(base_path: &Path) -> Self {
        debug!("creating metadata handler with base path `{:?}`", base_path);

        if !base_path.is_dir() {
            fs::create_dir_all(&base_path).unwrap();
        }

        let path = base_path.join(METADATA_FILENAME);

        let records = if path.is_file() {
            let file = fs::File::open(&path).unwrap();
            serde_yaml::from_reader(file).unwrap()
        } else {
            HashMap::new()
        };

        Metadata { path, records }
    }
}

impl Handler for Metadata {
    fn handle_file(&mut self, path: &Path, _data: &[u8], last_modified: u64) {
        debug!("last_modified `{:?}`", last_modified);

        if self.records.contains_key(path) {
            warn!(
                "metadata already contains record for `{:?}`, overwriting",
                path
            );
        }

        self.records.insert(
            path.to_owned(),
            MetadataRecord {
                last_mod: last_modified,
            },
        );
    }

    fn handle_dir(&mut self, _path: &Path) {
        // do nothing
    }
}

impl Drop for Metadata {
    fn drop(&mut self) {
        let file = fs::File::create(&self.path).unwrap();
        serde_yaml::to_writer(file, &self.records).unwrap();
    }
}
