pub mod modrinth;

use serde_json::Value;

/// Get the latest version number of the game
pub fn get_latest_version() -> Result<String, String> {
    let res =
        reqwest::blocking::get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
            .map_err(|err| format!("failed to get version_manifest_v2.json: {err}"))?
            .json::<Value>()
            .map_err(|err| format!("failed to parse version_manifest_v2.json: {err}"))?;
    let version = res
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
