#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use anyhow::Result;
use clap::Parser;
use env_logger::{Builder, Env};
use log::debug;

use crate::args::{Args, Command};
#[cfg(target_os = "macos")]
use crate::launch_agent::LaunchAgent;

mod args;
#[cfg(target_os = "macos")]
mod enforcer;
#[cfg(windows)]
mod enforcer_windows;
#[cfg(target_os = "macos")]
mod input_source;
#[cfg(target_os = "macos")]
mod launch_agent;

#[cfg(target_os = "macos")]
const LABEL: &str = "enforce-abc";

fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    debug!("{args:?}");

    match args.command.unwrap_or(Command::Start) {
        Command::Start => run(),
        #[cfg(target_os = "macos")]
        Command::Register => register(),
        #[cfg(target_os = "macos")]
        Command::Unregister => unregister(),
    }
}

#[cfg(target_os = "macos")]
fn run() -> Result<()> {
    enforcer::run()
}

#[cfg(windows)]
fn run() -> Result<()> {
    enforcer_windows::run()
}

#[cfg(target_os = "macos")]
fn register() -> Result<()> {
    LaunchAgent::new(LABEL)?.register()
}

#[cfg(target_os = "macos")]
fn unregister() -> Result<()> {
    LaunchAgent::new(LABEL)?.unregister()
}

#[cfg(not(any(target_os = "macos", windows)))]
compile_error!("enforce-abc supports only macOS and Windows");
