mod cli;
mod tasks;
mod utils;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};
use tasks::docker;


fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Docker { images, volumes, full }) => docker::run(images, volumes, full),
        None => {
            Cli::command()
                .print_help()
                .expect("An error occurred while printing help");
        }
    }
}
