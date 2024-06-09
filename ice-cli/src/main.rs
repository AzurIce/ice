#![feature(fs_try_exists)]
use std::env;
use std::{fs, path::Path};

use ice_cli::config::Config;
use ice_cli::core::loader::Loader;
use ice_cli::core::Core;
use clap::Parser;
use log::{info, warn};
use regex::Regex;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
enum Cli {
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

fn main() {
    env_logger::init();

    match Cli::parse() {
        Cli::New {
            name,
            version,
            loader,
        } => {
            info!("creating directory...");
            let dir = Path::new("./").join(&name);
            fs::create_dir(&dir).expect(format!("failed to create directory [{}]", name).as_str());

            info!("version unspecified, fetching latest version...");
            // TODO: Get latest version
            // TODO: Add snapshot flag
            let version = version.unwrap_or("1.20.4".to_string());

            info!("writing config...");
            let bish_config = Config::new(name, version, loader);
            let bish_config = toml::to_string_pretty(&bish_config).expect("toml err");
            let bish_config_path = &dir.join("Bish.toml");
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
        Cli::Init { version, loader } => {
            let dir = env::current_dir().expect("failed to get current_dir");
            let name = dir.file_name().unwrap();
            let name = name.to_str().unwrap();
            info!("version unspecified, fetching latest version...");
            // TODO: Get latest version
            // TODO: Add snapshot flag
            let version = version.unwrap_or("1.20.4".to_string());

            info!("writing config...");
            let bish_config = Config::new(name.to_string(), version, loader);
            let bish_config = toml::to_string_pretty(&bish_config).expect("toml err");
            let bish_config_path = &dir.join("Bish.toml");
            fs::write(&bish_config_path, bish_config)
                .expect(format!("failed to write to [{:?}]", bish_config_path).as_str());

            info!("initializing dir...");
            // fs::create_dir(dir.join("server")).expect("failed to create directory");
            fs::create_dir(dir.join("mods")).expect("failed to create directory");
            fs::create_dir_all(dir.join("backups/snapshots")).expect("failed to create directory");
            fs::create_dir_all(dir.join("backups/archives")).expect("failed to create directory");
            fs::create_dir(dir.join("files")).expect("failed to create directory");
        }
        Cli::Run => {
            let dir = Path::new("./");
            info!("loading config...");
            let bish_config =
                fs::read_to_string(dir.join("Bish.toml")).expect("failed to read config");
            let bish_config =
                toml::from_str::<Config>(&bish_config).expect("failed to parse config");

            info!("checking server...");
            let server_dir = dir.join("server");
            if !fs::try_exists(&server_dir).expect("failed to try exist on server") {
                info!("server not exist, installing...");
                bish_config
                    .loader
                    .init_server_jar(&bish_config.version)
                    .expect("failed to init server jar");
            }

            info!("checking properties...");
            if fs::try_exists(&server_dir.join("server.properties"))
                .expect("failed to try exist on server")
            {
                info!("patching properties...");
                let property_file = server_dir.join("server.properties");
                let mut buf =
                    fs::read_to_string(&property_file).expect("failed to read server.properties");

                for (key, value) in &bish_config.properties {
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
            Core::run(bish_config)
            // run(bish_config);
        }
    }
}
