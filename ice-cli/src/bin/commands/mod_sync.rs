use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use ice_cli::{config::Config, core::loader::Loader, download, modrinth::get_project_versions};
use tracing::{info, instrument};

pub fn mod_sync_blocking<P: AsRef<Path>>(dir: P) {
    let dir = dir.as_ref();

    info!("loading config...");
    let ice_config = Config::load(dir.join("Ice.toml")).unwrap();

    for (mod_name, version_number) in &ice_config.mods {
        info!("downloading mod [{}]...", mod_name);
        download_mod_blocking(
            mod_name,
            version_number,
            ice_config.loader,
            dir.join("mods"),
        );
    }
}

async fn download_mod<S: AsRef<str>>(slug: S, version_number: S, loader: Loader, mod_dir: PathBuf) {
}

#[instrument]
fn download_mod_blocking<S: AsRef<str> + Debug>(
    slug: S,
    version_number: S,
    loader: Loader,
    mod_dir: PathBuf,
) {
    let slug = slug.as_ref();
    let version_number = version_number.as_ref();
    info!("downloading mod {slug} = [{loader:?}] {version_number}...");

    let versions = get_project_versions(&slug);
    match versions.iter().find(|v| {
        // println!(
        //     "{:?}:{} {:?}:{}",
        //     v.version_number,
        //     v.version_number == version_number,
        //     v.loaders,
        //     v.loaders.contains(&loader)
        //         || loader == Loader::Quilt && v.loaders.contains(&Loader::Fabric)
        // );
        v.version_number == version_number
            && (v.loaders.contains(&loader)
                || loader == Loader::Quilt && v.loaders.contains(&Loader::Fabric))
    }) {
        Some(version) => {
            if version.files.len() > 1 {
                info!("has multiple files: {:?}, skipping...", version.files);
                return;
            }
            let version_file = version.files.first().unwrap();
            let local_path = mod_dir.join(&version_file.filename);

            if !local_path.exists() {
                match download(&version_file.url, local_path) {
                    Ok(_) => info!("finished"),
                    Err(err) => info!("failed: {err}, skipping..."),
                }
            } else {
                info!("already exists, skipping...");
            }
        }
        None => {
            info!("version not found, skipping...");
        }
    }
}
