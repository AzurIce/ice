use std::{fmt::Display, vec};

use anyhow::Context;
use serde::Serialize;
use serde_json::json;
use types::{Loader, Project, Version};

const HOST: &str = "https://api.modrinth.com/v2";

pub async fn get_project<S: AsRef<str>>(id_or_slug: S) -> Result<Project, anyhow::Error> {
    let id_or_slug = id_or_slug.as_ref();

    let res = reqwest::get(format!("{HOST}/project/{id_or_slug}")).await?;
    let project = res.json::<Project>().await?;
    Ok(project)
}

pub async fn get_project_versions<S: AsRef<str>>(
    id_or_slug: S,
    loaders: Option<&Vec<Loader>>,
    game_version: Option<String>,
) -> Result<Vec<Version>, anyhow::Error> {
    let slug = id_or_slug.as_ref();
    let url = format!("{HOST}/project/{slug}/version");
    let mut params = vec![];
    if let Some(loaders) = loaders {
        params.push(("loaders", serde_json::to_string(loaders).unwrap()));
    }
    if let Some(game_version) = game_version {
        params.push((
            "game_versions",
            format!(
                "[{}]",
                vec![game_version]
                    .into_iter()
                    .map(|s| format!(r#""{s}""#))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        ))
    }
    let url = reqwest::Url::parse_with_params(&url, params)?;

    let res = reqwest::get(url).await?;
    let versions = res.json::<Vec<Version>>().await?;
    Ok(versions)
}

/// Get `Version` from id
pub async fn get_version<S: AsRef<str>>(id: S) -> Result<Version, anyhow::Error> {
    let id = id.as_ref();
    let url = format!("{HOST}/version/{id}");

    let url = reqwest::Url::parse(&url)?;
    let res = reqwest::get(url).await?;
    let version = res.json::<Version>().await?;
    Ok(version)
}

// Get `Version` from the hash of the file
pub async fn get_version_from_hash<H: AsRef<str>>(
    hash: H,
    hash_method: HashMethod,
) -> Result<Version, anyhow::Error> {
    let hash = hash.as_ref();
    let url = format!("{HOST}/version_file/{hash}");

    let params = [("algorithm", hash_method.to_string())];
    let url = reqwest::Url::parse_with_params(&url, params)?;
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
) -> Result<Version, anyhow::Error> {
    let hash = hash.as_ref();
    let game_version = game_version.as_ref();

    let url = format!("{HOST}/version_file/{hash}/update");
    let params = [("algorithm", hash_method.to_string())];
    let url = reqwest::Url::parse_with_params(&url, params)?;

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .json(&json!({
            "loaders": loaders,
            "game_versions": [game_version],
        }))
        .send()
        .await
        .context("error getting response")?;
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
) -> Result<Version, anyhow::Error> {
    let slug = slug.as_ref();
    let game_version = game_version.as_ref();

    let versions = get_project_versions(slug, None, None).await?;
    versions
        .into_iter()
        .find(|v| {
            v.game_versions.contains(&game_version.to_string())
                && (loaders.iter().any(|l| v.loaders.contains(l)))
        })
        .ok_or(anyhow::anyhow!(
            "cannot find a version of {slug} satisfied {loaders:?} and {game_version}"
        ))
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
    use std::path::Path;

    use anyhow::Context;
    use ice_util::download_from_url;

    use super::types::VersionFile;

    pub async fn download_version_file<P: AsRef<Path>>(
        version_file: &VersionFile,
        dir: P,
    ) -> Result<(), anyhow::Error> {
        let dir = dir.as_ref();
        let path = dir.join(&version_file.filename);
        if path.exists() {
            return Ok(());
            // return Err("already exists".into());
        }
        download_from_url(&version_file.url, path, |_| {})
            .await
            .context("failed to download from url")
    }
}

pub mod types {
    use std::fmt::{self, Display, Formatter};

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(rename_all = "lowercase")]
    pub enum Loader {
        Quilt,
        Fabric,
        Forge,
        NeoForge,
        LiteLoader,
        Rift,
        Bukkit,
        Folia,
        Paper,
        Spigot,
        Sponge,
        Bungeecord,
        Datapack,
        PurPur,
        Ornithe,
        Babric,
        #[serde(rename(serialize = "bta-babric", deserialize = "bta-babric"))]
        BtaBabric,
        Canvas,
        Iris,
        #[serde(rename(serialize = "java-agent", deserialize = "java-agent"))]
        JavaAgent,
        #[serde(rename(serialize = "legacy-fabric", deserialize = "legacy-fabric"))]
        LegacyFabric,
        Minecraft,
        Modloader,
        Nilloader,
        Optifine,
        Vanilla,
        Velocity,
        Waterfall,
        // #[serde(untagged)]
        // Unknown(String)
    }

    impl Display for Loader {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                Self::Quilt => "quilt",
                Self::Fabric => "fabric",
                Self::Forge => "forge",
                Self::NeoForge => "neoforge",
                Self::LiteLoader => "liteloader",
                Self::Rift => "rift",
                Self::Bukkit => "bukkit",
                Self::Folia => "folia",
                Self::Paper => "paper",
                Self::Spigot => "spigot",
                Self::Sponge => "sponge",
                Self::Bungeecord => "bungeecord",
                Self::Datapack => "datapack",
                Self::PurPur => "purpur",
                Self::Ornithe => "ornithe",
                Self::Babric => "babric",
                Self::BtaBabric => "bta-babric",
                Self::Canvas => "canvas",
                Self::Iris => "iris",
                Self::JavaAgent => "java-agent",
                Self::LegacyFabric => "legacy-fabric",
                Self::Minecraft => "minecraft",
                Self::Modloader => "modloader",
                Self::Nilloader => "nilloader",
                Self::Optifine => "optifine",
                Self::Vanilla => "vanilla",
                Self::Velocity => "velocity",
                Self::Waterfall => "waterfall",
                // Self::Unknown(tag) => tag,
            })
        }
    }

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
    use macro_rules_attribute::apply;
    use serde::Deserialize;
    use smol_macros::test;

    use super::types::*;
    use super::*;

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

    #[apply(test!)]
    async fn test_loader() {
        let res = reqwest::get("https://api.modrinth.com/v2/tag/loader")
            .await
            .unwrap();
        #[derive(Serialize, Deserialize)]
        struct LoaderTag {
            icon: String,
            name: Loader,
            supported_project_types: Vec<String>,
        }
        let _loader_tags = res.json::<Vec<LoaderTag>>().await.unwrap();
        // let loaders = _loader_tags.iter().map(|tag| tag.name.clone());
        // let unknown_loaders = loaders.filter_map(|loader| if let Loader::Unknown(loader) = loader {
        //     Some(loader)
        // } else {
        //     None
        // }).collect::<Vec<_>>();
        // if !unknown_loaders.is_empty() {
        //     panic!("has {} unknown_loaders: {:?}", unknown_loaders.len(), unknown_loaders)
        // }
    }

    #[apply(test!)]
    async fn test_get_project_versions() {
        let versions = get_project_versions(
            "terralith",
            Some(&vec![Loader::Quilt, Loader::Fabric]),
            Some("1.20.1".to_string()),
        )
        .await;
        println!("{versions:?}")
    }

    #[apply(test!)]
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

    #[apply(test!)]
    async fn test_get_latest_version_from_slug() {
        let version = get_latest_version_from_slug("minihud", vec![Loader::Fabric], "1.21.6").await;
        println!("{version:?}");
    }

    // #[apply(test!)]
    #[tokio::test]
    async fn test_get_latest_version_from_hash() {
        // Compat::new(async {
        // let hashes = &iris_version().files[0].hashes;
        let version = get_latest_version_from_hash(
            // &hashes.sha1,
            "5cb96cdb2ad2cda38d4fb268e5eae95d00e853c2",
            HashMethod::Sha1,
            &vec![Loader::Fabric],
            "1.21.6",
        )
        .await;
        println!("{version:?}");
        // })
        // .await
    }
}
