use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};
use toml::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub name: String,
    pub command: String,

    #[serde(default)]
    pub properties: HashMap<String, String>,
    #[serde(default)]
    pub plugins: HashMap<String, PluginConfig>,
}

pub type PluginConfig = HashMap<String, Value>;

impl Config {
    pub fn new(name: String) -> Self {
        Self {
            name,
            command: String::new(),
            properties: HashMap::new(),
            plugins: HashMap::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let config = fs::read_to_string(path)
            .map_err(|err| format!("failed to read config file: {:?}", err))?;
        let config = toml::from_str::<Config>(&config)
            .map_err(|err| format!("failed to parse config: {:?}", err))?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let config = toml::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize config: {:?}", err))?;
        fs::write(path, config).map_err(|err| format!("failed to write config file: {:?}", err))?;
        Ok(())
    }

    pub fn get_plugin_config(&self, name: &str) -> Option<&PluginConfig> {
        self.plugins.get(name)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_config() {
        let mut config = Config::new("name".to_string());
        let mut map: PluginConfig = HashMap::new();
        map.insert("interval".to_string(), Value::Integer(10));
        config.plugins.insert("scoreboard".to_string(), map);
        let toml = toml::to_string(&config).unwrap();
        println!("{toml}");
        // let toml = toml::to_string_pretty(&config).unwrap();
        // println!("{toml}");
        println!("{:?}", config.get_plugin_config("scoreboard"));
    }
}
