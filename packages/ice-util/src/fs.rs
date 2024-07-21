use std::{fs, io, path::Path};

use sha::{
    sha1::Sha1,
    sha512::Sha512,
    utils::{Digest, DigestExt},
};

/// get sha1 hash of a file
pub fn get_sha1_hash<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let bytes = fs::read(path)?;
    let res = Sha1::default().digest(&bytes).to_hex();
    Ok(res)
}

/// get sha512 hash of a file
pub fn get_sha512_hash<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let bytes = fs::read(path)?;
    let res = Sha512::default().digest(&bytes).to_hex();
    Ok(res)
}

fn is_empty<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    match fs::read_dir(path) {
        Ok(entries) => entries.count() == 0,
        Err(_) => true,
    }
}

fn clear_dir<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    if !is_empty(path) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                fs::remove_file(entry.path()).unwrap();
            }
        }
    }
}

pub fn copy_dir<P: AsRef<Path>>(src: P, dst: P) -> io::Result<()> {
    let (src, dst) = (src.as_ref(), dst.as_ref());
    if !src.is_dir() || !src.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Source directory does not exist.",
        ));
    }

    if dst.exists() {
        clear_dir(dst)
    } else {
        fs::create_dir_all(dst)?;
    }

    let entries = fs::read_dir(src)?;

    for entry in entries {
        let entry = entry?;

        let entry_path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir(&entry_path, &dest_path)?;
        } else if entry.file_name().to_str() != Some("session.lock") {
            fs::copy(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}
