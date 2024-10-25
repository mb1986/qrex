use std::path::Path;

use anyhow::Result;

pub trait Handler {
    fn handle_file(&mut self, path: &Path, data: &[u8], last_modified: u64) -> Result<()>;
    fn handle_dir(&mut self, path: &Path) -> Result<()>;
    fn handle_before(&mut self) -> Result<()>;
    fn handle_after(&mut self) -> Result<()>;
}
