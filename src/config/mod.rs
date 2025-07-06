use std::{
    collections::BTreeMap,
    fmt::Display,
    fs,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    str::FromStr,
};

use ice_core::Loader;
use serde::{Deserialize, Serialize};

use crate::core::{Mod, ModrinthMod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomlModsManifest {
    pub version: String,
    pub loader: Loader,
    pub mods: BTreeMap<String, TomlMod>,

    #[serde(skip_serializing)]
    #[serde(flatten)]
    pub _unused_keys: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TomlMod {
    Simple(String),
    Detailed(TomlDetailedMod),
}

impl From<(String, TomlMod)> for Mod {
    fn from((slug, value): (String, TomlMod)) -> Self {
        match value {
            TomlMod::Simple(version_id_number) => Mod::Modrinth(ModrinthMod {
                slug,
                version_id_number,
            }),
            TomlMod::Detailed(detailed) => {
                if let Some(version_id_number) = detailed.version_id_number {
                    Mod::Modrinth(ModrinthMod {
                        slug,
                        version_id_number,
                    })
                } else {
                    Mod::Unknown
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TomlDetailedMod {
    pub version_id_number: Option<String>,
    // pub strict_match_game_version: Option<bool>,
    pub url: Option<String>,

    #[serde(skip_serializing)]
    #[serde(flatten)]
    pub _unused_keys: BTreeMap<String, toml::Value>,
}

/// A wrapper around the toml config data in order to
/// support format preserving edits while still
/// providing easy access to the data.
#[derive(Debug, Clone)]
pub struct ModsConfig {
    config: TomlModsManifest,
    document: toml_edit::DocumentMut,
}

impl ModsConfig {
    pub fn new(version: String, loader: Loader) -> Self {
        let mut document = toml_edit::DocumentMut::new();
        document["version"] = toml_edit::value(version.clone());
        document["loader"] = toml_edit::value(loader.to_string());
        document["mods"] = toml_edit::Item::Table(toml_edit::Table::default());

        let config = TomlModsManifest {
            version,
            loader,
            mods: BTreeMap::new(),
            _unused_keys: BTreeMap::new(),
        };

        Self { config, document }
    }

    pub fn get_mod(&self, key: &str) -> Option<Mod> {
        self.mods
            .get(key)
            .map(|v| (key.to_string(), v.clone()).into())
    }

    pub fn get_mods(&self) -> Vec<Mod> {
        self.mods
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()).into())
            .collect()
    }

    pub fn insert_mod(&mut self, value: Mod) {
        match value {
            Mod::Modrinth(modrinth) => {
                let toml_mod = TomlMod::Simple(modrinth.version_id_number.clone());
                self.config.mods.insert(modrinth.slug.clone(), toml_mod);

                let document = &mut self.document;
                if let Some(item) = document["mods"][&modrinth.slug].as_value_mut() {
                    let decor = item.decor();
                    let prefix = decor.prefix().map(|s| s.as_str().unwrap()).unwrap_or("");
                    let suffix = decor.suffix().map(|s| s.as_str().unwrap()).unwrap_or("");
                    *item = toml_edit::value(modrinth.version_id_number)
                        .into_value()
                        .unwrap()
                        .decorated(prefix, suffix);
                } else {
                    document["mods"][&modrinth.slug] = toml_edit::value(modrinth.version_id_number);
                }
            }
            _ => {}
        }
    }
}

/// Dereference to the inner `TomlManifest` struct.
///
/// This allows for easy access to the data
impl Deref for ModsConfig {
    type Target = TomlModsManifest;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl FromStr for ModsConfig {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config = toml::from_str::<TomlModsManifest>(s)?;
        let document = s.parse()?;
        Ok(ModsConfig { config, document })
    }
}

impl Display for ModsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.document.fmt(f)
    }
}

/// A wrapper around the mods config that also holds the path to the file
/// on the local filesystem.
#[derive(Debug, Clone)]
pub struct LocalModsConfig {
    inner: ModsConfig,
    path: PathBuf,
}

impl Deref for LocalModsConfig {
    type Target = ModsConfig;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for LocalModsConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl LocalModsConfig {
    pub fn new(version: String, loader: Loader, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let inner = ModsConfig::new(version, loader);
        LocalModsConfig { inner, path }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let path = path.as_ref().to_path_buf();
        let inner = fs::read_to_string(&path)?;
        let inner = ModsConfig::from_str(&inner)?;
        Ok(LocalModsConfig { inner, path })
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let config = self.inner.to_string();
        fs::write(self.path.clone(), config)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::core::ModrinthMod;

    use super::*;

    #[test]
    fn foo() {
        let mut config =
            LocalModsConfig::new("1.21.6".to_string(), Loader::Fabric, "H:/_mc/ice-test");
        // let config = config.inner;
        // println!("{:?}", config);
        // println!("{:?}", config.to_string());

        config.insert_mod(Mod::Modrinth(ModrinthMod {
            slug: "ashdajsdhasdk".to_string(),
            version_id_number: "asdhfjkladhsfjkl#asdasdasdasdasdasd".to_string(),
        }));
        println!("{:?}", config.to_string())
    }
}
