use std::{error::Error, fs, path::Path};

use log::info;
use serde::Deserialize;

use crate::utils::download;

const MODRINTH: &str = "https://api.modrinth.com/v2";

#[cfg(test)]
mod test {
    use super::{get_project_versions, File, Project, init_mod};

    #[test]
    fn test_get_project() {
        let project = Project::get("fabric-api");
        println!("{:?}", project);
    }

    #[test]
    fn test_get_project_versions() {
        let res = get_project_versions("fabric-api", "1.20.4").unwrap();
        // println!("{:#?}", res);
        let version_primary_files = res
            .into_iter()
            .map(|version| version.get_primary_file())
            .collect::<Vec<File>>();
        println!("{:?}", version_primary_files);
    }

    #[test]
    fn test_init_mod() {
        init_mod("fabric-api", "1.20.4", "./").expect("failed to init mod");
    }
}

#[derive(Deserialize, Debug)]
pub struct Project {
    id: String,
    slug: String,
    client_side: String,
    server_side: String,
    game_versions: Vec<String>,
}

impl Project {
    pub fn get(slug: &str) -> Result<Self, Box<dyn Error>> {
        let res = reqwest::blocking::get(format!("{MODRINTH}/project/{slug}"))?;
        let res = res.json::<Self>()?;
        Ok(res)
    }
}

pub fn get_project_versions(slug: &str, version: &str) -> Result<Vec<Version>, Box<dyn Error>> {
    let url = format!("{MODRINTH}/project/{slug}/version");
    let params = [
        ("loaders", r#"["fabric"]"#.to_string()),
        ("game_versions", format!(r#"["{version}"]"#)),
    ];
    let url = reqwest::Url::parse_with_params(&url, params)?;
    let res = reqwest::blocking::get(url)?;
    let res = res.json::<Vec<Version>>()?;
    Ok(res)
}

#[derive(Deserialize, Debug)]
pub struct Version {
    name: String,
    version_number: String,
    game_versions: Vec<String>,
    version_type: String,
    id: String,
    project_id: String,
    date_published: String,
    files: Vec<File>,
}

impl Version {
    pub fn get_primary_file(&self) -> File {
        for file in &self.files {
            if file.primary == true {
                return file.clone();
            }
        }
        return self.files.first().unwrap().clone();
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct File {
    url: String,
    filename: String,
    primary: bool,
    size: usize,
}

pub fn init_mod<P: AsRef<Path>>(slug: &str, version: &str, dir: P) -> Result<(), String> {
    let dir = dir.as_ref();
    // let project = Project::get(slug).map_err(|err| format!("failed to get project: {:?}", err))?;
    let versions = get_project_versions(slug, version)
        .map_err(|err| format!("failed to get project versions: {:?}", err))?;

    let version_files = versions
        .into_iter()
        .map(|version| version.get_primary_file())
        .collect::<Vec<File>>();
    let latest_version_files = version_files.first().unwrap();

    let entries = fs::read_dir(dir).map_err(|err| format!("failed to read mod dir: {:?}", err))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read entry: {:?}", err))?;
        let metadata = entry
            .metadata()
            .map_err(|err| format!("failed to read entry metadata: {:?}", err))?;
        if metadata.is_file() {
            if latest_version_files.filename.as_str() == entry.file_name() {
                info!("latest version of mod {} already exist", slug);
                return Ok(());
            }
            for version_file in &version_files[1..] {
                if version_file.filename.as_str() == entry.file_name() {
                    info!("found old version of mod {}, removing...", slug);
                    fs::remove_file(entry.path())
                        .map_err(|err| format!("failed to remove old version file: {:?}", err))?;
                }
            }
        }
    }

    download(
        &latest_version_files.url,
        dir.join(&latest_version_files.filename),
    )
    .map_err(|err| format!("failed to download file: {:?}", err))?;

    Ok(())
}
