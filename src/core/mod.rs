#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[non_exhaustive]
pub enum Mod {
    Modrinth(ModrinthMod),
    Unknown,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ModrinthMod {
    pub slug: String,
    pub version_id_number: String,
}

impl ModrinthMod {
    pub fn version_id(&self) -> &str {
        &self.version_id_number.split("#").next().unwrap()
    }
    pub fn version_number(&self) -> &str {
        &self.version_id_number.split("#").skip(1).next().unwrap()
    }
}
