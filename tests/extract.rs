//! End-to-end test: build a tiny synthetic Qt resource binary in a temp dir,
//! invoke the `qrex` binary against it, and verify extracted output.

use std::fs;
use std::io::Write;
use std::path::Path;

use assert_cmd::Command;
use flate2::{Compression, write::ZlibEncoder};
use tempfile::tempdir;

const RECORD_SIZE: usize = 22;

#[derive(Default)]
struct Blob {
    tree: Vec<u8>,
    names: Vec<u8>,
    data: Vec<u8>,
}

impl Blob {
    fn record_count(&self) -> u32 {
        (self.tree.len() / RECORD_SIZE) as u32
    }

    fn add_dir(&mut self, name_offset: u32, child_count: u32, first_child_index: u32) {
        self.tree.extend_from_slice(&name_offset.to_be_bytes());
        self.tree.extend_from_slice(&2i16.to_be_bytes());
        self.tree.extend_from_slice(&child_count.to_be_bytes());
        self.tree
            .extend_from_slice(&first_child_index.to_be_bytes());
        self.tree.extend_from_slice(&0u64.to_be_bytes());
    }

    fn add_file(&mut self, name_offset: u32, flags: i16, data_offset: u32, last_modified: u64) {
        self.tree.extend_from_slice(&name_offset.to_be_bytes());
        self.tree.extend_from_slice(&flags.to_be_bytes());
        self.tree.extend_from_slice(&0i16.to_be_bytes());
        self.tree.extend_from_slice(&0i16.to_be_bytes());
        self.tree.extend_from_slice(&data_offset.to_be_bytes());
        self.tree.extend_from_slice(&last_modified.to_be_bytes());
    }

    fn add_name(&mut self, name: &str) -> u32 {
        let offset = self.names.len() as u32;
        let utf16: Vec<u16> = name.encode_utf16().collect();
        self.names
            .extend_from_slice(&(utf16.len() as u16).to_be_bytes());
        self.names.extend_from_slice(&[0u8; 4]);
        for cu in utf16 {
            self.names.extend_from_slice(&cu.to_be_bytes());
        }
        offset
    }

    fn add_data(&mut self, payload: &[u8]) -> u32 {
        let offset = self.data.len() as u32;
        self.data
            .extend_from_slice(&(payload.len() as u32).to_be_bytes());
        self.data.extend_from_slice(payload);
        offset
    }

    /// Returns (binary, tree_addr, names_addr, data_addr).
    fn build(self) -> (Vec<u8>, u64, u64, u64) {
        let names_addr = self.tree.len() as u64;
        let data_addr = names_addr + self.names.len() as u64;
        let mut buf = Vec::new();
        buf.extend(self.tree);
        buf.extend(self.names);
        buf.extend(self.data);
        (buf, 0, names_addr, data_addr)
    }
}

fn zlib_with_size_prefix(plain: &[u8]) -> Vec<u8> {
    let mut out = (plain.len() as u32).to_be_bytes().to_vec();
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(plain).unwrap();
    out.extend(encoder.finish().unwrap());
    out
}

/// Builds a fixture binary with this layout:
///
///   /
///   ├── plain.txt  (uncompressed "hello")
///   └── nested/
///       └── deep.bin (zlib-compressed binary payload)
fn build_fixture_binary() -> (Vec<u8>, u64, u64, u64) {
    let mut blob = Blob::default();

    let nested_name = blob.add_name("nested");
    let plain_name = blob.add_name("plain.txt");
    let deep_name = blob.add_name("deep.bin");

    let plain_data = blob.add_data(b"hello");
    let deep_payload: Vec<u8> = (0u8..32).collect();
    let deep_data = blob.add_data(&zlib_with_size_prefix(&deep_payload));

    // Layout (record indices):
    //   0: root  (children: indices 1..3)
    //   1: nested  (dir, children: index 3..4)
    //   2: plain.txt  (file)
    //   3: deep.bin  (file inside nested)
    blob.add_dir(0, 2, 1);
    blob.add_dir(nested_name, 1, 3);
    blob.add_file(plain_name, 0x00, plain_data, 1000);
    blob.add_file(deep_name, 0x01, deep_data, 2000);

    assert_eq!(blob.record_count(), 4);
    blob.build()
}

fn write_yaml_config(path: &Path, binary: &Path, output: &Path, addrs: (u64, u64, u64)) {
    let body = format!(
        "binary_path: {}\noutput_path: {}\nresources:\n  - version: 3\n    addresses: {{ tree: {}, names: {}, data: {} }}\n",
        binary.display(),
        output.display(),
        addrs.0,
        addrs.1,
        addrs.2,
    );
    fs::write(path, body).unwrap();
}

#[test]
fn extracts_files_from_synthetic_binary() {
    let dir = tempdir().unwrap();
    let binary_path = dir.path().join("fixture.bin");
    let output_path = dir.path().join("out");
    let config_path = dir.path().join("config.yaml");

    let (bytes, tree, names, data) = build_fixture_binary();
    fs::write(&binary_path, &bytes).unwrap();
    write_yaml_config(
        &config_path,
        &binary_path,
        &output_path,
        (tree, names, data),
    );

    Command::cargo_bin("qrex")
        .unwrap()
        .args(["--extract", "--skip-dirs"])
        .arg(&config_path)
        .assert()
        .success();

    let plain = fs::read(output_path.join("plain.txt")).unwrap();
    assert_eq!(plain, b"hello");

    let deep = fs::read(output_path.join("nested").join("deep.bin")).unwrap();
    assert_eq!(deep, (0u8..32).collect::<Vec<u8>>());
}

#[test]
fn dry_run_without_flags_creates_no_output() {
    let dir = tempdir().unwrap();
    let binary_path = dir.path().join("fixture.bin");
    let output_path = dir.path().join("out");
    let config_path = dir.path().join("config.yaml");

    let (bytes, tree, names, data) = build_fixture_binary();
    fs::write(&binary_path, &bytes).unwrap();
    write_yaml_config(
        &config_path,
        &binary_path,
        &output_path,
        (tree, names, data),
    );

    Command::cargo_bin("qrex")
        .unwrap()
        .arg(&config_path)
        .assert()
        .success();

    assert!(!output_path.exists(), "dry-run must not create output dir");
}

#[test]
fn metadata_flag_writes_metadata_files() {
    let dir = tempdir().unwrap();
    let binary_path = dir.path().join("fixture.bin");
    let output_path = dir.path().join("out");
    let config_path = dir.path().join("config.toml");

    let (bytes, tree, names, data) = build_fixture_binary();
    fs::write(&binary_path, &bytes).unwrap();
    fs::write(
        &config_path,
        format!(
            "binary_path = \"{}\"\noutput_path = \"{}\"\n\n[[resource]]\nversion = 3\naddresses = {{ tree = {}, names = {}, data = {} }}\n",
            binary_path.display(),
            output_path.display(),
            tree,
            names,
            data,
        ),
    )
    .unwrap();

    Command::cargo_bin("qrex")
        .unwrap()
        .arg("--metadata")
        .arg(&config_path)
        .assert()
        .success();

    assert!(output_path.join("metadata.yaml").is_file());
    assert!(output_path.join("metadata.toml").is_file());

    let toml_text = fs::read_to_string(output_path.join("metadata.toml")).unwrap();
    assert!(toml_text.contains("plain.txt"));
    assert!(toml_text.contains("last_mod = 1000"));
}

#[test]
fn base_address_above_resource_address_errors_cleanly() {
    let dir = tempdir().unwrap();
    let binary_path = dir.path().join("fixture.bin");
    let output_path = dir.path().join("out");
    let config_path = dir.path().join("config.yaml");

    let (bytes, _tree, _names, _data) = build_fixture_binary();
    fs::write(&binary_path, &bytes).unwrap();

    // base_address higher than the resource's tree/names/data addresses (which are 0/etc)
    // would have wrapped on subtraction in release mode and panicked in debug mode.
    fs::write(
        &config_path,
        format!(
            "binary_path: {}\noutput_path: {}\nbase_address: 0x10000\nresources:\n  - version: 3\n    addresses: {{ tree: 0x100, names: 0x200, data: 0x300 }}\n",
            binary_path.display(),
            output_path.display(),
        ),
    )
    .unwrap();

    let assert = Command::cargo_bin("qrex")
        .unwrap()
        .arg("--extract")
        .arg(&config_path)
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();
    assert!(
        stderr.contains("below base_address"),
        "expected base_address error, got stderr: {stderr}"
    );
}
