//! Fetching one entry by name or URL, and choosing among the candidates.
//!
//! AON ranks every entry named exactly "Shield" (a spell, a thaumaturge
//! implement, a weapon group) the same, so search order alone picks one
//! arbitrarily. [`pick`] prefers an exact name, then the category a player most
//! likely means, and keeps the rest so callers can offer them.

use serde_json::{Value, json};

use super::models::Document;
use super::query::{Edition, MAX_INPUT_LEN, invalid, non_empty};
use crate::error::Result;

/// How many candidates a lookup by name fetches.
pub const CANDIDATES: u32 = 10;

/// When several entries share a name and no category was given, earlier
/// categories win; unlisted ones follow in search order.
const CATEGORY_PRIORITY: &[&str] = &[
    "spell",
    "feat",
    "action",
    "condition",
    "equipment",
    "weapon",
    "armor",
    "shield",
    "creature",
    "ancestry",
    "heritage",
    "class",
    "archetype",
    "background",
    "deity",
    "ritual",
    "hazard",
    "skill",
    "trait",
    "rules",
];

/// A request for one entry: by `url`, or by `name` (current or legacy),
/// optionally within a `category`.
#[derive(Debug, Clone, Default)]
pub struct Lookup {
    pub name: Option<String>,
    /// An AoN page URL, absolute or site-relative; wins over `name`.
    pub url: Option<String>,
    pub category: Option<String>,
    /// Ignored for URL lookups, which name one exact page.
    pub edition: Edition,
}

impl Lookup {
    /// Look up `name`, optionally within `category`.
    pub fn named(name: &str, category: Option<&str>) -> Self {
        Self {
            name: Some(name.to_string()),
            category: category.map(str::to_string),
            ..Self::default()
        }
    }

    /// The URL, if this is a lookup by URL.
    pub fn by_url(&self) -> Option<&str> {
        non_empty(&self.url)
    }

    /// The Elasticsearch body. `base_url` (e.g. `https://2e.aonprd.com`) is
    /// stripped from absolute URLs, since the index stores relative paths.
    pub fn body(&self, base_url: &str) -> Result<Value> {
        let inputs = [&self.name, &self.url, &self.category];
        if inputs
            .iter()
            .filter_map(|s| s.as_deref())
            .any(|s| s.len() > MAX_INPUT_LEN)
        {
            return invalid(format!("input exceeds {MAX_INPUT_LEN} characters"));
        }
        if let Some(url) = self.by_url() {
            return Ok(json!({
                "size": 1,
                "query": { "term": { "url": relative_url(url, base_url) } }
            }));
        }
        let Some(name) = non_empty(&self.name) else {
            return invalid("either a name or a url is required".into());
        };
        let mut filter = Vec::new();
        if let Some(category) = non_empty(&self.category) {
            filter.push(json!({ "term": { "category": category.to_lowercase() } }));
        }
        Ok(json!({
            "size": CANDIDATES,
            "query": { "bool": {
                "must": [{ "multi_match": {
                    "query": name,
                    "fields": ["name^10", "legacy_name^5"],
                    "type": "phrase"
                } }],
                "filter": filter,
                "must_not": [self.edition.exclusion()]
            } }
        }))
    }
}

/// Reduce an absolute AoN URL (http or https) to the path the index stores.
fn relative_url(url: &str, base_url: &str) -> String {
    let url = url.trim();
    let host = base_url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    ["https://", "http://"]
        .iter()
        .find_map(|scheme| url.strip_prefix(scheme)?.strip_prefix(host))
        .unwrap_or(url)
        .to_string()
}

/// The entry a lookup chose, plus the other candidates that share its exact
/// name (other categories, usually).
#[derive(Debug, Clone)]
pub struct Pick {
    pub best: Document,
    pub same_name: Vec<Document>,
}

/// Rank among exact matches: an entry's current name beats a legacy name
/// ("Flanking" the rule over "3D Flanking", formerly "Flanking"), then the
/// likelier category.
fn rank(doc: &Document, name: &str) -> (bool, usize) {
    let current = doc
        .name
        .as_deref()
        .is_some_and(|n| n.trim().eq_ignore_ascii_case(name.trim()));
    let cat = doc.category.as_deref().unwrap_or("");
    let priority = CATEGORY_PRIORITY
        .iter()
        .position(|&c| c == cat)
        .unwrap_or(CATEGORY_PRIORITY.len());
    (!current, priority)
}

/// Choose the best of `candidates` (in search-relevance order) for `name`.
pub fn pick(name: &str, candidates: Vec<Document>) -> Option<Pick> {
    let (mut exact, rest): (Vec<_>, Vec<_>) =
        candidates.into_iter().partition(|d| d.is_named(name));
    if exact.is_empty() {
        // No exact name: trust search relevance.
        return rest.into_iter().next().map(|best| Pick {
            best,
            same_name: Vec::new(),
        });
    }
    // Stable, so equal priorities keep search order.
    exact.sort_by_key(|d| rank(d, name));
    let best = exact.remove(0);
    Some(Pick {
        best,
        same_name: exact,
    })
}
