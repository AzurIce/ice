pub mod fs;
pub mod minecraft;
pub mod path;
pub mod regex;
pub mod time;

use std::path::Path;

use anyhow::Context;
use futures_util::StreamExt;
use smol::io::AsyncWriteExt;

pub fn get_url_filename(url: &str) -> Option<&str> {
    url.split("/").last()
}

pub fn get_parent_version(version: String) -> String {
    let x = version.rfind(".");
    if let Some(x) = x {
        version[..x].to_string()
    } else {
        version
    }
}

/// Download a file from a URL to a local path.
pub async fn download_from_url<S: AsRef<str>, P: AsRef<Path>>(
    url: S,
    path: P,
    on_progress: impl Fn((u64, u64)),
) -> Result<(), anyhow::Error> {
    let url = url.as_ref();
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).context("failed to create dir")?;
        }
    }

    let res = reqwest::get(url).await?;
    let total_bytes = res.content_length().unwrap();
    let mut downloaded_bytes = 0;

    let mut file = smol::fs::File::create(path).await?;

    let mut stream = res.bytes_stream();
    while let Some(bytes) = stream.next().await {
        match bytes {
            Ok(bytes) => {
                file.write_all(&bytes).await?;
                downloaded_bytes += bytes.len() as u64;
                on_progress((downloaded_bytes, total_bytes));
            }
            Err(err) => return Err(err.into()),
        }
    }

    file.sync_all().await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::{io::Read, path::PathBuf};

    use sha::sha1::Sha1;

    use crate::{download_from_url, fs::get_sha1_hash};

    #[tokio::test]
    async fn test_doawnload_from_url() {
        let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test");
        std::fs::create_dir_all(&test_dir).unwrap();

        let url = "https://cdn.modrinth.com/data/P7dR8mSH/versions/F5TVHWcE/fabric-api-0.128.2%2B1.21.6.jar";

        let path = test_dir.join("1.jar");
        download_from_url(url, &path, |_| ()).await.unwrap();
        let sha_1 = get_sha1_hash(path).unwrap();

        let path = test_dir.join("2.jar");
        download_from_url(url, &path, |_| ()).await.unwrap();
        let sha_2 = get_sha1_hash(path).unwrap();

        assert_eq!(sha_1, sha_2);
    }
}
