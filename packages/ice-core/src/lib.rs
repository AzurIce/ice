use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context;
use async_compat::Compat;
use ice_api_tool as api;
use ice_util::{download_from_url, get_url_filename};
use log::info;
use serde::{Deserialize, Serialize};

pub trait ServerLoaderTrait {
    fn get_latest_installer_url(game_version: &str) -> Result<String, anyhow::Error>;
    fn download_installer<P: AsRef<Path>>(
        current_dir: P,
        game_version: &str,
    ) -> Result<PathBuf, anyhow::Error> {
        let current_dir = current_dir.as_ref();
        let ice_dir = current_dir.join(".ice");
        if !ice_dir.exists() || ice_dir.is_file() {
            std::fs::create_dir_all(&ice_dir).context("failed to create ice dir")?;
        }

        let url = Self::get_latest_installer_url(game_version)
            .context("failed to get latest installer url")?;

        let filename = get_url_filename(url.as_str()).unwrap_or("loader-installer");
        let installer_path = ice_dir.join(filename);
        if installer_path.exists() {
            return Ok(installer_path);
        }
        smol::block_on(Compat::new(download_from_url(
            url.as_str(),
            &installer_path,
            |_| {},
        )))?;
        Ok(installer_path)
    }
    fn install_cmd(current_dir: &Path, installer_path: &Path, game_version: &str) -> Command;
    fn install<P: AsRef<Path>, S: AsRef<str>>(
        current_dir: P,
        game_version: S,
    ) -> Result<(), anyhow::Error> {
        let current_dir = current_dir.as_ref();
        let game_version = game_version.as_ref();

        let installer_path = Self::download_installer(current_dir, game_version)
            .context("failed to download installer")?;

        // Install
        info!("installing server...");
        if !current_dir.join("server").exists() {
            std::fs::create_dir(current_dir.join("server"))
                .expect("failed to create server folder");
        }

        let success = Self::install_cmd(current_dir, &installer_path, game_version)
            .status()?
            .success();
        if !success {
            panic!("failed to install server")
        }

        Ok(())
    }
}

// MARK: QuiltLoader
pub struct QuiltLoader;

impl ServerLoaderTrait for QuiltLoader {
    fn get_latest_installer_url(_game_version: &str) -> Result<String, anyhow::Error> {
        smol::block_on(Compat::new(api::quilt::get_latest_installer_url()))
    }
    fn install_cmd(current_dir: &Path, installer_path: &Path, game_version: &str) -> Command {
        let mut cmd = Command::new("java");
        cmd.current_dir(current_dir).args([
            "-jar",
            installer_path.as_os_str().to_str().unwrap(),
            "install",
            "server",
            game_version,
            "--download-server",
        ]);
        cmd
    }
}

// MARK: FabricLoader
pub struct FabricLoader;

impl ServerLoaderTrait for FabricLoader {
    fn get_latest_installer_url(_game_version: &str) -> Result<String, anyhow::Error> {
        smol::block_on(Compat::new(api::fabric::get_latest_installer_url()))
    }
    fn install_cmd(current_dir: &Path, installer_path: &Path, game_version: &str) -> Command {
        let mut cmd = Command::new("java");
        cmd.current_dir(current_dir).args([
            "-jar",
            installer_path.as_os_str().to_str().unwrap(),
            "server",
            "-dir",
            "server",
            "-mcversion",
            game_version,
            "-downloadMinecraft",
        ]);
        cmd
    }
}

// MARK: NeoForgeLoader
pub struct NeoForgeLoader;

impl ServerLoaderTrait for NeoForgeLoader {
    fn get_latest_installer_url(game_version: &str) -> Result<String, anyhow::Error> {
        smol::block_on(Compat::new(api::neoforge::get_latest_installer_url(
            game_version,
        )))
    }
    fn install_cmd(current_dir: &Path, installer_path: &Path, _game_version: &str) -> Command {
        let mut cmd = Command::new("java");
        cmd.current_dir(current_dir.join("server")).args([
            "-jar",
            installer_path.as_os_str().to_str().unwrap(),
            "--installServer",
        ]);
        cmd
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum ServerLoader {
    Quilt,
    Fabric,
    NeoForge,
}

impl ServerLoader {
    pub fn to_compatible_loaders(&self) -> Vec<ice_api_tool::modrinth::types::Loader> {
        use ice_api_tool::modrinth::types::Loader as ModrinthLoader;

        match self {
            ServerLoader::Fabric => vec![ModrinthLoader::Fabric],
            ServerLoader::Quilt => vec![ModrinthLoader::Fabric, ModrinthLoader::Quilt],
            ServerLoader::NeoForge => vec![ModrinthLoader::NeoForge],
        }
    }
}

mod convert {
    use super::ServerLoader;
    use ice_api_tool::modrinth::types::Loader as ModrinthLoader;

    impl Into<ModrinthLoader> for ServerLoader {
        fn into(self) -> ice_api_tool::modrinth::types::Loader {
            match self {
                ServerLoader::Fabric => ModrinthLoader::Fabric,
                ServerLoader::Quilt => ModrinthLoader::Quilt,
                ServerLoader::NeoForge => ModrinthLoader::NeoForge,
            }
        }
    }
}

impl Display for ServerLoader {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Quilt => "quilt",
            Self::Fabric => "fabric",
            Self::NeoForge => "neoforge",
        })
    }
}

impl ServerLoader {
    pub fn install<P: AsRef<Path>, S: AsRef<str>>(
        &self,
        current_dir: P,
        game_version: S,
    ) -> Result<(), Box<dyn Error>> {
        let current_dir = current_dir.as_ref();
        let game_version = game_version.as_ref();

        match self {
            ServerLoader::Fabric => FabricLoader::install(current_dir, game_version),
            ServerLoader::Quilt => QuiltLoader::install(current_dir, game_version),
            ServerLoader::NeoForge => NeoForgeLoader::install(current_dir, game_version),
        }?;

        Ok(())
    }
}

pub fn install_server<L: ServerLoaderTrait>(
    current_dir: impl AsRef<Path>,
    game_version: impl AsRef<str>,
) -> Result<(), anyhow::Error> {
    L::install(current_dir, game_version)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    fn test_install_server<L: ServerLoaderTrait>(
        path: impl AsRef<Path>,
        game_version: impl AsRef<str>,
    ) {
        L::install(path, game_version).unwrap();
    }

    #[test]
    fn test_install() {
        let root = env!("CARGO_MANIFEST_DIR");
        let test_dir = PathBuf::from(root).join("test");

        test_install_server::<QuiltLoader>(test_dir.join("quilt"), "1.21.1");
        test_install_server::<FabricLoader>(test_dir.join("fabric"), "1.21.1");
        test_install_server::<NeoForgeLoader>(test_dir.join("neoforge"), "1.21.1");
    }
}
