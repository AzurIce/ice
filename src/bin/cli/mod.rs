mod modrinth;
mod server;

use clap::{Parser, Subcommand};
use ice_core::Loader;
use std::env;

use server::ServerCommands;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Modrinth related commands
    Modrinth {
        #[command(subcommand)]
        command: ModCommands,
    },
    /// Server related commands
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },
}

#[derive(Subcommand)]
enum ModCommands {
    /// Initialize a mods.toml
    Init {
        #[arg(short, long)]
        version: Option<String>,
        #[arg(short, long, default_value_t = Loader::Quilt, value_enum)]
        loader: Loader,
    },
    /// Sync mods
    Sync,
    /// Update mods
    Update,
    /// Add mod
    Add { slugs: Vec<String> },
}

impl Cli {
    pub fn exec(self) {
        let current_dir = env::current_dir().expect("failed to get current_dir");

        match self.command {
            Commands::Modrinth { command } => match command {
                ModCommands::Init { version, loader } => {
                    modrinth::init(version, loader, current_dir);
                }
                ModCommands::Sync => {
                    modrinth::sync(current_dir);
                }
                ModCommands::Update => {
                    modrinth::update(current_dir);
                }
                ModCommands::Add { slugs } => {
                    modrinth::add(slugs, current_dir);
                }
            },
            Commands::Server { command } => command.exec(current_dir),
        }
    }
}
