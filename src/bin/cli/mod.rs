mod modrinth;
mod server;

use clap::{Parser, Subcommand};
use ice::config::ModConfig;
use ice_core::Loader;
use std::{env, path::Path};
use tracing::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Modrinth related commands
    #[command(subcommand)]
    Modrinth(ModCommands),
    /// Server related commands
    #[command(subcommand)]
    Server(ServerCommands),
}

#[derive(Subcommand)]
pub enum ModCommands {
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

impl ModCommands {
    pub fn exec<P1: AsRef<Path>, P2: AsRef<Path>>(self, current_dir: P1, config_path: P2) {
        let current_dir = current_dir.as_ref();
        let config_path = config_path.as_ref();

        if let ModCommands::Init { version, loader } = self {
            modrinth::init(version, loader, current_dir);
            return;
        }

        info!("loading mods.toml...");
        let mut config = ModConfig::load(config_path).unwrap();

        match self {
            ModCommands::Sync => {
                modrinth::sync(current_dir, &mut config);
            }
            ModCommands::Update => {
                modrinth::update(current_dir, &mut config, config_path);
            }
            ModCommands::Add { slugs } => {
                modrinth::add(slugs, current_dir, &mut config, config_path);
            }
            _ => (),
        }
    }
}

#[derive(Subcommand)]
pub enum ServerCommands {
    /// mod command for servers Ice.toml
    #[command(subcommand)]
    Mod(ModCommands),
    New {
        name: String,

        #[arg(short, long)]
        version: Option<String>,

        #[arg(short, long, default_value_t = Loader::Quilt, value_enum)]
        loader: Loader,
    },
    Init {
        #[arg(short, long)]
        version: Option<String>,

        #[arg(short, long, default_value_t = Loader::Quilt, value_enum)]
        loader: Loader,
    },
    Install,
    Run,
}

impl ServerCommands {
    pub fn exec<P: AsRef<Path>>(self, current_dir: P) {
        let current_dir = current_dir.as_ref();
        match self {
            ServerCommands::Mod(command) => {
                let config_path = current_dir.join("Ice.toml");
                let current_dir = current_dir.join("server").join("mods");
                command.exec(current_dir, config_path);
            }
            ServerCommands::New {
                name,
                version,
                loader,
            } => {
                server::new(name, version, loader, current_dir);
            }
            ServerCommands::Init { version, loader } => {
                server::init(version, loader, current_dir);
            }
            ServerCommands::Install => {
                server::install(current_dir);
            }
            ServerCommands::Run => {
                server::run(current_dir);
            }
        }
    }
}

impl Cli {
    pub fn exec(self) {
        let current_dir = env::current_dir().expect("failed to get current_dir");

        match self.command {
            Commands::Modrinth(command) => {
                let config_path = current_dir.join("mods.toml");
                command.exec(current_dir, config_path);
            }
            Commands::Server(command) => command.exec(current_dir),
        }
    }
}
