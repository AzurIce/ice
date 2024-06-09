mod config;
mod core;
mod fabric;
mod server;
mod utils;
#[allow(unused)]
mod modrinth;

use core::Core;
use std::{error::Error, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use log::info;
use server::services::servers::{get_servers, start_server, stop_server};

// #[tokio::main]
async fn async_main(app_state: Arc<Core>) {
    let app = Router::new()
        .route("/servers", get(get_servers))
        .route("/servers/:name/start", post(start_server))
        .route("/servers/:name/stop", post(stop_server));
    let app = app.with_state(app_state.clone());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    info!("loading config...");
    let config = config::load_config().expect("Failed to load config");
    info!("initializing core...");
    let core = Core::init(config);
    let app_state = Arc::new(core);

    info!("initializing axum...");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main(app_state));
    Ok(())
}
