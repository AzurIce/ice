use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::core::loader::Loader;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModConfig {
    pub version: String,
    pub loader: Loader,
    #[serde(default)]
    pub mods: HashMap<String, String>, // slug -> version_number
}

impl ModConfig {
    pub fn new(version: String, loader: Loader) -> Self {
        Self {
            version,
            loader,
            mods: HashMap::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let config = fs::read_to_string(path)
            .map_err(|err| format!("failed to read config file: {}", err))?;
        let config = toml::from_str::<ModConfig>(&config)
            .map_err(|err| format!("failed to parse config: {}", err))?;
        // TODO: check server version
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let config = toml::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize config: {}", err))?;
        fs::write(path, config).map_err(|err| format!("failed to write config file: {}", err))?;
        Ok(())
    }
}
