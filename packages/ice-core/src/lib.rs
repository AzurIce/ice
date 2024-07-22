use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs,
    io::Write,
    path::Path,
    process::Command,
};

use ice_util::download_from_url_blocking;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
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
        })
    }
}

impl Loader {
    pub fn installed<P: AsRef<Path>>(&self, current_dir: P) -> bool {
        let current_dir = current_dir.as_ref();
        current_dir.join("server").join("server.jar").exists()
    }

    pub fn install<P: AsRef<Path>, S: AsRef<str>>(
        &self,
        current_dir: P,
        game_version: S,
    ) -> Result<(), Box<dyn Error>> {
        let current_dir = current_dir.as_ref();
        let game_version = game_version.as_ref();

        if !matches!(self, Loader::Quilt) {
            return Err("not implemented".into());
        }

        let ice_dir = current_dir.join(".ice");
        fs::create_dir_all(&ice_dir).unwrap();
        let res = reqwest::blocking::get(
            "https://quiltmc.org/api/v1/download-latest-installer/java-universal",
        )?;

        let url = res.url();
        let filename = res
            .url()
            .path()
            .split("/")
            .last()
            .unwrap_or("quilt-installer");
        let installer_path = ice_dir.join(filename);
        download_from_url_blocking(url.as_str(), &installer_path, |_| {})?;

        info!("installing server");
        let success = Command::new("java")
            .current_dir(current_dir)
            .args([
                "-jar",
                installer_path.as_os_str().to_str().unwrap(),
                "install",
                "server",
                game_version,
                "--download-server",
            ])
            .status()?
            .success();
        if !success {
            panic!("failed to install server")
        }

        Ok(())
    }

    pub fn init_server_jar(&self, version: &str) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Quilt => {
                // 下载 Quilt installer
                info!("fetching quilt install info...");
                let res = reqwest::blocking::get(
                    "https://quiltmc.org/api/v1/download-latest-installer/java-universal",
                )?;

                let url = res.url();
                let filename = res
                    .url()
                    .path()
                    .split("/")
                    .last()
                    .unwrap_or("quilt-installer");
                let path = Path::new(".ice").join(filename);
                info!("downloading {filename} from {url} to {path:?}...");
                download_from_url_blocking(url.as_str(), &path, |_| {})?;

                info!("installing server");
                let success = Command::new("java")
                    .args([
                        "-jar",
                        path.as_os_str().to_str().unwrap(),
                        "install",
                        "server",
                        version,
                        "--download-server",
                    ])
                    .status()?
                    .success();
                if !success {
                    panic!("failed to install server")
                }

                // 写入 eula=true 到 eula.txt
                let mut eula_file =
                    fs::File::create("eula.txt").expect("failed to create eula file");
                eula_file
                    .write_all("eula=true".as_bytes())
                    .expect("failed to write into eula file");
            }
            _ => {
                println!("not implemented")
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_init_server_jar() {
        let loader = Loader::Quilt;
        loader
            .init_server_jar("1.20.4")
            .expect("failed to init server jar");
    }
}
