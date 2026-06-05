use anyhow::Result;
use clap::Parser;
use enforcer::run;
use env_logger::{Builder, Env};
use log::debug;

use crate::{
    args::{Args, Command},
    launch_agent::LaunchAgent,
};

mod args;
mod enforcer;
mod input_source;
mod launch_agent;

const LABEL: &str = "enforce-abc";

fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    debug!("{args:?}");

    match args.command.unwrap_or(Command::Start) {
        Command::Start => run(),
        Command::Register => LaunchAgent::new(LABEL)?.register(),
        Command::Unregister => LaunchAgent::new(LABEL)?.unregister(),
    }
}
