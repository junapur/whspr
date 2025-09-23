mod cli;

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::Result;

use tracing_error::ErrorLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let fmt_layer = fmt::layer().compact().with_target(false);
    let filter_layer = cli.verbosity.tracing_level_filter();
    let error_layer = ErrorLayer::default();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .with(error_layer)
        .init();

    color_eyre::install()?;

    Ok(())
}
