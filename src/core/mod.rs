use crate::config::TomlMod;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[non_exhaustive]
pub enum Mod {
    Modrinth(ModrinthMod),
    Unknown,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ModrinthMod {
    pub slug: String,
    pub version: String,
}

impl From<(String, TomlMod)> for Mod {
    fn from((slug, value): (String, TomlMod)) -> Self {
        match value {
            TomlMod::Simple(version) => Mod::Modrinth(ModrinthMod { slug, version }),
            TomlMod::Detailed(detailed) => {
                if let Some(version) = detailed.version {
                    Mod::Modrinth(ModrinthMod { slug, version })
                } else {
                    Mod::Unknown
                }
            }
        }
    }
}
