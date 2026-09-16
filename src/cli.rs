use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    /// Find and prune large build/cache artifacts (dry-run by default)
    #[command(visible_alias = "c")]
    Clean {
        /// Directory to scan for project artifacts (defaults to current dir)
        path: Option<PathBuf>,
        /// Actually delete (default is a dry-run report)
        #[arg(short = 'e', long)]
        execute: bool,
        /// Only report matches at least this big, e.g. 1G, 500M
        #[arg(short = 'm', long = "min-size")]
        min_size: Option<String>,
        /// Also prune system trash and ~/.cache
        #[arg(short = 's', long)]
        system: bool,
        /// Extra directory names to treat as artifacts (repeatable)
        #[arg(short = 'n', long = "name")]
        name: Vec<String>,
    },
}
