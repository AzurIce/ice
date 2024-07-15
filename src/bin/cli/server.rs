use std::{
    fs::{self, create_dir},
    path::Path,
};

use clap::Subcommand;
use ice::{api::mojang::get_latest_version, config::Config, core::Core, loader::Loader};
use regex::Regex;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Subcommand)]
pub(super) enum ServerCommands {
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
    Install,
    Run,
}

impl ServerCommands {
    pub fn exec<P: AsRef<Path>>(self, current_dir: P) {
        let current_dir = current_dir.as_ref();
        match self {
            Self::New {
                name,
                version,
                loader,
            } => {
                let version = version.unwrap_or(get_latest_version().unwrap());
                let dir = current_dir.join(&name);
                create_dir(&dir).unwrap();
                init_dir(dir, version, loader);
            }
            Self::Init { version, loader } => {
                let version = version.unwrap_or(get_latest_version().unwrap());
                init_dir(current_dir, &version, loader);
            }
            Self::Install => {
                let config = Config::load(current_dir.join("Ice.toml")).unwrap();
                if config.loader.installed(current_dir) {
                    println!("already intalled")
                } else {
                    config
                        .loader
                        .install(current_dir, config.version.clone())
                        .expect("failed to install");
                }
            }
            Self::Run => {
                // a builder for `FmtSubscriber`.
                let subscriber = FmtSubscriber::builder()
                    // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
                    // will be written to stdout.
                    .with_max_level(Level::TRACE)
                    // completes the builder.
                    .finish();

                tracing::subscriber::set_global_default(subscriber)
                    .expect("setting default subscriber failed");

                let config = Config::load(current_dir.join("Ice.toml")).unwrap();
                if !config.loader.installed(current_dir) {
                    println!("server not installed, use `ice server install` to install server");
                    return;
                }

                info!("checking properties...");
                let server_dir = current_dir.join("server");
                if server_dir.join("server.properties").exists() {
                    info!("patching properties...");
                    let property_file = server_dir.join("server.properties");
                    let mut buf = fs::read_to_string(&property_file)
                        .expect("failed to read server.properties");

                    for (key, value) in &config.properties {
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
                Core::run(config)
            }
        }
    }
}

pub fn init_dir<P: AsRef<Path>, S: AsRef<str>>(dir: P, version: S, loader: Loader) {
    let dir = dir.as_ref();
    let version = version.as_ref();
    let name = dir.file_name().unwrap();
    let name = name.to_str().unwrap();

    info!("writing config...");
    let bish_config = Config::new(name.to_string(), version.to_string(), loader);
    let bish_config = toml::to_string_pretty(&bish_config).expect("toml err");
    let bish_config_path = &dir.join("Ice.toml");
    fs::write(bish_config_path, bish_config)
        .map_err(|err| format!("failed to write to [{:?}]: {err}", bish_config_path))
        .unwrap();

    // info!("initializing dir...");
    // fs::create_dir(dir.join("server")).expect("failed to create directory");
    // fs::create_dir(dir.join("mods")).expect("failed to create directory");
    // fs::create_dir_all(dir.join("backups/snapshots")).expect("failed to create directory");
    // fs::create_dir_all(dir.join("backups/archives")).expect("failed to create directory");
    // fs::create_dir(dir.join("files")).expect("failed to create directory");
    info!("done!")
}
