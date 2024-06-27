use std::path::PathBuf;

use ice_cli::{
    core::{loader::Loader, modrinth::get_project_versions},
    download,
};

pub fn download_mod<S: AsRef<str>>(slug: S, version_number: S, loader: Loader, mod_dir: PathBuf) {
    let slug = slug.as_ref();
    let version_number = version_number.as_ref();
    println!("downloading mod {slug} = {version_number}...");

    let versions = get_project_versions(&slug);
    match versions
        .iter()
        .find(|v| v.version_number == version_number && v.loaders.contains(&loader))
    {
        Some(version) => {
            if version.files.len() > 1 {
                println!("has multiple files: {:?}, skipping...", version.files);
                return;
            }
            let version_file = version.files.first().unwrap();
            let local_path = mod_dir.join(&version_file.filename);

            if !local_path.exists() {
                match download(&version_file.url, local_path) {
                    Ok(_) => println!("downloaded {slug} = {version_number}"),
                    Err(err) => println!("failed to download {slug} = {version_number}: {err}"),
                }
            } else {
                println!("{slug} = {version_number} already exists, skipping...");
            }
        }
        None => {
            println!("version not found for {slug} = {version_number}, skipping...");
        }
    }
}
