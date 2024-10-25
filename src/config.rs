use std::{
    ffi::OsStr,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
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
