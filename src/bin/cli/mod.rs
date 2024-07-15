mod mod_commands;
mod server;

use clap::{Parser, Subcommand};
use std::env;

use mod_commands::ModCommands;
use server::ServerCommands;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Mod related commands
    Mod {
        #[command(subcommand)]
        command: ModCommands,
    },
    /// Server related commands
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    }
}

impl Cli {
    pub fn exec(self) {
        let current_dir = env::current_dir().expect("failed to get current_dir");

        match self.command {
            Commands::Mod { command } => command.exec(current_dir),
            Commands::Server { command } => command.exec(current_dir),
        }
    }
}
