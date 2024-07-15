use std::{fs, path::Path};

use clap::Subcommand;
use ice::{
    api,
    config::{mod_config::ModConfig, Config},
    loader::Loader,
};
use log::info;

#[derive(Subcommand)]
pub(super) enum ModCommands {
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
}

impl ModCommands {
    pub(super) fn exec<P: AsRef<Path>>(self, current_dir: P) {
        let current_dir = current_dir.as_ref();
        match self {
            ModCommands::Init { version, loader } => {
                let path = current_dir.join("mods.toml");
                if path.exists() {
                    println!("mods.toml is already exists!");
                    return;
                }

                let config = ModConfig::new(
                    version.unwrap_or(api::mojang::get_latest_version().unwrap()),
                    loader,
                );
                config.save(current_dir.join("mods.toml")).unwrap();
            }
            ModCommands::Sync => {
                info!("loading mods.toml...");
                let ice_config = ModConfig::load(current_dir.join("mods.toml")).unwrap();

                info!("syncing mods...");
                for (mod_name, version_number) in &ice_config.mods {
                    info!("downloading mod [{}]...", mod_name);
                    api::modrinth::download_mod(
                        mod_name,
                        version_number,
                        ice_config.loader,
                        current_dir,
                    );
                }
            }
            ModCommands::Update => {
                info!("loading mods.toml...");
                let mut ice_config = ModConfig::load(current_dir.join("mods.toml")).unwrap();

                info!("updating mods...");
                for file in fs::read_dir(current_dir).unwrap() {
                    let file = file.unwrap();
                    let path = file.path();
                    if path.extension().unwrap() == "jar" {
                        if let Ok((slug, version_number)) =
                            api::modrinth::update_mod(path, ice_config.loader, &ice_config.version)
                        {
                            ice_config.mods.insert(slug, version_number);
                        }
                    }
                }

                info!("updating mods.toml...");
                ice_config.save(current_dir.join("mods.toml")).unwrap();
            }
        }
    }
}
