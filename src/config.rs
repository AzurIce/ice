use std::{collections::HashMap, fs, path::Path};

use ice_core::Loader;
use serde::{Deserialize, Serialize};
use toml_edit::{value, DocumentMut};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModConfig {
    pub version: String,
    pub loader: Loader,
    #[serde(default)]
    pub mods: HashMap<String, String>, // slug -> version_number
    #[serde(skip)]
    inner: Option<DocumentMut>
}

impl ModConfig {
    pub fn new(version: String, loader: Loader) -> Self {
        Self {
            version,
            loader,
            mods: HashMap::new(),
            inner: None,
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        let document = s.parse::<DocumentMut>().unwrap();
        let mut config = toml::from_str::<ModConfig>(s)
            .map_err(|err| format!("failed to parse config: {}", err))?;
        config.inner = Some(document);
        Ok(config)
    }

    pub fn to_string(&self) -> String {
        let config = toml::to_string_pretty(self).unwrap();
        let config = self.inner.as_ref().map(|d| d.to_string()).unwrap_or(config);
        return config;
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let config = fs::read_to_string(path)
            .map_err(|err| format!("failed to read config file: {}", err))?;
        let config = Self::from_str(&config)?;
        // TODO: check server version
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let config = self.to_string();
        fs::write(path, config).map_err(|err| format!("failed to write config file: {}", err))?;
        Ok(())
    }
}

impl ModConfig {
    pub fn insert_mod(&mut self, slug: String, version: String) {
        self.mods.insert(slug.clone(), version.clone());
        if let Some(document) = &mut self.inner {
            if let Some(item) = document["mods"][&slug].as_value_mut() {
                let decor = item.decor();
                let prefix = decor.prefix().map(|s| s.as_str().unwrap()).unwrap_or("");
                let suffix = decor.suffix().map(|s| s.as_str().unwrap()).unwrap_or("");
                *item = value(version).into_value().unwrap().decorated(prefix, suffix);
            } else {
                document["mods"][&slug] = value(version);
            }
        }
    }
}


#[cfg(test)]
mod test {
    use std::path::Path;

    use super::ModConfig;

    #[test]
    fn foo() {
        let mut config = ModConfig::load(Path::new(r#"G:\_MCServer\1.20.1 Survival\Ice.toml"#)).unwrap();
        config.insert_mod("ashdajsdhasdk".to_string(), "asdhfjkladhsfjkl".to_string());
        println!("{:?}", config.to_string())
    }
}