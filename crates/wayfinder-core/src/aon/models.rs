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
        Some(Value::Number(n)) => n.as_i64().and_then(|v| i32::try_from(v).ok()),
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
    /// Pre-remaster names of a remastered entry (e.g. "Magic Missile" on
    /// Force Barrage).
    #[serde(default)]
    pub legacy_name: Vec<String>,

    /// All other fields from the source document.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Document {
    /// A legacy (pre-remaster) entry that the Remaster replaced.
    pub fn is_legacy(&self) -> bool {
        !self.remaster_id.is_empty()
    }

    /// The entry's AON page, absolute (the index stores site-relative paths).
    pub fn absolute_url(&self, base_url: &str) -> Option<String> {
        let url = self
            .url
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())?;
        Some(
            if url.starts_with("http://") || url.starts_with("https://") {
                url.to_string()
            } else {
                format!("{base_url}{url}")
            },
        )
    }

    /// The rarity, unless it is "common" or already listed among the traits
    /// (AON repeats non-common rarities as a trait).
    pub fn notable_rarity(&self) -> Option<&str> {
        self.rarity
            .as_deref()
            .filter(|r| !r.eq_ignore_ascii_case("common"))
            .filter(|r| !self.traits.iter().any(|t| t.eq_ignore_ascii_case(r)))
    }

    /// Short edition note: `legacy` for replaced entries, `formerly X` for
    /// renamed remastered ones, otherwise `None`.
    pub fn edition_note(&self) -> Option<String> {
        if self.is_legacy() {
            Some("legacy".to_string())
        } else if !self.legacy_name.is_empty() {
            Some(format!("formerly {}", self.legacy_name.join(", ")))
        } else {
            None
        }
    }

    /// Whether `name` is this entry's name or one of its legacy names
    /// (case-insensitive).
    pub fn is_named(&self, name: &str) -> bool {
        let name = name.trim();
        self.name
            .iter()
            .chain(&self.legacy_name)
            .any(|n| n.trim().eq_ignore_ascii_case(name))
    }

    /// A string field outside the typed ones (e.g. `actions`), trimmed.
    pub fn extra_str(&self, key: &str) -> Option<&str> {
        self.extra
            .get(key)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    /// A string-array field outside the typed ones (e.g. `source_raw`).
    pub fn extra_strings(&self, key: &str) -> Vec<&str> {
        self.extra
            .get(key)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default()
    }

    /// The entry's rules text as content blocks: AON's markdown when present
    /// (it keeps stat-block and heightening structure), else the flat text.
    pub fn content(&self, base_url: &str) -> Vec<crate::render::ContentBlock> {
        let raw = self
            .markdown
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .or(self.text.as_deref())
            .unwrap_or("");
        crate::render::parse_content(raw, base_url)
    }

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
