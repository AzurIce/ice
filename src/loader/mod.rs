use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs,
    io::Write,
    path::Path,
    process::Command,
};

use clap::ValueEnum;
use ice_core::utils::download;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    Quilt,
    Fabric,
    Forge,
    NeoForge,
}

impl Display for Loader {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Quilt => "quilt",
            Self::Fabric => "fabric",
            Self::Forge => "forge",
            Self::NeoForge => "neoforge",
        })
    }
}

impl Loader {
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
                download(url.as_str(), &path)?;

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
