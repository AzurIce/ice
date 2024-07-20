use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    vec,
};

use color_print::{cprint, cprintln};
use serde::Serialize;
use serde_json::json;
use types::{Project, Version};

use crate::{loader::Loader, utils::fs::get_sha1_hash};

/// Download the latest version of `slug` to `mod_dir`
///
/// the downloaded version is satisfied to `loaders` and `game_version`
pub fn download_latest_mod<S: AsRef<str>, V: AsRef<str>>(
    slug: S,
    loader: Loader,
    game_version: V,
    mod_dir: PathBuf,
) {
    let loaders = if let Loader::Quilt = loader {
        vec![Loader::Quilt, Loader::Fabric]
    } else {
        vec![loader]
    };
    let version = get_latest_version_from_slug(slug, loaders, game_version).unwrap();
    version.download_to(mod_dir).unwrap();
}

/// Download the version `version_number` of `slug` to `mod_dir`
///
/// the downloaded version is satisfied to `loader`
pub fn download_mod<S: AsRef<str>, P: AsRef<Path>>(
    slug: S,
    version_number: S,
    loader: Loader,
    game_version: S,
    dir: P,
) {
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
    let versions = get_project_versions(slug);
    match versions.iter().find(|v| {
        v.version_number == version_number
            && loaders.iter().any(|l| v.loaders.contains(l))
            && v.game_versions.contains(&game_version.to_string())
    }) {
        Some(version) => {
            let filename = version.get_filename();
            if dir.join(filename).exists() {
                println!("already exists, skipping...");
            } else {
                println!();
                version.download_to(dir).unwrap()
            }
        }
        None => {
            println!("version not found, skipping...");
        }
    }
}

/// Add a mod `slug`
///
/// use the latest version satisfies to `loader` and `game_version`
pub fn add_mod<S: AsRef<str>, V: AsRef<str>, P: AsRef<Path>>(
    slug: S,
    loader: Loader,
    game_version: V,
    dir: P,
) -> Result<(String, String), String> {
    let slug = slug.as_ref();
    let dir = dir.as_ref();

    let loaders = if let Loader::Quilt = loader {
        vec![Loader::Quilt, Loader::Fabric]
    } else {
        vec![loader]
    };
    let version = get_latest_version_from_slug(slug, loaders, game_version).unwrap();
    version.download_to(dir)?;
    Ok((slug.to_string(), version.version_number))
}

/// Update the mod from `path`
///
/// if success return the new version_number
pub fn update_mod<P: AsRef<Path>, S: AsRef<str>>(
    path: P,
    loader: Loader,
    game_version: S,
) -> Result<(String, String), String> {
    let path = path.as_ref();
    let dir = path.parent().unwrap();

    let hash = get_sha1_hash(path).map_err(|err| format!("failed to get hash: {err}"))?;
    let loaders = if let Loader::Quilt = loader {
        vec![Loader::Quilt, Loader::Fabric]
    } else {
        vec![loader]
    };

    let cur_version = get_version_from_hash(&hash, HashMethod::Sha1)?;
    let new_version = get_latest_version_from_hash(&hash, HashMethod::Sha1, loaders, game_version)?;
    let project = get_project(&new_version.project_id);
    cprint!("<g!>Updating</> {}...", project.slug);
    if cur_version == new_version {
        cprintln!("is already latest, no need to update, skipped.")
    } else {
        cprintln!(
            "new version: {} -> {}",
            cur_version.version_number,
            new_version.version_number
        );
        if let Err(err) = new_version.download_to(dir) {
            cprintln!("<r>error</>: {err}");
        }
        // cprintln!("removing old version...");
        fs::remove_file(path).map_err(|err| format!("failed to remove old version: {err}"))?;
    }
    Ok((project.slug, new_version.version_number))
}

const HOST: &str = "https://api.modrinth.com/v2";

pub fn get_project<S: AsRef<str>>(id_or_slug: S) -> Project {
    let id_or_slug = id_or_slug.as_ref();
    let res = reqwest::blocking::get(format!("{HOST}/project/{id_or_slug}")).unwrap();
    res.json::<Project>().unwrap()
}

pub fn get_project_versions<S: AsRef<str>>(slug: S) -> Vec<Version> {
    let slug = slug.as_ref();
    let res = reqwest::blocking::get(format!("{HOST}/project/{slug}/version")).unwrap();
    res.json::<Vec<Version>>().unwrap()
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

pub fn get_version_from_hash<S: AsRef<str>>(
    hash: S,
    hash_method: HashMethod,
) -> Result<Version, String> {
    let hash = hash.as_ref();
    let url = format!("{HOST}/version_file/{hash}");
    let params = [("algorithm", hash_method.to_string())];
    let url = reqwest::Url::parse_with_params(&url, params)
        .map_err(|err| format!("failed to parse url: {err}"))?;
    let body = reqwest::blocking::get(url)
        .unwrap()
        .json::<Version>()
        .unwrap();
    Ok(body)
}

/// get the latest version of `slug`
///
/// if the version supports on of the loaders, it will consider valid,
/// same as game_version(current only on version is provided)
pub fn get_latest_version_from_slug<S: AsRef<str>, V: AsRef<str>>(
    slug: S,
    loaders: Vec<Loader>,
    game_version: V,
) -> Result<Version, String> {
    let slug = slug.as_ref();
    let game_version = game_version.as_ref();

    let versions = get_project_versions(slug);
    versions
        .into_iter()
        .find(|v| {
            v.game_versions.contains(&game_version.to_string())
                && (loaders.iter().any(|l| v.loaders.contains(l)))
        })
        .ok_or(format!(
            "cannot find a version of {slug} satisfied {loaders:?} and {game_version}"
        ))
}

/// get the latest version of a mod from a hash
///
/// if the version supports on of the loaders, it will consider valid,
/// same as game_version(current only on version is provided)
pub fn get_latest_version_from_hash<H: AsRef<str>, V: AsRef<str>>(
    hash: H,
    hash_method: HashMethod,
    loaders: Vec<Loader>,
    game_version: V,
) -> Result<Version, String> {
    let hash = hash.as_ref();
    let game_version = game_version.as_ref();

    let url = format!("{HOST}/version_file/{hash}/update");
    let params = [("algorithm", hash_method.to_string())];
    let url = reqwest::Url::parse_with_params(&url, params)
        .map_err(|err| format!("failed to parse url: {err}"))?;

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .json(&json!({
            "loaders": loaders,
            "game_versions": [game_version],
        }))
        .send()
        .unwrap();
    // println!("{res:?}");
    let body = res.json::<Version>().unwrap();
    Ok(body)
}

mod types {
    use std::path::Path;

    use color_print::cprintln;
    use serde::Deserialize;

    use crate::{loader::Loader, utils::download};

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
        pub fn get_filename(&self) -> String {
            let version_file = self
                .files
                .iter()
                .find(|f| f.primary)
                .unwrap_or(self.files.first().unwrap());
            version_file.filename.clone()
        }

        pub fn download_to<P: AsRef<Path>>(&self, dir: P) -> Result<(), String> {
            let dir = dir.as_ref();

            cprintln!(
                "<dim>downloading {:?} {} = {} to {:?}...</>",
                self.loaders,
                self.name,
                self.version_number,
                dir
            );
            let mut version_file = self.files.first().unwrap();
            if self.files.len() > 1 {
                cprintln!("<dim>multiple files found, downloading primary one...</>");
                if let Some(primary_version_file) = self.files.iter().find(|vf| vf.primary) {
                    version_file = primary_version_file;
                } else {
                    cprintln!("<dim>no primary file found, downloading the first one...</>");
                }
            }
            version_file.download_to(dir)?;
            Ok(())
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

    impl VersionFile {
        pub fn download_to<P: AsRef<Path>>(&self, dir: P) -> Result<(), String> {
            let dir = dir.as_ref();

            let local_path = dir.join(&self.filename);
            if !local_path.exists() {
                download(&self.url, local_path)
                    .map_err(|err| format!("failed to download: {err}"))?;
            }
            Ok(())
        }
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
    use crate::loader::Loader;

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

    #[test]
    fn test_get_project_versions() {
        let versions = get_project_versions("iris");
        println!("{versions:?}")
    }

    #[test]
    fn test_get_version_from_hash() {
        let version_sha1 =
            get_version_from_hash(&iris_version().files[0].hashes.sha1, HashMethod::Sha1).unwrap();
        let version_sha512 =
            get_version_from_hash(&iris_version().files[0].hashes.sha512, HashMethod::Sha512)
                .unwrap();
        assert_eq!(version_sha1, version_sha512);
    }

    #[test]
    fn test_get_latest_version_from_hash() {
        let hashes = &iris_version().files[0].hashes;
        let version = get_latest_version_from_hash(
            &hashes.sha1,
            HashMethod::Sha1,
            vec![Loader::Quilt],
            "1.21",
        );
        println!("{version:?}");
    }

    #[test]
    fn f() {
        // let res = get_project_versions("fabric-api");
        // println!("{:?}", res)
        let s = serde_json::to_string(&HashMethod::Sha1).unwrap();
        println!("{s:?}")
    }
}
