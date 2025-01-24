use std::{
    collections::HashSet,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use color_print::{cformat, cprintln};
use futures::{stream, StreamExt};
use ice::{
    config::LocalModsConfig,
    core::{Mod, ModrinthMod},
    // log::logger,
};
use ice_api_tool::{
    self as api,
    modrinth::{
        get_latest_version_from_hash, get_latest_version_from_slug, types::Version,
        utils::download_version_file, HashMethod,
    },
};
use ice_core::Loader;
use ice_util::{fs::get_sha1_hash, get_parent_version};
use indicatif::ProgressStyle;
use tokio::task::JoinSet;
use tracing::{info, info_span, Instrument, Span};
use tracing_indicatif::{span_ext::IndicatifSpanExt, IndicatifLayer};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logger() {
    let indicatif_layer = IndicatifLayer::new();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("ice=info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_level(false)
                .with_target(false)
                .without_time()
                .with_writer(indicatif_layer.get_stderr_writer()),
        )
        .with(indicatif_layer)
        .init();
}

// MARK: CLI

/// Initialize a `mods.toml` under `current_dir`
///
/// If `version` is `None` then it will default to the latest release
#[tokio::main]
pub async fn init<P: AsRef<Path>, S: AsRef<str>>(
    version: Option<S>,
    loader: Loader,
    current_dir: P,
) {
    let version = version.map(|s| s.as_ref().to_string());
    let current_dir = current_dir.as_ref();

    let config_path = current_dir.join("mods.toml");
    if config_path.exists() {
        println!("mods.toml is already exists!");
        return;
    }

    let config = LocalModsConfig::new(
        version.unwrap_or(api::mojang::get_latest_version().await.unwrap()),
        loader,
        config_path,
    );
    config.save().unwrap();
}

/// The `sync` command
#[tokio::main]
pub async fn sync<P: AsRef<Path>>(current_dir: P, config: &LocalModsConfig) {
    init_logger();

    let current_dir = current_dir.as_ref();

    let mut synced_mods = HashSet::<String>::new();

    // First, use [`sync_file`] to sync all existed mod files
    info!("checking existed mods...");
    let jar_files = get_jar_files(current_dir);
    let mut stream = stream::iter(jar_files)
        .map(|entry| async {
            let filename = entry.file_name();
            let filename = filename.to_str().unwrap();

            sync_file(entry, config, &current_dir)
                .await
                .context(format!("sync file {}", filename))
        })
        .buffer_unordered(5);

    while let Some(res) = stream.next().await {
        match res {
            Ok(res) => match res {
                SyncRes::Downloaded(slug, version) => {
                    synced_mods.insert(slug.clone());
                    info!("{}", cformat!("<g>Updated</> {} to {}", slug, version));
                }
                SyncRes::Removed(path) => {
                    info!(
                        "{}",
                        cformat!(
                            "<r>Removed</> {}",
                            path.file_name().unwrap().to_str().unwrap()
                        )
                    );
                }
                SyncRes::Unchanged(slug) => {
                    synced_mods.insert(slug.clone());
                    info!("{}", cformat!("<y>Unchanged</> {}", slug));
                }
                _ => (),
            },
            Err(err) => {
                info!("{}", cformat!("<r>Failed</> {}", err));
            }
        }
    }

    // Then, download mods not existed
    info!("downloading other mods...");
    let mut stream = stream::iter(config.get_mods().into_iter().filter(|value| {
        if let Mod::Modrinth(modrinth_mod) = value {
            !synced_mods.contains(&modrinth_mod.slug)
        } else {
            false
        }
    }))
    .map(|modrinth_mod| async {
        match modrinth_mod {
            Mod::Modrinth(modrinth_mod) => {
                download_modrinth_mod(modrinth_mod.clone(), &current_dir, config)
                    .await
                    .context(format!(
                        "download mod {} = {}",
                        modrinth_mod.slug, modrinth_mod.version
                    ))
            }
            _ => unreachable!(),
        }
    })
    .buffer_unordered(5);

    while let Some(res) = stream.next().await {
        match res {
            Ok(res) => {
                info!(
                    "{}",
                    cformat!("<g>Downloaded</> {} = {}", res.slug, res.version)
                );
            }
            Err(err) => {
                info!("{}", cformat!("<r>Failed</> {}", err));
            }
        }
    }
    info!("done!");
}

/// The `update` command
#[tokio::main]
pub async fn update(current_dir: impl AsRef<Path>, config: &mut LocalModsConfig) {
    init_logger();

    let current_dir = current_dir.as_ref();

    let _config = Arc::new(config.clone());
    let mut stream = stream::iter(get_jar_files(current_dir))
        .map(|file| {
            let filename = file.file_name();
            let filename = filename.into_string().unwrap();
            let _config = _config.clone();
            async move {
                update_mod(file, _config, current_dir)
                    .await
                    .context(format!("update mod {}", filename))
            }
        })
        .buffer_unordered(5);

    while let Some(res) = stream.next().await {
        match res {
            Ok(res) => match res {
                UpdateRes::Updated(slug, version) => {
                    config.insert_mod(Mod::Modrinth(ModrinthMod {
                        slug: slug.clone(),
                        version: version.clone(),
                    }));
                    config.save().unwrap();
                    info!("{}", cformat!("<g>Updated</> {} = {}", slug, version));
                }
                UpdateRes::Unchanged(slug, version) => {
                    info!("{}", cformat!("<y>Unchanged</> {} = {}", slug, version));
                }
                UpdateRes::Skipped => (),
            },
            Err(err) => {
                info!("{}", cformat!("<r>Failed</> {err}"));
            }
        }
    }

    info!("done!")
}

/// The `add` command
#[tokio::main]
pub async fn add(slugs: Vec<String>, current_dir: impl AsRef<Path>, config: &mut LocalModsConfig) {
    init_logger();

    let current_dir = current_dir.as_ref();

    let slugs: Vec<String> = slugs
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let _config = Arc::new(config.clone());
    let mut stream = stream::iter(slugs)
        .map(|slug| {
            let slug = slug.clone();
            async {
                let _slug = slug.clone();
                add_mod(slug, _config.clone(), current_dir)
                    .await
                    .context(format!("add mod {}", _slug))
            }
        })
        .buffer_unordered(5);

    while let Some(res) = stream.next().await {
        match res {
            Ok(res) => match res {
                AddRes::Added(slug, version) => {
                    config.insert_mod(Mod::Modrinth(ModrinthMod {
                        slug: slug.clone(),
                        version: version.clone(),
                    }));
                    config.save().unwrap();
                    info!("{}", cformat!("<g>Added</> {} = {}", slug, version));
                }
                AddRes::AlreadyExist(slug, version) => {
                    config.insert_mod(Mod::Modrinth(ModrinthMod {
                        slug: slug.clone(),
                        version: version.clone(),
                    }));
                    config.save().unwrap();
                    info!("{}", cformat!("<y>Already Exist</> {} = {}", slug, version));
                }
            },
            Err(err) => {
                info!("{}", cformat!("<r>Failed</> {err}"));
            }
        }
    }
    info!("done!")
}

// MARK: Inner

/// Download a modrinth mod
///
/// This function creates a span with spinner
async fn download_modrinth_mod(
    modrinth_mod: ModrinthMod,
    dir: impl AsRef<Path>,
    config: &LocalModsConfig,
) -> Result<ModrinthMod, anyhow::Error> {
    let span = info_span!(
        "downloading",
        slug = modrinth_mod.slug,
        version = modrinth_mod.version
    );
    span.pb_set_style(
        &ProgressStyle::with_template(
            "{span_child_prefix}{spinner} {span_name:.bold}{{{span_fields}}}:\n  {msg:.dim}",
        )
        .unwrap(),
    );
    async {
        let span = Span::current();

        let mut game_version = config.version.clone();
        let loaders = config.loader.to_compatible_loaders();

        span.pb_set_message("fetching project versions...");
        let mut version = api::modrinth::get_project_versions(
            modrinth_mod.slug.clone(),
            Some(&loaders),
            Some(game_version.clone()),
        )
        .await?;
        if version.is_empty() {
            game_version = get_parent_version(game_version);

            span.pb_set_message(
                "cannot match exact game version, refetching with parent game version...",
            );
            version = api::modrinth::get_project_versions(
                modrinth_mod.slug.clone(),
                Some(&loaders),
                Some(game_version.clone()),
            )
            .await?;
        }
        if version.is_empty() {
            anyhow::bail!(
                "cannot find any versions for {} under {}",
                modrinth_mod.slug,
                game_version
            );
        }

        let target_version = version
            .iter()
            .find(|v| {
                v.version_number == modrinth_mod.version
                    && loaders.iter().any(|l| v.loaders.contains(l))
                    && v.game_versions.contains(&game_version)
            })
            .ok_or(anyhow::anyhow!(
                "cannot find version {} for {} under {}",
                modrinth_mod.version,
                modrinth_mod.slug,
                game_version
            ))?;

        span.pb_set_message("downloading...");
        let version_file = target_version.get_primary_file();
        download_version_file(version_file, dir).await?;

        Ok(modrinth_mod)
    }
    .instrument(span)
    .await
}

/// Remove a file
///
/// This function creates a span with spinner
fn remove_file(path: impl AsRef<Path>) -> Result<(), anyhow::Error> {
    let path = path.as_ref();
    let span = info_span!(
        "removing",
        file = path.file_name().unwrap().to_str().unwrap()
    );
    span.pb_set_style(
        &ProgressStyle::with_template(
            "{span_child_prefix}{spinner} {span_name:.bold}{{{span_fields}}}:\n  {msg:.dim}",
        )
        .unwrap(),
    );
    span.in_scope(|| fs::remove_file(&path))?;
    Ok(())
}

#[derive(Debug)]
enum SyncRes {
    Skipped,
    Downloaded(String, String),
    Removed(PathBuf),
    Unchanged(String),
}

/// Sync a local .jar file with mods.toml
///
/// This function creates a span with spinner
///
/// If the file is a modrinth mod file:
/// - If the version is not match, remove the file and download the new version
/// - If the file is not in mods.toml, remove it
///
/// Else:
/// - Do nothing
async fn sync_file(
    entry: DirEntry,
    config: &LocalModsConfig,
    current_dir: impl AsRef<Path>,
) -> Result<SyncRes, anyhow::Error> {
    let span = info_span!("syncing", file = entry.file_name().to_str().unwrap());
    span.pb_set_style(
        &ProgressStyle::with_template(
            "{span_child_prefix}{spinner} {span_name:.bold}{{{span_fields}}}:\n  {msg:.dim}",
        )
        .unwrap(),
    );
    async {
        let span = Span::current();
        let path = entry.path();

        span.pb_set_message("calculating sha1 hash...");
        let _path = path.clone();
        let hash = tokio::task::spawn_blocking(move || get_sha1_hash(_path).unwrap())
            .await
            .unwrap();

        span.pb_set_message("fetching version...");
        let version = api::modrinth::get_version_from_hash(&hash, HashMethod::Sha1).await;
        if version.is_err() {
            return Ok(SyncRes::Skipped);
        }
        let version = version.unwrap();

        span.pb_set_message("fetching project...");
        let project = api::modrinth::get_project(version.project_id).await?;

        if let Some(value) = config.get_mod(&project.slug) {
            if let Mod::Modrinth(modrinth_mod) = value {
                if modrinth_mod.version != version.version_number {
                    span.pb_set_message("version not match, redownloading...");
                    remove_file(&path)?;
                    download_modrinth_mod(modrinth_mod, &current_dir, config).await?;
                    return Ok(SyncRes::Downloaded(project.slug, version.version_number));
                }
            }
            Ok(SyncRes::Unchanged(project.slug))
        } else {
            span.pb_set_message("not in mods.toml, removing...");
            remove_file(&path)?;
            Ok(SyncRes::Removed(path))
        }
    }
    .instrument(span)
    .await
}

enum UpdateRes {
    Skipped,
    Updated(String, String),
    Unchanged(String, String),
}

async fn update_mod(
    entry: DirEntry,
    config: Arc<LocalModsConfig>,
    current_dir: impl AsRef<Path>,
) -> Result<UpdateRes, anyhow::Error> {
    let path = entry.path();

    let span = info_span!(
        "updating",
        file = path.file_name().unwrap().to_str().unwrap()
    );
    span.pb_set_style(
        &ProgressStyle::with_template(
            "{span_child_prefix}{spinner} {span_name:.bold}{{{span_fields}}}:\n  {msg:.dim}",
        )
        .unwrap(),
    );

    async {
        let span = Span::current();
        let path = entry.path();

        span.pb_set_message("calculating sha1 hash...");
        let _path = path.clone();
        let hash = tokio::task::spawn_blocking(move || get_sha1_hash(_path).unwrap())
            .await
            .unwrap();

        let cur_version = api::modrinth::get_version_from_hash(&hash, HashMethod::Sha1).await;
        if cur_version.is_err() {
            return Ok(UpdateRes::Skipped);
        }
        let cur_version = cur_version.unwrap();

        span.pb_set_message("fetching project...");
        let project = api::modrinth::get_project(cur_version.project_id).await?;

        let loaders = config.loader.to_compatible_loaders();
        let mut game_version = config.version.clone();

        span.pb_set_message("fetching latest version...");
        let mut version = api::modrinth::get_latest_version_from_hash(
            &hash,
            HashMethod::Sha1,
            &loaders,
            game_version.clone(),
        )
        .await;
        if version.is_err() {
            game_version = get_parent_version(game_version);
            span.pb_set_message(&format!(
                "failed to get latest version of {} under {}, trying to get it under {game_version}",
                project.slug,config.version
            ));
            version = api::modrinth::get_latest_version_from_hash(
                &hash,
                HashMethod::Sha1,
                &loaders,
                game_version,
            )
            .await;
        }
        let version = version?;

        if version.version_number == cur_version.version_number {
            return Ok(UpdateRes::Unchanged(
                project.slug,
                cur_version.version_number,
            ));
        }

        span.pb_set_message("downloading...");
        let version_file = version.get_primary_file();
        download_version_file(version_file, current_dir).await?;
        span.pb_set_message("removing old file...");
        remove_file(path)?;
        Ok(UpdateRes::Updated(project.slug, version.version_number))
    }
    .instrument(span)
    .await
}

enum AddRes {
    Added(String, String),
    AlreadyExist(String, String),
}

async fn add_mod(
    slug: impl AsRef<str>,
    config: Arc<LocalModsConfig>,
    current_dir: impl AsRef<Path>,
) -> Result<AddRes, anyhow::Error> {
    let slug = slug.as_ref().to_string();

    let span = info_span!("adding", slug = slug);
    span.pb_set_style(
        &ProgressStyle::with_template(
            "{span_child_prefix}{spinner} {span_name:.bold}{{{span_fields}}}:\n  {msg:.dim}",
        )
        .unwrap(),
    );
    async {
        let span = Span::current();

        if let Some(value) = config.get_mod(&slug) {
            if let Mod::Modrinth(modrinth_mod) = value {
                return Ok(AddRes::AlreadyExist(slug.clone(), modrinth_mod.version));
            }
        }

        let loaders = config.loader.to_compatible_loaders();
        let mut game_version = config.version.clone();

        let mut version =
            get_latest_version_from_slug(&slug, loaders.clone(), game_version.clone()).await;
        if version.is_err() {
            game_version = get_parent_version(game_version);
            span.pb_set_message(&format!(
                "failed to get latest version of {slug} under {}, trying to get it under {game_version}",
                config.version
            ));
            version = get_latest_version_from_slug(&slug, loaders, game_version).await;
        }
        let version = version?;

        span.pb_set_message("downloading...");
        let version_file = version.get_primary_file();
        download_version_file(version_file, current_dir).await?;
        Ok(AddRes::Added(slug, version.version_number))
    }
    .instrument(span)
    .await
}

fn get_jar_files(dir: &Path) -> Vec<DirEntry> {
    if !dir.exists() {
        return vec![]
    }
    fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|s| s.to_str().unwrap())
                .unwrap_or("")
                == "jar"
        })
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
