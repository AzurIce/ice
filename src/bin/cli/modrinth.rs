use std::{
    collections::HashSet,
    fs::{self, remove_file, DirEntry},
    path::{Path, PathBuf},
    sync::Arc,
};

use color_print::{cprint, cprintln};
use ice::{
    api::{
        self,
        modrinth::{
            add_mod, get_project_versions, types::Version, utils::download_version_file, HashMethod,
        },
    },
    config::ModConfig,
};
use ice_core::Loader;
use ice_util::fs::get_sha1_hash;
use log::info;
use tokio::task::JoinSet;

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

    let loaders = if let Loader::Quilt = config.loader {
        vec![Loader::Quilt, Loader::Fabric]
    } else {
        vec![config.loader]
    };
    let game_version = config.version.clone();
    let mods = Arc::new(config.mods.clone());

    //? Sync modrinth files
    enum SyncRes {
        Update(String, String),
        Remove(PathBuf),
        Unchanged(String),
    }
    let mut join_set = JoinSet::<Result<SyncRes, String>>::new();

    let modrinth_files = get_modrinth_files(current_dir).await;
    for (file, hash, cur_version) in modrinth_files {
        let loaders = loaders.clone();
        let game_version = game_version.clone();
        let mods = mods.clone();

        let path = file.path();
        let current_dir = current_dir.to_owned();
        join_set.spawn(async move {
            let project = api::modrinth::get_project(cur_version.project_id)
                .await
                .map_err(|err| format!("failed to get project: {err}"))?;
            let sync_type = match mods.get(&project.slug) {
                // Check if is in mods.toml
                Some(version_number) => {
                    // Update it version is not correct
                    if version_number != &cur_version.version_number
                        || !loaders.iter().any(|l| cur_version.loaders.contains(l))
                        || !cur_version.game_versions.contains(&game_version)
                    {
                        let latest_version = api::modrinth::get_latest_version_from_hash(
                            hash,
                            HashMethod::Sha1,
                            &loaders,
                            game_version,
                        )
                        .await
                        .unwrap();
                        let version_file = latest_version.get_primary_file();
                        download_version_file(&version_file, &current_dir)
                            .await
                            .unwrap();
                        remove_file(&path).unwrap();
                        SyncRes::Update(project.slug.clone(), version_number.clone())
                    } else {
                        SyncRes::Unchanged(project.slug)
                    }
                }
                // Remove if not in mods.toml
                None => {
                    fs::remove_file(&path).unwrap();
                    SyncRes::Remove(path.clone())
                }
            };
            Ok(sync_type)
        });
    }
    while let Some(res) = join_set.join_next().await {
        let res = res.unwrap();
        match res {
            Ok(res) => match res {
                SyncRes::Update(slug, version) => {
                    synced_mods.insert(slug.clone());
                    cprintln!("<g>Updated</> {} to {}", slug, version);
                }
                SyncRes::Remove(path) => {
                    cprintln!(
                        "<r>Removed</> {}",
                        path.file_name().unwrap().to_str().unwrap()
                    );
                }
                SyncRes::Unchanged(slug) => {
                    synced_mods.insert(slug.clone());
                    cprintln!("<y>Unchanged</> {}", slug);
                }
            },
            Err(err) => {
                cprintln!("<r>Failed</> {}", err);
            }
        }
    }

    //? Download other mods
    let mut join_set = JoinSet::<Result<(String, String), String>>::new();
    for (slug, version_number) in mods.iter().filter(|(k, _)| !synced_mods.contains(*k)) {
        let loaders = loaders.clone();
        let game_version = game_version.clone();

        let slug = slug.to_owned();
        let version_number = version_number.to_owned();
        let current_dir = current_dir.to_owned();
        join_set.spawn(async move {
            let versions = get_project_versions(&slug, Some(&loaders), Some(game_version.clone()))
                .await
                .map_err(|err| format!("failed to get project versions: {err}"))?;
            if let Some(version) = versions
                .into_iter()
                .find(|v| v.version_number == version_number)
            {
                let version_file = version.get_primary_file();
                let path = current_dir.join(format!("{}.jar", slug));
                download_version_file(&version_file, &path).await.unwrap();
                Ok((slug, version_number))
            } else {
                Err(format!("failed to find version {} = {}", slug, version_number).into())
            }
        });
    }
    while let Some(res) = join_set.join_next().await {
        let res = res.unwrap();
        match res {
            Ok((slug, version_number)) => {
                synced_mods.insert(slug.clone());
                cprintln!("<g>Downloaded</> {} = {}", slug, version_number);
            }
            Err(err) => {
                cprintln!("<r>Failed</> {err}")
            }
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

async fn get_modrinth_files(dir: &Path) -> Vec<(DirEntry, String, Version)> {
    //? Get modrinth files
    let jar_files = get_jar_files(dir);
    let mut join_set = JoinSet::new();
    for f in jar_files {
        join_set.spawn(async {
            let path = f.path();
            let hash = tokio::task::spawn_blocking(|| get_sha1_hash(path).unwrap())
                .await
                .unwrap();
            let version = api::modrinth::get_version_from_hash(&hash, HashMethod::Sha1).await;
            version.map(|v| (f, hash, v)).ok()
        });
    }
    let mut modrinth_files = vec![];
    while let Some(res) = join_set.join_next().await {
        let res = res.unwrap();
        if let Some(res) = res {
            modrinth_files.push(res)
        }
    }
    modrinth_files
}