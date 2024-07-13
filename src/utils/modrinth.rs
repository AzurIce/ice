use std::path::PathBuf;

use crate::{
    core::{loader::Loader, modrinth::get_project_versions},
    download,
};

/// Download mod from modrinth to `mod_dir` by `slug`, `version_number` and `loader`
pub fn download_mod<S: AsRef<str>>(slug: S, version_number: S, loader: Loader, mod_dir: PathBuf) {
    let slug = slug.as_ref();
    let version_number = version_number.as_ref();
    print!("downloading mod {slug} = [{loader:?}] {version_number}...");

    let versions = get_project_versions(&slug);
    match versions.iter().find(|v| {
        // println!(
        //     "{:?}:{} {:?}:{}",
        //     v.version_number,
        //     v.version_number == version_number,
        //     v.loaders,
        //     v.loaders.contains(&loader)
        //         || loader == Loader::Quilt && v.loaders.contains(&Loader::Fabric)
        // );
        v.version_number == version_number
            && (v.loaders.contains(&loader)
                || loader == Loader::Quilt && v.loaders.contains(&Loader::Fabric))
    }) {
        Some(version) => {
            if version.files.len() > 1 {
                println!("has multiple files: {:?}, skipping...", version.files);
                return;
            }
            let version_file = version.files.first().unwrap();
            let local_path = mod_dir.join(&version_file.filename);

            if !local_path.exists() {
                match download(&version_file.url, local_path) {
                    Ok(_) => println!("finished"),
                    Err(err) => println!("failed: {err}, skipping..."),
                }
            } else {
                println!("already exists, skipping...");
            }
        }
        None => {
            println!("version not found, skipping...");
        }
    }
}
