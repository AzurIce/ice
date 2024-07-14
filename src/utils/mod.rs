pub mod fs;
pub mod path;
pub mod time;

use curl::easy::Easy;
use std::{error::Error, io::Write, path::Path};
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
        let mut f = std::fs::File::create(path)?;
        let mut easy = Easy::new();
        easy.url(url).unwrap();
        easy.follow_location(true).unwrap();
        easy.write_function(move |data| {
            f.write_all(data).unwrap();
            Ok(data.len())
        })
        .unwrap();
        easy.perform().unwrap();
        info!("Downloaded!");
    }
    Ok(())
}
