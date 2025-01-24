use types::InstallerMeta;

pub fn get_latest_installer_url() -> Result<String, anyhow::Error> {
    let res = reqwest::blocking::get("https://meta.fabricmc.net/v2/versions/installer")?
        .json::<Vec<InstallerMeta>>()?;
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
    use super::*;
    use crate::fabric::types::*;

    use std::error::Error;

    #[test]
    fn test_get_latest_installer_url() -> Result<(), Box<dyn Error>> {
        let res = get_latest_installer_url();
        println!("{:?}", res);
        Ok(())
    }
}
