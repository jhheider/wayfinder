//! Tool parameter structs (the schemars-described MCP interface) and the
//! game-selection parsing that maps them onto `wayfinder_core`'s `GameSystem`.

use serde::Deserialize;
use wayfinder_core::aon::GameSystem;

/// A curated set of common, broadly-useful categories, surfaced in tool hints.
/// `list_categories` queries the live index for the authoritative set, so this
/// is documentation only and does not gate searches.
pub const COMMON_CATEGORIES: &[&str] = &[
    "action",
    "ancestry",
    "archetype",
    "armor",
    "background",
    "class",
    "class-feature",
    "creature",
    "creature-family",
    "deity",
    "equipment",
    "feat",
    "hazard",
    "heritage",
    "ritual",
    "rules",
    "shield",
    "skill",
    "source",
    "spell",
    "trait",
    "weapon",
];

/// Hint string listing common categories, for tool descriptions.
pub fn common_categories_hint() -> String {
    COMMON_CATEGORIES.join(", ")
}

/// Parse an optional `game` parameter into a `GameSystem`; defaults to
/// Pathfinder 2e. Accepts common spellings ("pf2e"/"pathfinder", etc.).
pub fn game_system(value: Option<&str>) -> Result<GameSystem, String> {
    match value.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(GameSystem::Pathfinder),
        Some(v) => match v.to_lowercase().as_str() {
            "pf2e" | "pf" | "pathfinder" | "pathfinder2e" | "aonprd" => Ok(GameSystem::Pathfinder),
            "sf2e" | "sf" | "starfinder" | "starfinder2e" | "aonsf" | "aonsrd" => {
                Ok(GameSystem::Starfinder)
            }
            other => Err(format!(
                "unknown game {other:?}; expected \"pf2e\" (Pathfinder 2e) or \"sf2e\" (Starfinder 2e)"
            )),
        },
    }
}

/// Parameters for the `search` tool. Every field is optional; an empty set
/// returns the first page of all entries.
#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    /// Which game to search: "pf2e" (Pathfinder 2e, the default) or "sf2e"
    /// (Starfinder 2e).
    #[serde(default)]
    pub game: Option<String>,

    /// Free-text query matched against entry names, summaries, and rules text
    /// (e.g. "magic missile", "flat-footed", "grab an edge"). Omit to browse by
    /// filters alone.
    #[serde(default)]
    pub query: Option<String>,

    /// Restrict to a single category, e.g. "spell", "feat", "creature",
    /// "equipment", "ancestry", "class". Use `list_categories` for the full list.
    #[serde(default)]
    pub category: Option<String>,

    /// Restrict to entries that have ALL of these traits (case-insensitive),
    /// e.g. ["Fire", "Evocation"] or ["uncommon"].
    #[serde(default)]
    pub traits: Vec<String>,

    /// Minimum level/rank, inclusive (spells use rank; creatures use level).
    #[serde(default)]
    pub min_level: Option<i64>,

    /// Maximum level/rank, inclusive.
    #[serde(default)]
    pub max_level: Option<i64>,

    /// Restrict to a source book by name, e.g. "Player Core".
    #[serde(default)]
    pub source: Option<String>,

    /// Restrict to a rarity: "common", "uncommon", "rare", or "unique".
    #[serde(default)]
    pub rarity: Option<String>,

    /// Maximum number of results to return (default 10, capped at 50).
    #[serde(default)]
    pub limit: Option<u32>,

    /// Result ordering: "relevance" (default), "level" (ascending), or "name".
    #[serde(default)]
    pub sort: Option<String>,
}

impl SearchParams {
    /// Effective result limit, clamped to a sane range.
    pub fn effective_limit(&self) -> u32 {
        self.limit.unwrap_or(10).clamp(1, 50)
    }
}

/// Parameters for tools that only need to pick a game (e.g. `list_categories`).
#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct GameParams {
    /// Which game to query: "pf2e" (Pathfinder 2e, the default) or "sf2e".
    #[serde(default)]
    pub game: Option<String>,
}

/// Parameters for the `get` tool. Provide either `name` (optionally narrowed by
/// `category`) or an AoN `url`.
#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
pub struct GetParams {
    /// Which game to query: "pf2e" (Pathfinder 2e, the default) or "sf2e".
    #[serde(default)]
    pub game: Option<String>,

    /// Exact entry name, e.g. "Fireball". Legacy/pre-remaster names are also
    /// matched (e.g. "Magic Missile" finds "Force Barrage"). Either `name` or
    /// `url` is required.
    #[serde(default)]
    pub name: Option<String>,

    /// Category to disambiguate when several entries share a name, e.g. "spell".
    #[serde(default)]
    pub category: Option<String>,

    /// An AoN document URL, full or relative, e.g. "/Spells.aspx?ID=119".
    /// Takes precedence over `name`.
    #[serde(default)]
    pub url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_defaults_to_pathfinder() {
        assert_eq!(game_system(None).unwrap(), GameSystem::Pathfinder);
        assert_eq!(game_system(Some("  ")).unwrap(), GameSystem::Pathfinder);
    }

    #[test]
    fn game_parses_aliases_case_insensitively() {
        for v in ["pf2e", "PF2E", "Pathfinder", "pathfinder2e", "aonprd"] {
            assert_eq!(game_system(Some(v)).unwrap(), GameSystem::Pathfinder, "{v}");
        }
        for v in ["sf2e", "SF2E", "Starfinder", "aonsf", "aonsrd"] {
            assert_eq!(game_system(Some(v)).unwrap(), GameSystem::Starfinder, "{v}");
        }
    }

    #[test]
    fn game_rejects_unknown() {
        assert!(game_system(Some("dnd")).is_err());
    }

    #[test]
    fn limit_clamps() {
        assert_eq!(SearchParams::default().effective_limit(), 10);
        let p = SearchParams {
            limit: Some(1000),
            ..Default::default()
        };
        assert_eq!(p.effective_limit(), 50);
    }
}
