use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "deepwash",
    version = env!("CARGO_PKG_VERSION"),
    about = "Rust-based CLI cleaner"
)]

pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Remove Docker containers, whether running or stopped
    Docker{
        /// Remove volumes
        #[arg(short = 'v', long = "volumes")]
        volumes: bool,
    },
}
