use std::{error::Error, fmt::Display, fs, path::Path, vec};

use color_print::{cprint, cprintln};
use ice_core::Loader;
use ice_util::fs::get_sha1_hash;
use serde::Serialize;
use serde_json::json;
use types::{Project, Version};
use utils::download_version_file;

/// Download the latest version of `slug` to `mod_dir`
///
/// the downloaded version is satisfied to `loaders` and `game_version`
pub async fn download_latest_mod<S: AsRef<str>, V: AsRef<str>, P: AsRef<Path>>(
    slug: S,
    loader: Loader,
    game_version: V,
    dir: P,
) -> Result<(), Box<dyn Error>> {
    let loaders = if let Loader::Quilt = loader {
        vec![Loader::Quilt, Loader::Fabric]
    } else {
        vec![loader]
    };
    let version = get_latest_version_from_slug(slug, loaders, game_version).await?;
    download_version_file(version.get_primary_file(), dir)
        .await
        .map_err(|err| format!("failed to download: {err}"))?;
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
) -> Result<(), Box<dyn Error>> {
    let slug = slug.as_ref();
    let version_number = version_number.as_ref();
    let game_version = game_version.as_ref();
    let dir = dir.as_ref();
    cprint!("<g>Syncing</> [{loader:?}] {slug} = {version_number}...");

    let loaders = if let Loader::Quilt = loader {
        vec![Loader::Quilt, Loader::Fabric]
    } else {
        vec![loader]
    };
    let versions = get_project_versions(slug).await?;
    match versions.iter().find(|v| {
        v.version_number == version_number
            && loaders.iter().any(|l| v.loaders.contains(l))
            && v.game_versions.contains(&game_version.to_string())
    }) {
        Some(version) => {
            let version_file = version.get_primary_file();

            if dir.join(&version_file.filename).exists() {
                return Err("already exists".into());
            } else {
                println!();
                download_version_file(version_file, dir)
                    .await
                    .map_err(|err| format!("failed to download: {err}"))?;
            }
        }
        None => {
            return Err("version not found".into());
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
) -> Result<(String, String), Box<dyn Error>> {
    let slug = slug.as_ref();
    let dir = dir.as_ref();

    let loaders = if let Loader::Quilt = loader {
        vec![Loader::Quilt, Loader::Fabric]
    } else {
        vec![loader]
    };
    let version = get_latest_version_from_slug(slug, loaders, game_version).await?;
    download_version_file(version.get_primary_file(), dir)
        .await
        .map_err(|err| format!("failed to download: {err}"))?;
    Ok((slug.to_string(), version.version_number))
}

/// Update the mod from `path`
///
/// if success return the new version_number
pub async fn update_mod<P: AsRef<Path>, S: AsRef<str>>(
    path: P,
    loader: Loader,
    game_version: S,
) -> Result<(String, String), Box<dyn Error>> {
    let path = path.as_ref();
    let dir = path.parent().unwrap();

    let hash = get_sha1_hash(path).map_err(|err| format!("failed to get hash: {err}"))?;
    let loaders = if let Loader::Quilt = loader {
        vec![Loader::Quilt, Loader::Fabric]
    } else {
        vec![loader]
    };

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
        fs::remove_file(path).map_err(|err| format!("failed to remove old version: {err}"))?;
    }
    Ok((project.slug, new_version.version_number))
}

const HOST: &str = "https://api.modrinth.com/v2";

pub async fn get_project<S: AsRef<str>>(id_or_slug: S) -> Result<Project, Box<dyn Error>> {
    let id_or_slug = id_or_slug.as_ref();

    let res = reqwest::get(format!("{HOST}/project/{id_or_slug}")).await?;
    let project = res.json::<Project>().await?;
    Ok(project)
}

pub async fn get_project_versions<S: AsRef<str>>(
    id_or_slug: S,
) -> Result<Vec<Version>, Box<dyn Error>> {
    let slug = id_or_slug.as_ref();

    let res = reqwest::get(format!("{HOST}/project/{slug}/version")).await?;
    let versions = res.json::<Vec<Version>>().await?;
    Ok(versions)
}

pub async fn get_version_from_hash<H: AsRef<str>>(
    hash: H,
    hash_method: HashMethod,
) -> Result<Version, Box<dyn Error>> {
    let hash = hash.as_ref();
    let url = format!("{HOST}/version_file/{hash}");

    let params = [("algorithm", hash_method.to_string())];
    let url = reqwest::Url::parse_with_params(&url, params)
        .map_err(|err| format!("failed to parse url: {err}"))?;
    let res = reqwest::get(url).await?;
    let version = res.json::<Version>().await?;
    Ok(version)
}

/// Get the latest version of a mod from a hash
///
/// if the version supports on of the loaders, it will consider valid,
/// same as game_version(current only on version is provided)
pub async fn get_latest_version_from_hash<H: AsRef<str>, V: AsRef<str>>(
    hash: H,
    hash_method: HashMethod,
    loaders: &Vec<Loader>,
    game_version: V,
) -> Result<Version, Box<dyn Error>> {
    let hash = hash.as_ref();
    let game_version = game_version.as_ref();

    let url = format!("{HOST}/version_file/{hash}/update");
    let params = [("algorithm", hash_method.to_string())];
    let url = reqwest::Url::parse_with_params(&url, params)
        .map_err(|err| format!("failed to parse url: {err}"))?;

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .json(&json!({
            "loaders": loaders,
            "game_versions": [game_version],
        }))
        .send()
        .await?;
    let version = res.json::<Version>().await?;
    Ok(version)
}

/// get the latest version of `slug`
///
/// if the version supports on of the loaders, it will consider valid,
/// same as game_version(current only on version is provided)
pub async fn get_latest_version_from_slug<S: AsRef<str>, V: AsRef<str>>(
    slug: S,
    loaders: Vec<Loader>,
    game_version: V,
) -> Result<Version, Box<dyn Error>> {
    let slug = slug.as_ref();
    let game_version = game_version.as_ref();

    let versions = get_project_versions(slug).await?;
    versions
        .into_iter()
        .find(|v| {
            v.game_versions.contains(&game_version.to_string())
                && (loaders.iter().any(|l| v.loaders.contains(l)))
        })
        .ok_or(
            format!("cannot find a version of {slug} satisfied {loaders:?} and {game_version}")
                .into(),
        )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HashMethod {
    Sha1,
    Sha512,
}

impl Display for HashMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            HashMethod::Sha1 => "sha1",
            HashMethod::Sha512 => "sha512",
        })
    }
}

pub mod utils {
    use std::{error::Error, path::Path};

    use ice_util::download_from_url;

    use super::types::VersionFile;

    pub async fn download_version_file<P: AsRef<Path>>(
        version_file: &VersionFile,
        dir: P,
    ) -> Result<(), Box<dyn Error>> {
        let dir = dir.as_ref();
        let path = dir.join(&version_file.filename);
        if path.exists() {
            return Err("already exists".into());
        }
        download_from_url(&version_file.url, path, |_| {}).await
    }
}

pub mod types {
    use ice_core::Loader;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    pub struct Project {
        pub slug: String,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    pub struct Version {
        pub name: String,
        pub version_number: String,
        pub game_versions: Vec<String>,
        pub loaders: Vec<Loader>,
        pub id: String,
        pub project_id: String,
        pub files: Vec<VersionFile>,
    }

    impl Version {
        pub fn get_primary_file(&self) -> &VersionFile {
            self.files
                .iter()
                .find(|f| f.primary)
                .unwrap_or(self.files.first().unwrap())
        }
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    pub struct VersionFile {
        pub hashes: VersionFileHashes,
        pub url: String,
        pub filename: String,
        pub primary: bool,
        pub size: i32,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    pub struct VersionFileHashes {
        pub sha1: String,
        pub sha512: String,
    }
}

#[cfg(test)]
mod test {
    use super::types::*;
    use super::*;
    use super::{get_project_versions, HashMethod};

    fn iris_version() -> Version {
        Version {
            name: "Iris 1.7.3 for Minecraft 1.21".to_string(),
            version_number: "1.7.3+1.21".to_string(),
            game_versions: vec!["1.21".to_string()],
            loaders: vec![Loader::Fabric, Loader::Quilt],
            id: "kuOV4Ece".to_string(),
            project_id: "YL57xq9U".to_string(),
            files: vec![
                VersionFile {
                    hashes: VersionFileHashes {
                        sha1: "1b4e44a74c850d407518bc5cbe33eec9577ad9e9".to_string(),
                        sha512: "bcd9b4f5cef1521c00b59cc9409725556922a077c60f62ba834dca680e29f334660a7c68150ff827779fb888585042b1ba687dfce6fd4d650507d9c59f023703".to_string()
                    },
                    url: "https://cdn.modrinth.com/data/YL57xq9U/versions/kuOV4Ece/iris-1.7.3%2Bmc1.21.jar".to_string(),
                    filename: "iris-1.7.3+mc1.21.jar".to_string(),
                    primary: true,
                    size: 2707610
                }
            ]
        }
    }

    #[tokio::test]
    async fn test_get_project_versions() {
        let versions = get_project_versions("iris").await;
        println!("{versions:?}")
    }

    #[tokio::test]
    async fn test_get_version_from_hash() {
        let version_sha1 =
            get_version_from_hash(&iris_version().files[0].hashes.sha1, HashMethod::Sha1)
                .await
                .unwrap();
        let version_sha512 =
            get_version_from_hash(&iris_version().files[0].hashes.sha512, HashMethod::Sha512)
                .await
                .unwrap();
        assert_eq!(version_sha1, version_sha512);
    }

    #[tokio::test]
    async fn test_get_latest_version_from_hash() {
        let hashes = &iris_version().files[0].hashes;
        let version = get_latest_version_from_hash(
            &hashes.sha1,
            HashMethod::Sha1,
            &vec![Loader::Quilt],
            "1.21",
        )
        .await;
        println!("{version:?}");
    }
}
