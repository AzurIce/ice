pub mod fs;
pub mod path;
pub mod time;
pub mod regex;

use std::{error::Error, path::Path};
use tracing::info;

pub fn download<P: AsRef<Path>>(url: &str, path: P) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();
    if !path.parent().unwrap().exists() {
        std::fs::create_dir_all(path).unwrap();
    }
    if path.exists() {
        info!("File already exist, skipping download...");
    } else {
        info!("Downloading to {:?} from {}", path, url);
        let res = reqwest::blocking::get(url)?;
        std::fs::write(path, res.bytes()?)?;
        info!("Downloaded!");
    }
    Ok(())
}
