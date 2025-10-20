pub async fn get_latest_installer_url(game_version: &str) -> Result<String, anyhow::Error> {
    use reqwest::Url;

    const MAIN_ENDPOINT: &str =
        "https://maven.neoforged.net/api/maven/latest/version/releases/net/neoforged/neoforge";
    const FALLBACK_ENDPOINT: &str =
        "https://maven.creeperhost.net/api/maven/latest/version/releases/net/neoforged/neoforge";
    const DOWNLOAD_URL: &str = "https://maven.neoforged.net/releases";
    const NEOFORGE_GAV: &str = "net/neoforged/neoforge";

    // Get latest NeoForge version for given MC version (filter=MC_MAJOR.MINOR)
    let mc_version_prefix = {
        let splits: Vec<&str> = game_version.split('.').collect();
        if splits.len() < 2 {
            return Err(anyhow::anyhow!(
                "invalid minecraft version: {}",
                game_version
            ));
        }
        format!("{}.{}.", splits[1], splits[2])
    };

    // Build request with filter param
    let mut url = Url::parse(MAIN_ENDPOINT)?;
    url.query_pairs_mut()
        .append_pair("filter", &mc_version_prefix);
    // println!("{url:?}");

    // Define response type
    #[derive(Debug, serde::Deserialize)]
    struct LatestResp {
        version: Option<String>,
    }

    // Fallback logic: try main endpoint, else fallback endpoint.
    let version = match reqwest::get(url.clone()).await {
        Ok(resp) => {
            let json: LatestResp = resp.json().await?;
            json.version
        }
        Err(_) => {
            // fallback
            let mut fallback_url = Url::parse(FALLBACK_ENDPOINT)?;
            fallback_url
                .query_pairs_mut()
                .append_pair("filter", &mc_version_prefix);
            let res = reqwest::get(fallback_url)
                .await
                .map_err(|e| anyhow::anyhow!("NeoForge main & fallback endpoints failed: {e}"))?;
            let json: LatestResp = res.json().await?;
            json.version
        }
    }
    .ok_or(anyhow::anyhow!(
        "No NeoForge version found for MC version {game_version}"
    ))?;

    // println!("{:?}", version);

    // https://maven.neoforged.net/releases/net/neoforged/neoforge/[VERSION]/neoforge-[VERSION]-installer.jar
    let url = format!("{DOWNLOAD_URL}/{NEOFORGE_GAV}/{version}/neoforge-{version}-installer.jar",);

    Ok(url)
}

#[cfg(test)]
mod test {
    use super::*;
    use async_compat::Compat;
    use macro_rules_attribute::apply;
    use smol_macros::test;

    #[apply(test!)]
    async fn test_get_latest_installer_url() {
        let url = Compat::new(get_latest_installer_url("1.21.1"))
            .await
            .unwrap();
        println!("{url}");
    }
}
