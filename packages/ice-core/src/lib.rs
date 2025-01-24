use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs,
    path::Path,
    process::Command,
};

use ice_api_tool as api;
use ice_util::{download_from_url_blocking, get_url_filename};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    Quilt,
    Fabric,
}

impl Loader {
    pub fn to_compatible_loaders(&self) -> Vec<ice_api_tool::modrinth::types::Loader> {
        use ice_api_tool::modrinth::types::Loader as ModrinthLoader;

        match self {
            Loader::Fabric => vec![ModrinthLoader::Fabric],
            Loader::Quilt => vec![ModrinthLoader::Fabric, ModrinthLoader::Quilt],
        }
    }
}

mod convert {
    use super::Loader;
    use ice_api_tool::modrinth::types::Loader as ModrinthLoader;

    impl Into<ModrinthLoader> for Loader {
        fn into(self) -> ice_api_tool::modrinth::types::Loader {
            match self {
                Loader::Fabric => ModrinthLoader::Fabric,
                Loader::Quilt => ModrinthLoader::Quilt,
            }
        }
    }
}

impl Display for Loader {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Quilt => "quilt",
            Self::Fabric => "fabric",
        })
    }
}

impl Loader {
    pub fn launch_filename_str(&self) -> &str {
        match self {
            Loader::Fabric => "fabric-server-launch.jar",
            Loader::Quilt => "quilt-server-launch.jar",
        }
    }
    pub fn installed<P: AsRef<Path>>(&self, current_dir: P) -> bool {
        let current_dir = current_dir.as_ref();
        let server_dir = current_dir.join("server");

        server_dir.join("server.jar").exists() && server_dir.join(self.launch_filename_str()).exists()
    }

    pub fn install<P: AsRef<Path>, S: AsRef<str>>(
        &self,
        current_dir: P,
        game_version: S,
    ) -> Result<(), Box<dyn Error>> {
        let current_dir = current_dir.as_ref();
        let game_version = game_version.as_ref();

        if !matches!(self, Loader::Fabric | Loader::Quilt) {
            return Err("not implemented".into());
        }

        // Download installer
        let ice_dir = current_dir.join(".ice");
        fs::create_dir_all(&ice_dir).unwrap();
        let url = match self {
            Loader::Fabric => api::fabric::get_latest_installer_url()?,
            Loader::Quilt => reqwest::blocking::get(
                "https://quiltmc.org/api/v1/download-latest-installer/java-universal",
            )?
            .url()
            .as_str()
            .to_string(),
        };
        let filename = get_url_filename(url.as_str()).unwrap_or("quilt-installer");
        let installer_path = ice_dir.join(filename);
        download_from_url_blocking(url.as_str(), &installer_path, |_| {})?;

        // Install
        info!("installing server({self})...");
        if !current_dir.join("server").exists() {
            std::fs::create_dir(current_dir.join("server"))
                .expect("failed to create server folder");
        }
        let args: &[&str] = match self {
            Loader::Fabric => &[
                "server",
                "-dir",
                "server",
                "-mcversion",
                game_version,
                "-downloadMinecraft",
            ],
            Loader::Quilt => &["install", "server", game_version, "--download-server"],
        };
        let success = Command::new("java")
            .current_dir(current_dir)
            .args(
                ["-jar", installer_path.as_os_str().to_str().unwrap()]
                    .iter()
                    .chain(args.into_iter()),
            )
            .status()?
            .success();
        if !success {
            panic!("failed to install server")
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_install() {
        let root = env!("CARGO_MANIFEST_DIR");
        let test_dir = PathBuf::from(root).join("test");

        let test_loader_install = |loader: Loader| {
            let loader_test_dir = test_dir.join(loader.to_string());
            println!("testing {loader} in {loader_test_dir:?}...");
            if !loader_test_dir.exists() {
                println!("dir not exist, creating...");
                std::fs::create_dir_all(&loader_test_dir).unwrap();
            }

            println!("installing loader...");
            loader
                .install(&loader_test_dir, "1.20.4")
                .expect("failed to init server jar");
            println!("done.")
        };

        // test_loader_install(Loader::Fabric);
        test_loader_install(Loader::Quilt);
    }
}
