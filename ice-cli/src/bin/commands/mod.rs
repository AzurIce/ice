mod mod_sync;

use mod_sync::*;

use std::{
    env,
    fs::{self, create_dir},
    path::PathBuf,
};

use clap::Parser;
use ice_cli::{
    config::Config,
    core::{loader::Loader, Core},
};
use log::{info, warn};
use regex::Regex;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub enum Cli {
    New {
        name: String,

        #[arg(short, long)]
        version: Option<String>,

        #[arg(short, long, default_value_t = Loader::Quilt, value_enum)]
        loader: Loader,
    },
    Init {
        #[arg(short, long)]
        version: Option<String>,

        #[arg(short, long, default_value_t = Loader::Quilt, value_enum)]
        loader: Loader,
    },
    Run,
    ModSync,
}

impl Cli {
    pub fn exec(self) {
        let dir = env::current_dir().expect("failed to get current_dir");

        match self {
            Cli::New {
                name,
                version,
                loader,
            } => {
                info!("creating directory...");
                let dir = dir.join(&name);
                create_dir(&dir).unwrap();
                init_dir(dir, version, loader);
            }
            Cli::Init { version, loader } => {
                init_dir(dir, version, loader);
            }
            Cli::ModSync => mod_sync_blocking(dir),
            Cli::Run => {
                info!("loading config...");
                let ice_config = Config::load(dir.join("Ice.toml")).unwrap();

                info!("checking server...");
                let server_dir = dir.join("server");
                if !server_dir.exists() {
                    info!("server not exist, installing...");
                    ice_config
                        .loader
                        .init_server_jar(&ice_config.version)
                        .expect("failed to init server jar");
                }

                info!("checking properties...");
                if server_dir.join("server.properties").exists() {
                    info!("patching properties...");
                    let property_file = server_dir.join("server.properties");
                    let mut buf = fs::read_to_string(&property_file)
                        .expect("failed to read server.properties");

                    for (key, value) in &ice_config.properties {
                        info!("setting property [{}] to [{}]", key, value);
                        let regex = Regex::new(format!(r"{}=([^#\n\r]*)", key).as_str()).unwrap();
                        buf = regex
                            .replace(&buf, format!("{}={}", key, value))
                            .to_string();
                    }
                    fs::write(property_file, buf.as_bytes())
                        .expect("failed to write server.properties: {:?}");
                } else {
                    warn!("server.properties not found, cannot patch, skipping...")
                }

                info!("checking mods...");
                // TODO: check mods

                info!("the core is running...");
                Core::run(ice_config)
                // run(bish_config);
            }
        }
    }
}

pub fn init_dir(dir: PathBuf, version: Option<String>, loader: Loader) {
    let name = dir.file_name().unwrap();
    let name = name.to_str().unwrap();

    info!("version unspecified, fetching latest version...");
    // TODO: Get latest version
    // TODO: Add snapshot flag
    let version = version.unwrap_or("1.21".to_string());
    // TODO: verify version

    info!("writing config...");
    let ice_config = Config::new(name.to_string(), version, loader);
    let ice_config = toml::to_string_pretty(&ice_config).expect("toml err");
    let ice_config_path = &dir.join("Ice.toml");
    fs::write(&ice_config_path, ice_config)
        .expect(format!("failed to write to [{:?}]", ice_config_path).as_str());

    info!("initializing dir...");
    // fs::create_dir(dir.join("server")).expect("failed to create directory");
    fs::create_dir(dir.join("mods")).expect("failed to create directory");
    fs::create_dir_all(dir.join("backups/snapshots")).expect("failed to create directory");
    fs::create_dir_all(dir.join("backups/archives")).expect("failed to create directory");
    fs::create_dir(dir.join("files")).expect("failed to create directory");
    info!("done!");
}
