use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tolerant deserializer for `level`: AON returns it as a number, a numeric
/// string, or null depending on the category, so accept all three.
fn de_opt_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde_json::Value;
    Ok(match Option::<Value>::deserialize(deserializer)? {
        Some(Value::Number(n)) => n.as_i64().map(|v| v as i32),
        Some(Value::String(s)) => s.trim().parse().ok(),
        _ => None,
    })
}

/// Common fields present on all AON documents.
/// Unknown fields are captured in `extra` to avoid deserialization failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Option<String>,
    pub name: Option<String>,
    pub category: Option<String>,
    #[serde(rename = "type")]
    pub doc_type: Option<String>,
    pub url: Option<String>,
    pub text: Option<String>,
    pub markdown: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub source: Vec<String>,
    pub rarity: Option<String>,
    #[serde(default, rename = "trait")]
    pub traits: Vec<String>,
    #[serde(default)]
    pub trait_group: Vec<String>,
    pub pfs: Option<String>,
    #[serde(default, deserialize_with = "de_opt_i32")]
    pub level: Option<i32>,
    #[serde(default)]
    pub tradition: Vec<String>,
    #[serde(default)]
    pub domain: Vec<String>,
    #[serde(default)]
    pub favored_weapon: Vec<String>,
    #[serde(default)]
    pub sanctification: Vec<String>,
    #[serde(default)]
    pub attribute: Vec<String>,
    #[serde(default)]
    pub deity: Vec<String>,
    #[serde(default)]
    pub remaster_id: Vec<String>,
    #[serde(default)]
    pub legacy_id: Vec<String>,

    /// All other fields from the source document.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Document {
    /// Short display: name and key info.
    pub fn display_short(&self) -> String {
        let name = self.name.as_deref().unwrap_or("Unknown");
        let cat = self.category.as_deref().unwrap_or("");
        match self.level {
            Some(lvl) => format!("[{cat}] {name} (Level {lvl})"),
            None => format!("[{cat}] {name}"),
        }
    }
}
