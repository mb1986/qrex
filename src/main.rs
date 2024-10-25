use std::fs::File;

use log::{debug, info, warn};

use crate::handlers::extractor::Extractor;
use crate::handlers::metadata::Metadata;
use crate::resource::runner::Runner;

pub mod args;
pub mod config;
pub mod error;
pub mod handlers;
pub mod resource;

fn main() -> error::Result<()> {
    let args = args::parse();

    env_logger::builder()
        .filter_level(if args.debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

    debug!("cmd line arguments parsed `{:?}`", &args);

    info!("reading configuration file `{:?}`", &args.config);
    let config = config::read_config(&args.config)?;

    debug!("configuration `{:?}`", &config);

    info!("processing binary `{:?}`", &config.binary_path);
    let mut binary = File::open(&config.binary_path)?;

    debug!("base address `{:#x}`", &config.base_address);

    if !args.extract && !args.metadata {
        warn!("there are no handlers attached, in dry run mode")
    }

    config.resources.iter().for_each(|res| {
        if res.version != 3 {
            warn!(
                "only version 3 resources are supported, `{:?}` provided, skipping",
                res.version
            );
            return;
        }

        let tree = res.addresses.tree - config.base_address;
        let names = res.addresses.names - config.base_address;
        let data = res.addresses.data - config.base_address;

        info!(
            "processing resource @ `{:#x}`, `{:#x}`, `{:#x}` (tree, names, data)",
            tree, names, data
        );
        let mut runner = Runner::new(tree, names, data);

        let base_path = if args.skip_dirs {
            &config.output_path
        } else {
            &config
                .output_path
                // FIXME: use below implementation instead of current one
                //.join(format!("{:#x}_{:#x}_{:#x}", tree, names, data))
                .join(format!(
                    "{:#x}_{:#x}_{:#x}",
                    res.addresses.tree, res.addresses.names, res.addresses.data
                ))
        };

        if args.extract {
            debug!("attaching extractor handler");
            let extractor = Extractor::new(&base_path);
            runner.attach_handler(Box::new(extractor));
        }

        if args.metadata {
            debug!("attaching metadata handler");
            let metadata = Metadata::new(&config.output_path);
            runner.attach_handler(Box::new(metadata));
        }

        debug!("starting the runner");
        runner.run(&mut binary);
        debug!("finishing the runner");
    });

    Ok(())
}
