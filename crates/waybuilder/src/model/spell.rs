use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellEntry {
    pub name: String,
    pub rank: u8,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpellCaster {
    pub tradition: Option<String>,
    pub ability: Option<String>,
    pub spells: Vec<SpellEntry>,
    pub focus_spells: Vec<SpellEntry>,
}
