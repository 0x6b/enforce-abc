use anyhow::{Context, Result};
use clap::Parser;
use cmd_lib::run_cmd;
use env_logger::{Builder, Env};
use log::info;

const BIN: &str = "enforce-abc";

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    command: SubCommand,
}

#[derive(Debug, Parser)]
enum SubCommand {
    /// Update enforce-abc with the latest version.
    Update,
}

fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    match Args::parse().command {
        SubCommand::Update => update(),
    }
}

fn update() -> Result<()> {
    let root = env!("CARGO_WORKSPACE_DIR");

    run_cmd!($BIN unregister).context("failed to unregister")?;
    info!("Unregistered current launch agent");

    run_cmd!(cargo install --path=$root).context("failed to install")?;
    info!("Installed new binary");

    run_cmd!($BIN register).context("failed to register")?;
    info!("Registered launch agent");

    info!("Update complete");
    Ok(())
}
