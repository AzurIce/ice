use std::path::Path;

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
                info!("loading config...");
                let ice_config = Config::load(current_dir.join("mods.toml")).unwrap();

                for (mod_name, version_number) in &ice_config.mods {
                    info!("downloading mod [{}]...", mod_name);
                    api::modrinth::download_mod(
                        mod_name,
                        version_number,
                        ice_config.loader,
                        current_dir.join("mods"),
                    );
                }
            }
        }
    }
}
