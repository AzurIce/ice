use ice_core::Loader;
use serde::{Deserialize, Serialize};
use toml::Value;
use std::{collections::HashMap, fs, path::Path};

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
    pub mods: HashMap<String, String>, // slug -> version_number
    #[serde(default)]
    pub plugins: HashMap<String, Value>,
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(untagged)]
// pub enum ModInfo {
//     SpecificVersion { version_id: String },
// }

impl Config {
    pub fn new(name: String, version: String, loader: Loader) -> Self {
        Self {
            name,
            version,
            loader,
            jvm_options: String::new(),
            properties: HashMap::new(),
            mods: HashMap::new(),
            plugins: HashMap::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let config = fs::read_to_string(path)
            .map_err(|err| format!("failed to read config file: {:?}", err))?;
        let config = toml::from_str::<Config>(&config)
            .map_err(|err| format!("failed to parse config: {:?}", err))?;
        // TODO: check server version
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let config = toml::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize config: {:?}", err))?;
        fs::write(path, config).map_err(|err| format!("failed to write config file: {:?}", err))?;
        Ok(())
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
