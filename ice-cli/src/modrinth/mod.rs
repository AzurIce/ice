use types::Version;

#[cfg(test)]
mod test {
    use super::get_project_versions;

    #[test]
    fn f() {
        let res = get_project_versions("fabric-api");
        println!("{:?}", res)
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

    use crate::core::loader::Loader;

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