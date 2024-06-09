use std::{path::Path, error::Error, io::Write};

use curl::easy::Easy;
use log::info;

pub mod time {
    use chrono::Local;

    pub fn get_cur_time_str() -> String {
        Local::now().format("%Y-%m-%d %H-%M-%S").to_string()
    }
}

pub mod path {
    use std::path::Path;

    pub fn split_parent_and_file(path: String) -> (String, String) {
        let path = Path::new(&path);
        let parent_path = path.parent().unwrap().to_str().unwrap();
        let file_path = path.file_name().unwrap().to_str().unwrap();
        (parent_path.to_string(), file_path.to_string())
    }
}

pub fn download<P: AsRef<Path>>(url: &str, path: P) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();
    if !path.parent().unwrap().exists() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
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

#[allow(unused)]
pub mod fs {
    use std::{fs, io, path::Path};

    fn is_empty(path: &Path) -> bool {
        match fs::read_dir(path) {
            Ok(entries) => entries.count() == 0,
            Err(_) => true,
        }
    }

    fn clear_dir(path: &Path) {
        if !is_empty(path) {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_file() {
                    fs::remove_file(entry.path()).unwrap();
                }
            }
        }
    }

    pub fn copy_dir(src: &Path, dest: &Path) -> io::Result<()> {
        if !src.is_dir() || !src.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Source directory does not exist.",
            ));
        }

        if dest.exists() {
            clear_dir(dest)
        } else {
            fs::create_dir_all(dest)?;
        }

        let entries = fs::read_dir(src)?;

        for entry in entries {
            let entry = entry?;

            let entry_path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if entry_path.is_dir() {
                copy_dir(&entry_path, &dest_path)?;
            } else if entry.file_name().to_str() != Some("session.lock") {
                fs::copy(&entry_path, &dest_path)?;
            }
        }

        Ok(())
    }
}

pub mod regex {
    use std::sync::OnceLock;

    use regex::Regex;

    const FORWARD: &str = r"^(.+) *\| *(\S+?)\n";
    pub fn forward_regex() -> &'static Regex {
        static FORWARD_REGEX: OnceLock<Regex> = OnceLock::new();
        FORWARD_REGEX.get_or_init(|| Regex::new(FORWARD).expect("regex err"))
    }

    /*
    [16:00:01] [Server thread/INFO]: _AzurIce_[/127.0.0.1:58952] logged in with entity id 112 at (-21.5, 72.0, -7.5)
    [16:00:01] [Server thread/INFO]: _AzurIce_ joined the game
    [16:00:04] [Server thread/INFO]: <_AzurIce_> asd
    [16:00:06] [Server thread/INFO]: _AzurIce_ lost connection: Disconnected
    [16:00:06] [Server thread/INFO]: _AzurIce_ left the game
     */
    /*
    [19:23:48] [Server thread/INFO]: [Not Secure] <_AzurIce_> #bksnap make
     */
    const PLAYER: &str = r"]: (?:\[Not Secure] )?<(.*?)> (.*)";
    pub fn player_regex() -> &'static Regex {
        static PLAYER_REGEX: OnceLock<Regex> = OnceLock::new();
        PLAYER_REGEX.get_or_init(|| Regex::new(PLAYER).expect("regex err"))
    }
}
