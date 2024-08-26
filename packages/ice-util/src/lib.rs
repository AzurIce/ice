pub mod fs;
pub mod path;
pub mod regex;
pub mod time;

use std::{io::Write, path::Path};

use futures_util::StreamExt;
use reqwest::Response;
use std::error::Error;

pub fn reqwest_get_blocking<S: AsRef<str>>(url: S) -> Result<Response, Box<dyn Error>> {
    let url = url.as_ref();
    let res = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(reqwest::get(url))?;
    Ok(res)
}

pub fn download_from_url_blocking<S: AsRef<str>, P: AsRef<Path>>(
    url: S,
    path: P,
    on_progress: impl Fn((u64, u64)),
) -> Result<(), Box<dyn Error>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(download_from_url(url, path, on_progress))
}

/// Download a file from a URL to a local path.
pub async fn download_from_url<S: AsRef<str>, P: AsRef<Path>>(
    url: S,
    path: P,
    on_progress: impl Fn((u64, u64)),
) -> Result<(), Box<dyn Error>> {
    let url = url.as_ref();
    let path = path.as_ref();

    let res = reqwest::get(url).await?;
    let total_bytes = res.content_length().unwrap();
    let mut downloaded_bytes = 0;

    let mut file = std::fs::File::create(path)?;

    let mut stream = res.bytes_stream();
    while let Some(bytes) = stream.next().await {
        match bytes {
            Ok(bytes) => {
                file.write_all(&bytes)?;
                downloaded_bytes += bytes.len() as u64;
                on_progress((downloaded_bytes, total_bytes));
            }
            Err(err) => return Err(err.into()),
        }
    }

    Ok(())
}
