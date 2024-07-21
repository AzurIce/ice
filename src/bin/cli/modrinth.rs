use std::{
    collections::HashSet,
    fs::{self, remove_file, DirEntry},
    path::Path,
};

use color_print::{cprint, cprintln};
use futures::StreamExt;
use ice::{
    api::{
        self,
        modrinth::{add_mod, types::Version, utils::download_version_file, HashMethod},
    },
    config::ModConfig,
};
use ice_core::Loader;
use ice_util::fs::get_sha1_hash;
use log::info;

/// Initialize a `mods.toml` under `current_dir`
///
/// if `version` is `None` then it will default to the latest release
#[tokio::main]
pub async fn init<P: AsRef<Path>, S: AsRef<str>>(
    version: Option<S>,
    loader: Loader,
    current_dir: P,
) {
    let version = version.map(|s| s.as_ref().to_string());
    let current_dir = current_dir.as_ref();

    let path = current_dir.join("mods.toml");
    if path.exists() {
        println!("mods.toml is already exists!");
        return;
    }

    let config = ModConfig::new(
        version.unwrap_or(api::mojang::get_latest_version().await.unwrap()),
        loader,
    );
    config.save(current_dir.join("mods.toml")).unwrap();
}

#[tokio::main]
pub async fn sync<P: AsRef<Path>>(current_dir: P) {
    let current_dir = current_dir.as_ref();

    info!("loading mods.toml...");
    let config = ModConfig::load(current_dir.join("mods.toml")).unwrap();

    let mut synced_mods = HashSet::<String>::new();

    let jar_files = get_jar_files(current_dir);
    let modrinth_files = tokio_stream::iter(jar_files.into_iter())
        .filter_map(|f| async {
            let hash = get_sha1_hash(&f.path()).unwrap();
            let version = api::modrinth::get_version_from_hash(&hash, HashMethod::Sha1).await;
            if version.is_err() {
                return None;
            }

            Some((f, hash, version.unwrap()))
        })
        .collect::<Vec<(DirEntry, String, Version)>>()
        .await;

    let loaders = if let Loader::Quilt = config.loader {
        vec![Loader::Quilt, Loader::Fabric]
    } else {
        vec![config.loader]
    };
    let game_version = config.version.clone();
    // Check every .jar file in the dir
    for (file, hash, cur_version) in modrinth_files {
        let path = file.path();

        cprintln!("<b>Checking</b> {}...", file.file_name().to_str().unwrap());
        // If it is a modrinth mod
        let project = api::modrinth::get_project(cur_version.project_id)
            .await
            .unwrap();
        match config.mods.get(&project.slug) {
            // Update if version is incorrect
            Some(version_number) => {
                if version_number != &cur_version.version_number
                    || !loaders.iter().any(|l| cur_version.loaders.contains(l))
                    || !cur_version.game_versions.contains(&config.version)
                {
                    let latest_version = api::modrinth::get_latest_version_from_hash(
                        hash,
                        HashMethod::Sha1,
                        &loaders,
                        &game_version,
                    )
                    .await
                    .unwrap();
                    let version_file = latest_version.get_primary_file();
                    download_version_file(&version_file, &path).await.unwrap();
                    remove_file(&path).unwrap();
                }
                synced_mods.insert(project.slug);
            }
            // Remove if not in mods.toml
            None => {
                cprintln!(
                    "<r>Removing</r> {} = {}...",
                    project.slug,
                    cur_version.version_number
                );
                fs::remove_file(&path).unwrap();
            }
        }
    }

    // Download other mods
    for (mod_name, version_number) in config
        .mods
        .iter()
        .filter(|(k, _)| !synced_mods.contains(*k))
    {
        if let Err(err) = api::modrinth::download_mod(
            mod_name,
            version_number,
            config.loader,
            &config.version,
            current_dir,
        )
        .await
        {
            cprintln!(
                "<r!>Error</> failed to download {} = {}: {}",
                mod_name,
                version_number,
                err
            )
        }
    }
}

#[tokio::main]
pub async fn update<P: AsRef<Path>>(current_dir: P) {
    let current_dir = current_dir.as_ref();

    info!("loading mods.toml...");
    let mut config = ModConfig::load(current_dir.join("mods.toml")).unwrap();

    info!("updating mods...");
    for file in fs::read_dir(current_dir).unwrap() {
        let file = file.unwrap();
        let path = file.path();
        if path.extension().unwrap() == "jar" {
            if let Ok((slug, version)) =
                api::modrinth::update_mod(path, config.loader, &config.version).await
            {
                config.insert_mod(slug, version);
            }
        }
    }

    cprintln!("updating mods.toml...");
    config.save(current_dir.join("mods.toml")).unwrap();
    cprintln!("done!")
}

#[tokio::main]
pub async fn add<P: AsRef<Path>>(slugs: Vec<String>, current_dir: P) {
    let current_dir = current_dir.as_ref();

    info!("loading mods.toml...");
    let mut config = ModConfig::load(current_dir.join("mods.toml")).unwrap();

    for slug in slugs {
        cprint!("<g>Adding</> {slug}...");
        if config.mods.contains_key(&slug) {
            cprintln!("already exists, skipped.");
            return;
        }
        cprintln!();
        match add_mod(slug, config.loader, config.version.clone(), current_dir).await {
            Ok((slug, version)) => {
                config.insert_mod(slug, version);
                config.save(current_dir.join("mods.toml")).unwrap();
            }
            Err(err) => cprintln!("<r>err</>: {err}"),
        }
    }
    cprintln!("done!")
}

fn get_jar_files(dir: &Path) -> Vec<DirEntry> {
    fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().unwrap() == "jar")
        .collect::<Vec<DirEntry>>()
}
