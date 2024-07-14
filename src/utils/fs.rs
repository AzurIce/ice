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
