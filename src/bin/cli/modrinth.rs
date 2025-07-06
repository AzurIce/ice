use std::{
    collections::HashSet,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use color_print::cformat;
use futures::{stream, StreamExt};
use ice::{
    config::LocalModsConfig,
    core::{Mod, ModrinthMod},
};
use ice_api_tool::{
    self as api,
    modrinth::{get_latest_version_from_slug, utils::download_version_file, HashMethod},
};
use ice_core::Loader;
use ice_util::fs::get_sha1_hash;
use indicatif::ProgressStyle;
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
                info!("{}", cformat!("<r>Failed</> {:?}", err));
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
            Mod::Modrinth(modrinth_mod) => download_modrinth_mod(&modrinth_mod, &current_dir)
                .await
                .map_err(|err| {
                    anyhow::anyhow!(format!(
                        "download mod {} = {} ({})",
                        modrinth_mod.slug,
                        modrinth_mod.version_id_number,
                        err.root_cause()
                    ))
                })
                .map(|_| modrinth_mod),
            _ => unreachable!(),
        }
    })
    .buffer_unordered(5);

    while let Some(res) = stream.next().await {
        match res {
            Ok(res) => {
                info!(
                    "{}",
                    cformat!("<g>Downloaded</> {} = {}", res.slug, res.version_id_number)
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
                UpdateRes::Updated {
                    slug,
                    version_id,
                    version_number,
                } => {
                    config.insert_mod(Mod::Modrinth(ModrinthMod {
                        slug: slug.clone(),
                        version_id_number: format!("{}#{}", version_id, version_number),
                    }));
                    config.save().unwrap();
                    info!(
                        "{}",
                        cformat!("<g>Updated</> {} = {}#{}", slug, version_id, version_number)
                    );
                }
                UpdateRes::Unchanged {
                    slug,
                    version_id,
                    version_number,
                } => {
                    info!(
                        "{}",
                        cformat!(
                            "<y>Unchanged</> {} = {}#{}",
                            slug,
                            version_id,
                            version_number
                        )
                    );
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
                AddRes::Added {
                    slug,
                    version_id,
                    version_number,
                } => {
                    config.insert_mod(Mod::Modrinth(ModrinthMod {
                        slug: slug.clone(),
                        version_id_number: format!("{}#{}", version_id, version_number),
                    }));
                    config.save().unwrap();
                    info!(
                        "{}",
                        cformat!("<g>Added</> {} = {}#{}", slug, version_id, version_number)
                    );
                }
                AddRes::AlreadyExist {
                    slug,
                    version_id,
                    version_number,
                } => {
                    info!(
                        "{}",
                        cformat!(
                            "<y>Already Exist</> {} = {}#{}",
                            slug,
                            version_id,
                            version_number
                        )
                    );
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
    modrinth_mod: &ModrinthMod,
    dir: impl AsRef<Path>,
) -> Result<(), anyhow::Error> {
    let span = info_span!(
        "downloading",
        slug = modrinth_mod.slug,
        version_id_number = modrinth_mod.version_id_number
    );
    span.pb_set_style(
        &ProgressStyle::with_template(
            "{span_child_prefix}{spinner} {span_name:.bold}{{{span_fields}}}:\n  {msg:.dim}",
        )
        .unwrap(),
    );
    async {
        let span = Span::current();

        span.pb_set_message("fetching project version...");
        let version = api::modrinth::get_version(modrinth_mod.version_id()).await?;

        span.pb_set_message("downloading...");
        let version_file = version.get_primary_file();
        download_version_file(version_file, dir).await?;

        Ok(())
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
                if modrinth_mod.version_id() != version.id {
                    span.pb_set_message("version not match, redownloading...");
                    remove_file(&path)?;
                    download_modrinth_mod(&modrinth_mod, &current_dir).await?;
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
    Updated {
        slug: String,
        version_id: String,
        version_number: String,
    },
    Unchanged {
        slug: String,
        version_id: String,
        version_number: String,
    },
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
        let game_version = config.version.clone();

        span.pb_set_message("fetching latest version...");
        let version = api::modrinth::get_latest_version_from_hash(
            &hash,
            HashMethod::Sha1,
            &loaders,
            game_version.clone(),
        )
        .await?;

        if version.id == cur_version.id {
            return Ok(UpdateRes::Unchanged {
                slug: project.slug,
                version_id: cur_version.id,
                version_number: cur_version.version_number,
            });
        }

        span.pb_set_message("downloading...");
        let version_file = version.get_primary_file();
        download_version_file(version_file, current_dir).await?;
        span.pb_set_message("removing old file...");
        remove_file(path)?;
        Ok(UpdateRes::Updated {
            slug: project.slug,
            version_id: version.id,
            version_number: version.version_number,
        })
    }
    .instrument(span)
    .await
}

enum AddRes {
    Added {
        slug: String,
        version_id: String,
        version_number: String,
    },
    AlreadyExist {
        slug: String,
        version_id: String,
        version_number: String,
    },
}

fn ice_core_loader_to_modrinth_loader(
    value: ice_core::Loader,
) -> ice_api_tool::modrinth::types::Loader {
    match value {
        ice_core::Loader::Fabric => ice_api_tool::modrinth::types::Loader::Fabric,
        ice_core::Loader::Quilt => ice_api_tool::modrinth::types::Loader::Quilt,
    }
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
                return Ok(AddRes::AlreadyExist {
                    slug: modrinth_mod.slug.clone(),
                    version_number: modrinth_mod.version_number().to_string(),
                    version_id: modrinth_mod.version_id().to_string(),
                });
            }
        }

        let game_version = config.version.clone();
        let loader = ice_core_loader_to_modrinth_loader(config.loader);

        span.pb_set_message("fetching latest version...");
        let version =
            get_latest_version_from_slug(&slug, vec![loader], game_version.clone()).await?;

        span.pb_set_message("downloading...");
        let version_file = version.get_primary_file();
        download_version_file(version_file, current_dir).await?;
        Ok(AddRes::Added {
            slug,
            version_number: version.version_number,
            version_id: version.id,
        })
    }
    .instrument(span)
    .await
}

fn get_jar_files(dir: &Path) -> Vec<DirEntry> {
    if !dir.exists() {
        return vec![];
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

// async fn get_modrinth_files(dir: &Path) -> Vec<(DirEntry, String, Version)> {
//     //? Get modrinth files
//     let jar_files = get_jar_files(dir);
//     let mut join_set = JoinSet::new();
//     for f in jar_files {
//         join_set.spawn(async {
//             let path = f.path();
//             let hash = tokio::task::spawn_blocking(|| get_sha1_hash(path).unwrap())
//                 .await
//                 .unwrap();
//             let version = api::modrinth::get_version_from_hash(&hash, HashMethod::Sha1).await;
//             version.map(|v| (f, hash, v)).ok()
//         });
//     }
//     let mut modrinth_files = vec![];
//     while let Some(res) = join_set.join_next().await {
//         let res = res.unwrap();
//         if let Some(res) = res {
//             modrinth_files.push(res)
//         }
//     }
//     modrinth_files
// }
