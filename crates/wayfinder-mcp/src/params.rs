//! Tool parameter structs (the schemars-described MCP interface) and their
//! mapping onto `wayfinder_core`'s requests: `Search`, `Lookup`, `GameSystem`.

use serde::Deserialize;
use wayfinder_core::aon::{Edition, GameSystem, Lookup, Search, Sort};

/// Most results one `search` returns: a model reads every line, so the
/// ceiling is lower than core's.
const MAX_LIMIT: u32 = 50;

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
        Some(v) => v.parse(),
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
    /// e.g. ["fire", "healing"] or ["uncommon"].
    #[serde(default)]
    pub traits: Vec<String>,

    /// Minimum level/rank, inclusive (spells use rank; creatures use level).
    #[serde(default)]
    pub min_level: Option<i64>,

    /// Maximum level/rank, inclusive.
    #[serde(default)]
    pub max_level: Option<i64>,

    /// Restrict to a source book by name, matched as a phrase, e.g. "Player
    /// Core" (which also matches "Player Core 2").
    #[serde(default)]
    pub source: Option<String>,

    /// Restrict to a rarity: "common", "uncommon", "rare", or "unique".
    #[serde(default)]
    pub rarity: Option<String>,

    /// Maximum number of results to return (default 10, capped at 50).
    #[serde(default)]
    pub limit: Option<u32>,

    /// Number of results to skip, for paging past the first `limit` matches.
    #[serde(default)]
    pub offset: Option<u32>,

    /// Result ordering (default "relevance").
    #[serde(default)]
    pub sort: Sort,

    /// Prefer legacy (pre-remaster) versions. By default an entry that the
    /// Remaster replaced is hidden in favor of its remastered version (e.g.
    /// the Player Core "Heal", not the Core Rulebook one); set true to see the
    /// legacy version instead. Entries the Remaster never touched always appear.
    #[serde(default)]
    pub legacy: bool,
}

impl SearchParams {
    /// The core search these parameters ask for. `category` must already be
    /// resolved (see `Wayfinder::resolve_category`).
    pub fn to_search(&self, category: Option<String>) -> Search {
        Search {
            text: self.query.clone(),
            category,
            traits: self.traits.clone(),
            min_level: self.min_level,
            max_level: self.max_level,
            source: self.source.clone(),
            rarity: self.rarity.clone(),
            edition: Edition::legacy_if(self.legacy),
            sort: self.sort,
            limit: Some(self.limit.unwrap_or(10).clamp(1, MAX_LIMIT)),
            offset: self.offset.unwrap_or(0),
            ..Search::default()
        }
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

    /// Prefer legacy (pre-remaster) versions. By default an entry that the
    /// Remaster replaced is hidden in favor of its remastered version (e.g.
    /// the Player Core "Heal", not the Core Rulebook one); set true to see the
    /// legacy version instead. Entries the Remaster never touched always appear.
    #[serde(default)]
    pub legacy: bool,
}

impl GetParams {
    /// The core lookup these parameters ask for, with `category` resolved.
    pub fn to_lookup(&self, category: Option<String>) -> Lookup {
        Lookup {
            name: self.name.clone(),
            url: self.url.clone(),
            category,
            edition: Edition::legacy_if(self.legacy),
        }
    }
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
    fn game_rejects_unknown() {
        assert!(game_system(Some("dnd")).is_err());
    }

    #[test]
    fn search_params_map_onto_core_search() {
        let p: SearchParams = serde_json::from_str(
            r#"{"query":"fire","traits":["fire"],"limit":1000,"offset":20,"legacy":true,"sort":"level"}"#,
        )
        .unwrap();
        let s = p.to_search(Some("spell".into()));
        assert_eq!(s.text.as_deref(), Some("fire"));
        assert_eq!(s.category.as_deref(), Some("spell"));
        assert_eq!(s.limit, Some(MAX_LIMIT));
        assert_eq!(s.offset, 20);
        assert_eq!(s.edition, Edition::Legacy);
        assert_eq!(s.sort, Sort::Level);
        assert_eq!(SearchParams::default().to_search(None).limit, Some(10));
    }

    #[test]
    fn get_params_map_onto_core_lookup() {
        let p = GetParams {
            name: Some("Heal".into()),
            ..Default::default()
        };
        let l = p.to_lookup(None);
        assert_eq!(l.name.as_deref(), Some("Heal"));
        assert_eq!(l.edition, Edition::Remastered);
    }

    #[test]
    fn sort_rejects_unknown_values() {
        let ok: SearchParams = serde_json::from_str(r#"{"sort":"level"}"#).unwrap();
        assert_eq!(ok.sort, Sort::Level);
        assert!(serde_json::from_str::<SearchParams>(r#"{"sort":"random"}"#).is_err());
    }
}
