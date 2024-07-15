pub mod fs;
pub mod path;
pub mod regex;
pub mod time;

use std::{env, error::Error, path::Path};
use tracing::info;

pub fn download<P: AsRef<Path>>(url: &str, path: P) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();

    let mut client = reqwest::blocking::Client::builder();
    if let Ok(proxy) = env::var("HTTP_PROXY") {
        println!("using proxy: {:?}", proxy);
        client = client.proxy(reqwest::Proxy::http(proxy)?);
    }
    let client = client.build()?;

    let parent = path.parent().unwrap();
    if !parent.exists() {
        std::fs::create_dir_all(parent).unwrap();
    }
    if path.exists() {
        info!("File already exist, skipping download...");
    } else {
        info!("Downloading to {:?} from {}", path, url);
        let res = client.get(url).send()?;
        std::fs::write(path, res.bytes()?)?;
        info!("Downloaded!");
    }
    Ok(())
}
