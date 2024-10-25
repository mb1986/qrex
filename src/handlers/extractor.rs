use log::{debug, warn};
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use anyhow::Result;

use super::handler::Handler;

pub struct Extractor {
    base_path: PathBuf,
}

impl Extractor {
    pub fn new(base_path: &Path) -> Self {
        debug!(
            "creating extractor handler with base path `{:?}`",
            base_path
        );
        Extractor {
            base_path: base_path.to_owned(),
        }
    }
}

impl Handler for Extractor {
    fn handle_file(&mut self, path: &Path, data: &[u8], _last_modified: u64) -> Result<()> {
        debug!("creating file `{:?}`", path);

        let file_path = self.base_path.join(path);
        let dir_path = file_path.parent().ok_or(anyhow!("Wrong file path!"))?;

        if !dir_path.is_dir() {
            debug!("creating directory `{:?}` for file `{:?}`", dir_path, path);
            fs::create_dir_all(dir_path)?;
        }

        if file_path.is_file() {
            warn!("file `{:?} exists, overwriting`", path);
        }
        fs::write(file_path, data)?;
        Ok(())
    }

    fn handle_dir(&mut self, path: &Path) -> Result<()> {
        let dir_path = self.base_path.join(path);

        debug!("creating directory `{:?}`", path);

        fs::create_dir_all(dir_path)?;
        Ok(())
    }

    fn handle_before(&mut self) -> Result<()> {
        // do nothing
        Ok(())
    }

    fn handle_after(&mut self) -> Result<()> {
        // do nothing
        Ok(())
    }
}
