mod modrinth;
mod server;

use clap::{Parser, Subcommand};
use ice::config::LocalModsConfig;
use ice_core::Loader;
use std::{
    env,
    path::{Path, PathBuf},
};
use tracing::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    working_dir: Option<PathBuf>,
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
    Check {
        #[arg(short, long)]
        version: Option<String>,
    },
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
    pub async fn exec<P1: AsRef<Path>, P2: AsRef<Path>>(self, current_dir: P1, config_path: P2) {
        let current_dir = current_dir.as_ref();
        let config_path = config_path.as_ref();

        if let ModCommands::Init { version, loader } = self {
            modrinth::init(version, loader, current_dir).await;
            return;
        }

        info!("loading mods.toml...");
        let mut config = LocalModsConfig::load(config_path).unwrap();

        match self {
            ModCommands::Sync => {
                modrinth::sync(current_dir, &config).await;
            }
            ModCommands::Update => {
                modrinth::update(current_dir, &mut config).await;
            }
            ModCommands::Add { slugs } => {
                modrinth::add(slugs, current_dir, &mut config).await;
            }
            ModCommands::Check { version } => {
                modrinth::check(version, current_dir, &config).await;
            }
            ModCommands::Init { .. } => (),
        }
    }
}

#[derive(Subcommand)]
pub enum ServerCommands {
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
    pub async fn exec<P: AsRef<Path>>(self, current_dir: P) {
        let current_dir = current_dir.as_ref();
        match self {
            ServerCommands::New {
                name,
                version,
                loader,
            } => {
                server::new(name, version, loader, current_dir).await;
            }
            ServerCommands::Init { version, loader } => {
                server::init(version, loader, current_dir).await;
            }
            ServerCommands::Install => {
                server::install(current_dir);
            }
            ServerCommands::Run => {
                server::run(current_dir).await;
            }
        }
    }
}

impl Cli {
    pub async fn exec(self) {
        let current_dir = self
            .working_dir
            .unwrap_or(env::current_dir().expect("failed to get current_dir"));

        match self.command {
            Commands::Modrinth(command) => {
                let config_path = current_dir.join("mods.toml");
                command.exec(current_dir, config_path).await;
            }
            Commands::Server(command) => command.exec(current_dir).await,
        }
    }
}
