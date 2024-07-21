use std::{collections::HashSet, fs, path::Path};

use clap::Subcommand;
use color_print::{cprint, cprintln};
use ice::{
    api::{
        self,
        modrinth::{add_mod, HashMethod},
    },
    config::mod_config::ModConfig,
    loader::Loader,
    utils::fs::get_sha1_hash,
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
                let loaders = if let Loader::Quilt = config.loader {
                    vec![Loader::Quilt, Loader::Fabric]
                } else {
                    vec![config.loader]
                };

                let mut done_mods = HashSet::<String>::new();

                // Check every .jar file in the dir
                for file in fs::read_dir(current_dir).unwrap() {
                    let file = file.unwrap();
                    let path = file.path();
                    cprintln!(
                        "<b>Checking</b> {}...",
                        file.file_name().to_str().unwrap()
                    );
                    if path.extension().unwrap() == "jar" {
                        let hash = get_sha1_hash(&path).unwrap();
                        // If it is a modrinth mod
                        if let Ok(version) =
                            api::modrinth::get_version_from_hash(hash, HashMethod::Sha1)
                        {
                            let project = api::modrinth::get_project(version.project_id);
                            if let Some(version_number) = config.mods.get(&project.slug) {
                                // Update if version is incorrect
                                if version_number != &version.version_number
                                    || !loaders.iter().any(|l| version.loaders.contains(l))
                                    || !version.game_versions.contains(&config.version)
                                {
                                    api::modrinth::update_mod(
                                        &path,
                                        config.loader,
                                        &config.version,
                                    )
                                    .unwrap();
                                }
                                done_mods.insert(project.slug);
                            } else {
                                // Remove if not in mods.toml
                                cprintln!(
                                    "<r>Removing</r> {} = {}...",
                                    project.slug,
                                    version.version_number
                                );
                                fs::remove_file(&path).unwrap();
                            }
                        }
                    }
                }

                // Download other mods
                for (mod_name, version_number) in
                    config.mods.iter().filter(|(k, _)| !done_mods.contains(*k))
                {
                    api::modrinth::download_mod(
                        mod_name,
                        version_number,
                        config.loader,
                        &config.version,
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
                        if let Ok((slug, version)) =
                            api::modrinth::update_mod(path, config.loader, &config.version)
                        {
                            config.insert_mod(slug, version);
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
                            config.insert_mod(slug, version);
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
