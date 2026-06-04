use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Copy, Subcommand)]
pub enum Command {
    /// Start the application. Default if no subcommand is provided.
    Start,

    /// Register the application to start on login.
    Register,

    /// Unregister the application from starting on login.
    Unregister,
}
