use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use log::info;
use reqwest::StatusCode;
use serde::Serialize;
use serde_json::json;

use crate::{config::ServerConfig, core::Core};

#[derive(Serialize)]
struct ServerInfo {
    pub name: String,
    pub config: ServerConfig,
    pub running: bool,
}

pub async fn get_servers(State(state): State<Arc<Core>>) -> (StatusCode, Json<serde_json::Value>) {
    let mut res = vec![];
    let servers = state.servers.lock().unwrap();
    for (name, server) in servers.iter() {
        let server = server.lock().unwrap();
        res.push(ServerInfo {
            name: name.clone(),
            config: server.config.clone(),
            running: server.running,
        })
    }

    (
        StatusCode::OK,
        Json(json!({
            "servers": res
        })),
    )
}

pub async fn start_server(
    State(state): State<Arc<Core>>,
    Path(name): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    info!("start_server: {}", name);
    let res = tokio::task::spawn_blocking(move || state.run_server(name)).await.expect("failed to exec");
    match res {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "msg": "success"
            })),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "msg": err
            })),
        ),
    }
}

pub async fn stop_server(
    State(state): State<Arc<Core>>,
    Path(name): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    if state.servers.lock().unwrap().get(&name).is_some() {
        state.stop_server(name);
        (
            StatusCode::OK,
            Json(json!({
                "msg": "success"
            })),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "msg": "The server is not running"
            })),
        )
    }
}
