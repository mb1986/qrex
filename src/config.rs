use std::{
    ffi::OsStr,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub binary_path: PathBuf,
    pub output_path: PathBuf,
    #[serde(default)]
    pub base_address: u64,
    #[serde(alias = "resource")]
    pub resources: Vec<ResourceConfig>,
}

#[derive(Deserialize, Debug)]
pub struct ResourceConfig {
    pub version: i32,
    pub addresses: AddressesConfig,
    pub calls: Option<Vec<u64>>,
}

#[derive(Deserialize, Debug)]
pub struct AddressesConfig {
    pub tree: u64,
    pub names: u64,
    pub data: u64,
}

pub fn read_config(path: &Path) -> Result<Config> {
    match &path.extension().map(OsStr::to_str) {
        Some(Some("toml")) => read_config_toml(path),
        Some(Some("yaml")) => read_config_yaml(path),
        _ => Err(anyhow!(
            "Config file must have a '.toml' or '.yaml' extension!"
        )),
    }
}

fn read_config_toml(path: &Path) -> Result<Config> {
    let config_str = std::fs::read_to_string(path)?;
    Ok(toml::from_str::<Config>(&config_str)?)
}

fn read_config_yaml(path: &Path) -> Result<Config> {
    let file = File::open(path)?;
    Ok(serde_yaml::from_reader(file)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_config(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn parses_toml_with_resource_alias() {
        let dir = tempdir().unwrap();
        let path = write_config(
            dir.path(),
            "qrex.toml",
            r#"
binary_path = "bin/app"
output_path = "out"
base_address = 0x1000

[[resource]]
version = 3
addresses = { tree = 0x100, names = 0x200, data = 0x300 }
calls = [0x10, 0x20]
"#,
        );

        let config = read_config(&path).unwrap();
        assert_eq!(config.binary_path, PathBuf::from("bin/app"));
        assert_eq!(config.output_path, PathBuf::from("out"));
        assert_eq!(config.base_address, 0x1000);
        assert_eq!(config.resources.len(), 1);

        let res = &config.resources[0];
        assert_eq!(res.version, 3);
        assert_eq!(res.addresses.tree, 0x100);
        assert_eq!(res.addresses.names, 0x200);
        assert_eq!(res.addresses.data, 0x300);
        assert_eq!(res.calls.as_deref(), Some([0x10u64, 0x20].as_slice()));
    }

    #[test]
    fn parses_yaml_with_resources_list() {
        let dir = tempdir().unwrap();
        let path = write_config(
            dir.path(),
            "qrex.yaml",
            r#"
binary_path: bin/app
output_path: out
resources:
  - version: 3
    addresses: { tree: 0x100, names: 0x200, data: 0x300 }
"#,
        );

        let config = read_config(&path).unwrap();
        assert_eq!(config.binary_path, PathBuf::from("bin/app"));
        assert_eq!(config.resources.len(), 1);
        assert!(config.resources[0].calls.is_none());
    }

    #[test]
    fn base_address_defaults_to_zero_when_omitted() {
        let dir = tempdir().unwrap();
        let path = write_config(
            dir.path(),
            "qrex.toml",
            r#"
binary_path = "bin/app"
output_path = "out"

[[resource]]
version = 3
addresses = { tree = 1, names = 2, data = 3 }
"#,
        );
        let config = read_config(&path).unwrap();
        assert_eq!(config.base_address, 0);
    }

    #[test]
    fn rejects_unknown_extension() {
        let dir = tempdir().unwrap();
        let path = write_config(dir.path(), "qrex.json", "{}");
        let err = read_config(&path).unwrap_err();
        assert!(err.to_string().contains("'.toml' or '.yaml'"));
    }

    #[test]
    fn rejects_extensionless_path() {
        let dir = tempdir().unwrap();
        let path = write_config(dir.path(), "qrex", "");
        assert!(read_config(&path).is_err());
    }
}
