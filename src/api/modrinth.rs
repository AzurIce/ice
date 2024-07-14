use std::path::PathBuf;

use types::Version;

use crate::{loader::Loader, utils::download};

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

const HOST: &str = "https://api.modrinth.com/v2";

pub fn get_project_versions<S: AsRef<str>>(slug: S) -> Vec<Version> {
    let slug = slug.as_ref();
    let body = reqwest::blocking::get(format!("{HOST}/project/{slug}/version"))
        .unwrap()
        .json::<Vec<Version>>()
        .unwrap();
    return body;
}

mod types {
    use serde::Deserialize;

    use crate::loader::Loader;

    #[derive(Deserialize, Debug)]
    pub struct Project {}

    #[derive(Deserialize, Debug)]
    pub struct Version {
        pub name: String,
        pub version_number: String,
        pub game_versions: Vec<String>,
        pub loaders: Vec<Loader>,
        pub id: String,
        pub project_id: String,
        pub files: Vec<VersionFile>,
    }

    #[derive(Deserialize, Debug)]
    pub struct VersionFile {
        pub url: String,
        pub filename: String,
        pub size: i32,
    }
}

#[cfg(test)]
mod test {
    use super::get_project_versions;

    #[test]
    fn f() {
        let res = get_project_versions("fabric-api");
        println!("{:?}", res)
    }
}
