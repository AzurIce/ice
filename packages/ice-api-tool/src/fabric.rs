use types::InstallerMeta;

pub async fn get_latest_installer_url() -> Result<String, anyhow::Error> {
    let res = reqwest::get("https://meta.fabricmc.net/v2/versions/installer")
        .await?
        .json::<Vec<InstallerMeta>>()
        .await?;
    res.first()
        .map(|meta| meta.url.clone())
        .ok_or(anyhow::anyhow!("failed to get latest fabric installer url"))
}

pub mod types {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct InstallerMeta {
        pub url: String,
        pub maven: String,
        pub version: String,
        pub stable: bool,
    }
}

#[cfg(test)]
mod test {
    use smol_macros::test;
    use macro_rules_attribute::apply;

    use super::*;

    use std::error::Error;

    #[apply(test!)]
    async fn test_get_latest_installer_url() -> Result<(), Box<dyn Error>> {
        let res = get_latest_installer_url().await;
        println!("{:?}", res);
        Ok(())
    }
}
