use std::path::Path;

use async_compat::Compat;
use ice_api_tool::mojang::get_latest_version;
use ice_core::ServerLoader;
use ice_server::{Core, config::Config};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

pub async fn new(name: impl AsRef<str>, current_dir: impl AsRef<Path>) {
    let name = name.as_ref();
    let current_dir = current_dir.as_ref();

    let dir = current_dir.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    init_dir(dir);
}

pub async fn init(current_dir: impl AsRef<Path>) {
    init_dir(current_dir);
}

/// A cli tool to install server with loader
///
/// Has nothing to do with Ice.toml
pub fn install(current_dir: impl AsRef<Path>, game_version: Option<String>, loader: ServerLoader) {
    let current_dir = current_dir.as_ref();
    let server_dir = current_dir.join("server");

    let game_version = game_version.unwrap_or(
        smol::block_on(Compat::new(get_latest_version()))
            .expect("failed to get latest game version"),
    );

    if server_dir.exists() {
        println!(
            "./server dir is not empty, the content will be override, are you sure you want to install the server? ([y]/n)"
        );
        use std::io::{self, Write};

        let mut input = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();
        if !input.is_empty() && input != "y" {
            println!("aborted.");
            return;
        }
    }

    loader
        .install(current_dir, &game_version)
        .expect("failed to install");
}

pub async fn run(current_dir: impl AsRef<Path>) {
    let current_dir = current_dir.as_ref();
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        .with_env_filter("ice=INFO")
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = Config::load(current_dir.join("Ice.toml")).unwrap();
    info!("the core is running...");
    Core::run(config, current_dir).await;
}

pub fn init_dir(dir: impl AsRef<Path>) {
    let dir = dir.as_ref();
    let name = dir.file_name().unwrap();
    let name = name.to_str().unwrap();

    info!("writing config...");
    let config = Config::new(name.to_string());
    let config = toml::to_string_pretty(&config).expect("toml err");
    let config_path = &dir.join("Ice.toml");
    std::fs::write(config_path, config)
        .map_err(|err| format!("failed to write to [{:?}]: {err}", config_path))
        .unwrap();

    // info!("initializing dir...");
    // fs::create_dir(dir.join("server")).expect("failed to create directory");
    // fs::create_dir(dir.join("mods")).expect("failed to create directory");
    // fs::create_dir_all(dir.join("backups/snapshots")).expect("failed to create directory");
    // fs::create_dir_all(dir.join("backups/archives")).expect("failed to create directory");
    // fs::create_dir(dir.join("files")).expect("failed to create directory");
    info!("done!")
}
