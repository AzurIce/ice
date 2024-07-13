pub mod mod_config;

use std::{fs, path::Path, collections::HashMap};
use serde::{Serialize, Deserialize};
use crate::core::loader::Loader;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub loader: Loader,
    #[serde(default)]
    pub jvm_options: String,

    #[serde(default)]
    pub properties: HashMap<String, String>,
    #[serde(default)]
    pub mods: HashMap<String, String> // slug -> version_number
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(untagged)]
// pub enum ModInfo {
//     SpecificVersion { version_id: String },
// }

impl Config {
    pub fn new(name: String, version: String, loader: Loader) -> Self {
        Self { name, version, loader, jvm_options: String::new(), properties: HashMap::new(), mods: HashMap::new() }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let config = fs::read_to_string(path).map_err(|err| format!("failed to read config file: {:?}", err))?;
        let config = toml::from_str::<Config>(&config).map_err(|err| format!("failed to parse config: {:?}", err))?;
        // TODO: check server version
        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_config() {
        let config = Config::new("name".to_string(), "version".to_string(), Loader::Quilt);
        let toml = toml::to_string(&config).unwrap();
        println!("{toml}");
        let toml = toml::to_string_pretty(&config).unwrap();
        println!("{toml}");
    }
}