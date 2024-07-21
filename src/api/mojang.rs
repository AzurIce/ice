use std::error::Error;

use serde_json::Value;

/// Get the latest version number of the game
pub async fn get_latest_version() -> Result<String, Box<dyn Error>> {
    let res =
        reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json").await?;
    let json = res.json::<Value>().await?;
    let version = json
        .as_object()
        .unwrap()
        .get("latest")
        .unwrap()
        .as_object()
        .unwrap()
        .get("release")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    Ok(version)
}
