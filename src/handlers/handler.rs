use std::path::Path;

pub trait Handler {
    fn handle_file(&mut self, path: &Path, data: &[u8], last_modified: u64);
    fn handle_dir(&mut self, path: &Path);
}
