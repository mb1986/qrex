use log::{debug, warn};
use std::{
    fs,
    path::{Path, PathBuf},
};

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
    fn handle_file(&mut self, path: &Path, data: &[u8], _last_modified: u64) {
        let file_path = self.base_path.join(path);
        let dir_path = file_path.parent().unwrap();

        debug!("creating file `{:?}`", path);

        if !dir_path.is_dir() {
            debug!(
                "creating directory `{:?}` for file `{:?}`",
                dir_path,
                file_path.file_name().unwrap()
            );
            fs::create_dir_all(dir_path).unwrap();
        }

        if file_path.is_file() {
            warn!("file `{:?} exists, overwriting`", path);
        }
        fs::write(file_path, data).unwrap();
    }

    fn handle_dir(&mut self, path: &Path) {
        let dir_path = self.base_path.join(path);

        debug!("creating directory `{:?}`", path);

        fs::create_dir_all(dir_path).unwrap();
    }
}
