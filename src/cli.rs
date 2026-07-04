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
    /// Remove Docker containers (keeps images by default)
    Docker {
        /// Also remove images
        #[arg(short = 'i', long = "images")]
        images: bool,
        /// Also remove volumes
        #[arg(short = 'v', long = "volumes")]
        volumes: bool,
        /// Full clean: images + volumes + system prune -a + buildx cache (+ macOS Docker restart)
        #[arg(short = 'f', long = "full")]
        full: bool,
    },
}
