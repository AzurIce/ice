use std::path::Path;

use color_print::{cprint, cprintln};
use ice_api_tool::modrinth::{utils::download_version_file, *};
use ice_core::Loader;
use ice_util::fs::get_sha1_hash;

/// Download the latest version of `slug` to `mod_dir`
///
/// the downloaded version is satisfied to `loaders` and `game_version`
pub async fn download_latest_mod<S: AsRef<str>, V: AsRef<str>, P: AsRef<Path>>(
    slug: S,
    loader: Loader,
    game_version: V,
    dir: P,
) -> Result<(), anyhow::Error> {
    let loaders = loader.to_compatible_loaders();
    let version = get_latest_version_from_slug(slug, loaders, game_version).await?;
    download_version_file(version.get_primary_file(), dir).await?;
    Ok(())
}

/// Download the version `version_number` of `slug` to `mod_dir`
///
/// the downloaded version is satisfied to `loader`
pub async fn download_mod<S: AsRef<str>, P: AsRef<Path>>(
    slug: S,
    version_number: S,
    loader: Loader,
    game_version: S,
    dir: P,
) -> Result<(), anyhow::Error> {
    let slug = slug.as_ref();
    let version_number = version_number.as_ref();
    let game_version = game_version.as_ref();
    let dir = dir.as_ref();
    cprint!("<g>Syncing</> [{loader:?}] {slug} = {version_number}...");

    let loaders = loader.to_compatible_loaders();
    let versions = get_project_versions(slug, None, None).await?;
    match versions.iter().find(|v| {
        v.version_number == version_number
            && loaders.iter().any(|l| v.loaders.contains(l))
            && v.game_versions.contains(&game_version.to_string())
    }) {
        Some(version) => {
            let version_file = version.get_primary_file();

            if dir.join(&version_file.filename).exists() {
                anyhow::bail!("already exists");
            } else {
                println!();
                download_version_file(version_file, dir).await?;
            }
        }
        None => {
            anyhow::bail!("version not found");
        }
    }
    Ok(())
}

/// Add a mod `slug`
///
/// use the latest version satisfies to `loader` and `game_version`
pub async fn add_mod<S: AsRef<str>, V: AsRef<str>, P: AsRef<Path>>(
    slug: S,
    loader: Loader,
    game_version: V,
    dir: P,
) -> Result<(String, String), anyhow::Error> {
    let slug = slug.as_ref();
    let dir = dir.as_ref();

    let loaders = loader.to_compatible_loaders();
    let version = get_latest_version_from_slug(slug, loaders, game_version).await?;
    download_version_file(version.get_primary_file(), dir).await?;
    Ok((slug.to_string(), version.version_number))
}


/// Update the mod from `path`
///
/// if success return the new version_number
pub async fn update_mod<P: AsRef<Path>, S: AsRef<str>>(
    path: P,
    loader: Loader,
    game_version: S,
) -> Result<(String, String), anyhow::Error> {
    let path = path.as_ref();
    let dir = path.parent().unwrap();

    let hash = get_sha1_hash(path)?;
    let loaders = loader.to_compatible_loaders();

    let cur_version = get_version_from_hash(&hash, HashMethod::Sha1).await?;
    let new_version =
        get_latest_version_from_hash(&hash, HashMethod::Sha1, &loaders, game_version).await?;
    let project = get_project(&new_version.project_id).await?;
    cprint!("<g!>Updating</> {}...", project.slug);
    if cur_version == new_version {
        cprintln!("is already latest, no need to update, skipped.")
    } else {
        cprintln!(
            "new version: {} -> {}",
            cur_version.version_number,
            new_version.version_number
        );
        let version_file = new_version.get_primary_file();
        if let Err(err) = download_version_file(version_file, dir).await {
            cprintln!("<r>error</>: {err}");
        }
        // cprintln!("removing old version...");
        std::fs::remove_file(path)?;
    }
    Ok((project.slug, new_version.version_number))
}