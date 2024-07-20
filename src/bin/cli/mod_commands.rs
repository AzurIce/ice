use std::{fs, path::Path};

use clap::Subcommand;
use color_print::{cprint, cprintln};
use ice::{
    api::{self, modrinth::add_mod},
    config::mod_config::ModConfig,
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
    /// Add mod
    Add { slugs: Vec<String> },
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
                let config = ModConfig::load(current_dir.join("mods.toml")).unwrap();

                info!("syncing mods...");
                for (mod_name, version_number) in &config.mods {
                    info!("downloading mod [{}]...", mod_name);
                    api::modrinth::download_mod(
                        mod_name,
                        version_number,
                        config.loader,
                        current_dir,
                    );
                }
            }
            ModCommands::Update => {
                info!("loading mods.toml...");
                let mut config = ModConfig::load(current_dir.join("mods.toml")).unwrap();

                info!("updating mods...");
                for file in fs::read_dir(current_dir).unwrap() {
                    let file = file.unwrap();
                    let path = file.path();
                    if path.extension().unwrap() == "jar" {
                        if let Ok((slug, version_number)) =
                            api::modrinth::update_mod(path, config.loader, &config.version)
                        {
                            config.mods.insert(slug, version_number);
                        }
                    }
                }

                cprintln!("updating mods.toml...");
                config.save(current_dir.join("mods.toml")).unwrap();
                cprintln!("done!")
            }
            ModCommands::Add { slugs } => {
                info!("loading mods.toml...");
                let mut config = ModConfig::load(current_dir.join("mods.toml")).unwrap();

                for slug in slugs {
                    cprint!("<g>Adding</> {slug}...");
                    if config.mods.contains_key(&slug) {
                        cprintln!("already exists, skipped.");
                        return;
                    }
                    cprintln!();
                    match add_mod(slug, config.loader, config.version.clone(), current_dir) {
                        Ok((slug, version)) => {
                            config.mods.insert(slug, version);
                            config.save(current_dir.join("mods.toml")).unwrap();
                        }
                        Err(err) => cprintln!("<r>err</>: {err}"),
                    }
                }
                cprintln!("done!")
            }
        }
    }
}
