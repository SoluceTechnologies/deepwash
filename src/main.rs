mod cli;
mod tasks;
mod utils;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};
use tasks::clean::{self, CleanOpts, default_names};
use tasks::docker;
use utils::parse_size;


fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Docker { images, volumes, full }) => docker::run(images, volumes, full),
        Some(Commands::Clean { path, execute, min_size, system, name }) => {
            if path.is_none() && !system {
                Cli::command()
                    .print_help()
                    .expect("An error occurred while printing help");
                return;
            }
            let min_size = match min_size {
                Some(s) => match parse_size(&s) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        eprintln!("⚠️ Invalid --min-size: {}", e);
                        std::process::exit(1);
                    }
                },
                None => None,
            };
            let mut names = default_names();
            names.extend(name);
            clean::run(CleanOpts { path, execute, min_size, system, names });
        }
        None => {
            Cli::command()
                .print_help()
                .expect("An error occurred while printing help");
        }
    }
}
