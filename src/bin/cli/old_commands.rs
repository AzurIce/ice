use std::{
    fs::{self, create_dir},
    path::Path,
};

use clap::Subcommand;
use ice::{config::Config, core::Core, loader::Loader};
use log::{info, warn};
use regex::Regex;

#[derive(Subcommand)]
pub(super) enum OldCommands {
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
}

impl OldCommands {
    pub fn exec<P: AsRef<Path>>(self, current_dir: P) {
        let current_dir = current_dir.as_ref();
        match self {
            OldCommands::New {
                name,
                version,
                loader,
            } => {
                info!("creating directory...");
                let dir = current_dir.join(&name);
                create_dir(&dir).unwrap();
                init_dir(dir, version, loader);
            }
            OldCommands::Init { version, loader } => {
                init_dir(current_dir, version, loader);
            }
            OldCommands::Run => {
                info!("loading config...");
                let ice_config = Config::load(current_dir.join("Ice.toml")).unwrap();

                info!("checking server...");
                let server_dir = current_dir.join("server");
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

pub fn init_dir<P: AsRef<Path>>(dir: P, version: Option<String>, loader: Loader) {
    let dir = dir.as_ref();
    let name = dir.file_name().unwrap();
    let name = name.to_str().unwrap();

    info!("version unspecified, fetching latest version...");
    // TODO: Get latest version
    // TODO: Add snapshot flag
    let version = version.unwrap_or("1.21".to_string());
    // TODO: verify version

    info!("writing config...");
    let bish_config = Config::new(name.to_string(), version, loader);
    let bish_config = toml::to_string_pretty(&bish_config).expect("toml err");
    let bish_config_path = &dir.join("Ice.toml");
    fs::write(&bish_config_path, bish_config)
        .expect(format!("failed to write to [{:?}]", bish_config_path).as_str());

    info!("initializing dir...");
    // fs::create_dir(dir.join("server")).expect("failed to create directory");
    fs::create_dir(dir.join("mods")).expect("failed to create directory");
    fs::create_dir_all(dir.join("backups/snapshots")).expect("failed to create directory");
    fs::create_dir_all(dir.join("backups/archives")).expect("failed to create directory");
    fs::create_dir(dir.join("files")).expect("failed to create directory");
    info!("done!")
}
