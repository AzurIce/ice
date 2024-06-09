use std::{collections::HashMap, error::Error, fs, io};

use derivative::Derivative;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        let config = BishConfig::default();
        let str = toml::to_string_pretty(&config).expect("");
        println!("{}", str)
    }

    #[test]
    fn test_read_config() -> Result<(), Box<dyn Error>> {
        load_config()?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Derivative, Clone)]
#[derivative(Default)]
pub struct BishConfig {
    #[derivative(Default(value = r##"String::from("#")"##))]
    pub command_prefix: String,
    #[derivative(Default(value = r##"String::from("./backups")"##))]
    pub backup_dir: String,
    pub servers: HashMap<String, ServerConfig>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ServerConfig {
    pub dir: String,
    pub jvm_options: String,
    pub version: String,
    pub properties: HashMap<String, String>,
    pub mods: HashMap<String, ModInfo>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ModInfo {
    SpecificVersion { version_id: String },
}

pub fn load_config() -> Result<BishConfig, Box<dyn Error>> {
    println!("[load_config]: reading config...");
    let config = fs::read_to_string("./config.toml").or_else(|err| {
        if let io::ErrorKind::NotFound = err.kind() {
            println!("[load_config]: config.toml not found, writing default config...");
            let default_config = BishConfig::default();
            let default_config = toml::to_string_pretty(&default_config).unwrap();
            fs::write("./config.toml", &default_config).unwrap();
            Ok(default_config)
        } else {
            Err(err)
        }
    })?;
    println!("[load_config]: parsing config...");
    let config = toml::from_str::<BishConfig>(&config)?;
    println!("{:#?}", config);

    Ok(config)
}
