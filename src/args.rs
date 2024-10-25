use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    /// Path to the TOML configuration file
    pub config: PathBuf,

    /// Extract resources
    #[arg(short, long)]
    pub extract: bool,

    /// Skip creating separate directories for each resource
    #[arg(short, long, requires("extract"))]
    pub skip_dirs: bool,

    /// Save resources' metadata
    #[arg(short, long)]
    pub metadata: bool,

    /// Enable debug messages
    #[arg(short, long)]
    pub debug: bool,
}

pub fn parse() -> Arguments {
    Arguments::parse()
}
