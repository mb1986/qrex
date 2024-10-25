use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use log::{debug, warn};
use serde::{Deserialize, Serialize};

use super::handler::Handler;

const METADATA_FILENAME: &str = "metadata.yaml";
const METADATA_FILENAME_TOML: &str = "metadata.toml";

#[derive(Serialize, Deserialize)]
pub struct MetadataRecord {
    last_mod: u64,
}

pub struct Metadata {
    base_path: PathBuf,
    path: PathBuf,
    records: BTreeMap<PathBuf, MetadataRecord>,
}

impl Metadata {
    pub fn new(base_path: &Path) -> Self {
        debug!("creating metadata handler with base path `{:?}`", base_path);

        let path = base_path.join(METADATA_FILENAME);

        Metadata {
            base_path: base_path.to_owned(),
            path,
            records: BTreeMap::new(),
        }
    }
}

impl Handler for Metadata {
    fn handle_before(&mut self) -> Result<()> {
        if !self.base_path.is_dir() {
            fs::create_dir_all(&self.base_path)?;
        }

        if self.path.is_file() {
            debug!("reading existing metadata file `{:?}`", &self.path);
            let file = fs::File::open(&self.path)?;
            let mut records: BTreeMap<PathBuf, MetadataRecord> = serde_yaml::from_reader(file)?;
            self.records.append(&mut records);
        }

        Ok(())
    }

    fn handle_file(&mut self, path: &Path, _data: &[u8], last_modified: u64) -> Result<()> {
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

        Ok(())
    }

    fn handle_dir(&mut self, _path: &Path) -> Result<()> {
        // do nothing
        Ok(())
    }

    fn handle_after(&mut self) -> Result<()> {
        debug!("writing metadata file `{:?}`", &self.path);
        let file = fs::File::create(&self.path)?;
        serde_yaml::to_writer(file, &self.records)?;
        fs::write(
            &self.base_path.join(METADATA_FILENAME_TOML),
            toml::to_string_pretty(&self.records)?,
        )?;
        Ok(())
    }
}
