use anyhow::Context;

pub async fn get_latest_installer_url() -> Result<String, anyhow::Error> {
    reqwest::get("https://quiltmc.org/api/v1/download-latest-installer/java-universal")
        .await
        .context("failed to get latest quilt installer url")
        .map(|x| x.url().as_str().to_string())
}
