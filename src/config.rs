use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub binary_path: PathBuf,
    pub output_path: PathBuf,
    #[serde(default)]
    pub base_address: u64,
    #[serde(rename = "resource")]
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
    let config_str = std::fs::read_to_string(path)?;
    let config = toml::from_str::<Config>(&config_str)?;
    Ok(config)
}
