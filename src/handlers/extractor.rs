use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::Result;
use anyhow::anyhow;
use log::{debug, warn};

use super::handler::Handler;

/// Reject paths that could escape the extraction root: absolute paths, paths
/// containing parent (`..`) or root components, or Windows prefix components.
/// Resource names come from an untrusted binary, so a malformed/malicious
/// resource must not be able to write outside `base_path`.
fn ensure_safe_path(path: &Path) -> Result<()> {
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            _ => {
                return Err(anyhow!(
                    "refusing resource path with traversal or absolute component: {:?}",
                    path
                ));
            }
        }
    }
    Ok(())
}

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

        ensure_safe_path(path)?;

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
        ensure_safe_path(path)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_paths_are_accepted() {
        assert!(ensure_safe_path(Path::new("a.txt")).is_ok());
        assert!(ensure_safe_path(Path::new("dir/sub/file.bin")).is_ok());
        assert!(ensure_safe_path(Path::new("./already-normalized")).is_ok());
    }

    #[test]
    fn parent_components_are_rejected() {
        assert!(ensure_safe_path(Path::new("../escape")).is_err());
        assert!(ensure_safe_path(Path::new("ok/../escape")).is_err());
        assert!(ensure_safe_path(Path::new("..")).is_err());
    }

    #[test]
    fn absolute_paths_are_rejected() {
        assert!(ensure_safe_path(Path::new("/etc/passwd")).is_err());
    }
}
