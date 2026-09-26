//! Search requests: what a caller asks for, and the Elasticsearch body that
//! asks AON for it. Pure (no I/O); [`crate::Wayfinder`] posts the bodies.

use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::error::{Error, Result};

/// Results per page when the caller sets no limit.
pub const DEFAULT_LIMIT: u32 = 10;
/// Most results one request may return. AON's own UI pages in 50s; a low
/// ceiling nudges toward narrower queries instead of bulk pulls.
pub const MAX_LIMIT: u32 = 100;
/// Longest accepted text input (query, name, filter value, ...).
pub const MAX_INPUT_LEN: usize = 500;
/// Most `field = value` filters one request may carry.
pub const MAX_FIELD_FILTERS: usize = 20;
/// Elasticsearch's default `index.max_result_window`: `from + size` may not
/// exceed it.
const MAX_WINDOW: u32 = 10_000;

/// Fields fetched for a result list, when [`Search::full`] is off.
const SUMMARY_FIELDS: &[&str] = &[
    "id",
    "name",
    "url",
    "category",
    "type",
    "level",
    "rarity",
    "pfs",
    "trait",
    "source",
    "summary",
    "actions",
    "legacy_name",
    "remaster_id",
    "legacy_id",
];

/// Which side of each legacy/remaster pair to return. Entries the Remaster
/// never touched belong to both and always appear.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    /// Remastered entries replace the legacy ones they superseded.
    #[default]
    Remastered,
    /// Legacy (pre-remaster) entries replace their remastered versions.
    Legacy,
}

impl Edition {
    /// `Legacy` when `legacy` is set.
    pub fn legacy_if(legacy: bool) -> Self {
        if legacy {
            Self::Legacy
        } else {
            Self::Remastered
        }
    }

    /// The `must_not` clause that hides the other side: legacy entries carry
    /// `remaster_id`, remastered ones `legacy_id`.
    pub(crate) fn exclusion(self) -> Value {
        let field = match self {
            Self::Remastered => "remaster_id",
            Self::Legacy => "legacy_id",
        };
        json!({ "exists": { "field": field } })
    }
}

/// Result ordering.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Sort {
    /// Best match first.
    #[default]
    Relevance,
    /// Lowest level/rank first, then relevance.
    Level,
    /// Alphabetical by name.
    Name,
}

/// A search over one game's entries. Every field is optional; an empty
/// search lists the first page of everything.
#[derive(Debug, Clone, Default)]
pub struct Search {
    /// Free text, matched against names (current and legacy), summaries and
    /// rules text.
    pub text: Option<String>,
    /// A phrase the entry's name must contain.
    pub name: Option<String>,
    /// One category, e.g. `spell` (resolve user input first, e.g. with
    /// [`crate::Wayfinder::resolve_category`]).
    pub category: Option<String>,
    /// Traits the entry must all have (case-insensitive).
    pub traits: Vec<String>,
    /// Exact `field = value` filters on other indexed fields (see
    /// [`super::categories::filterable_fields`]).
    pub fields: Vec<(String, String)>,
    /// Lowest level/rank, inclusive.
    pub min_level: Option<i64>,
    /// Highest level/rank, inclusive.
    pub max_level: Option<i64>,
    /// A source book, matched as a phrase ("Player Core" also matches
    /// "Player Core 2", never "Core Rulebook").
    pub source: Option<String>,
    /// `common`, `uncommon`, `rare` or `unique`.
    pub rarity: Option<String>,
    pub edition: Edition,
    pub sort: Sort,
    /// Page size; `None` is [`DEFAULT_LIMIT`], and it is capped at [`MAX_LIMIT`].
    pub limit: Option<u32>,
    /// Results to skip.
    pub offset: u32,
    /// Fetch every field of each entry, not just the summary set.
    pub full: bool,
}

impl Search {
    /// The page size this search will request.
    pub fn effective_limit(&self) -> u32 {
        self.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
    }

    /// The offset this search will request (kept inside the result window).
    pub fn effective_offset(&self) -> u32 {
        self.offset.min(MAX_WINDOW - self.effective_limit())
    }

    /// Reject input AON would choke on or silently misread.
    pub fn validate(&self) -> Result<()> {
        let texts = [
            &self.text,
            &self.name,
            &self.category,
            &self.source,
            &self.rarity,
        ];
        let too_long = texts.iter().filter_map(|t| t.as_deref()).chain(
            self.traits
                .iter()
                .chain(self.fields.iter().flat_map(|(k, v)| [k, v]))
                .map(String::as_str),
        );
        for t in too_long {
            if t.len() > MAX_INPUT_LEN {
                return invalid(format!("input exceeds {MAX_INPUT_LEN} characters"));
            }
        }
        if self.fields.len() > MAX_FIELD_FILTERS {
            return invalid(format!(
                "too many field filters (maximum {MAX_FIELD_FILTERS})"
            ));
        }
        if let (Some(min), Some(max)) = (self.min_level, self.max_level)
            && min > max
        {
            return invalid(format!(
                "min level ({min}) is greater than max level ({max})"
            ));
        }
        Ok(())
    }

    /// The Elasticsearch request body.
    pub fn body(&self) -> Result<Value> {
        self.validate()?;
        let mut must = Vec::new();
        if let Some(text) = non_empty(&self.text) {
            must.push(json!({ "multi_match": {
                "query": text,
                "fields": ["name^10", "legacy_name^5", "summary^3", "text"],
                "type": "best_fields"
            } }));
        }
        if let Some(name) = non_empty(&self.name) {
            must.push(json!({ "match_phrase": { "name": name } }));
        }
        if must.is_empty() {
            must.push(json!({ "match_all": {} }));
        }

        let mut filter = Vec::new();
        if let Some(category) = non_empty(&self.category) {
            filter.push(json!({ "term": { "category": category.to_lowercase() } }));
        }
        for t in self
            .traits
            .iter()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
        {
            filter.push(json!({ "term": { "trait": t.to_lowercase() } }));
        }
        for (field, value) in &self.fields {
            filter.push(json!({ "term": { field.as_str(): value } }));
        }
        if self.min_level.is_some() || self.max_level.is_some() {
            let mut range = Map::new();
            if let Some(min) = self.min_level {
                range.insert("gte".into(), json!(min));
            }
            if let Some(max) = self.max_level {
                range.insert("lte".into(), json!(max));
            }
            filter.push(json!({ "range": { "level": range } }));
        }
        if let Some(source) = non_empty(&self.source) {
            filter.push(json!({ "match_phrase": { "source": source } }));
        }
        if let Some(rarity) = non_empty(&self.rarity) {
            filter.push(json!({ "term": { "rarity": rarity.to_lowercase() } }));
        }

        let sort = match self.sort {
            Sort::Relevance => json!(["_score"]),
            Sort::Level => json!([{ "level": "asc" }, "_score"]),
            Sort::Name => json!([{ "name.keyword": "asc" }]),
        };
        let mut body = json!({
            "size": self.effective_limit(),
            "from": self.effective_offset(),
            "query": { "bool": {
                "must": must,
                "filter": filter,
                "must_not": [self.edition.exclusion()]
            } },
            "sort": sort
        });
        if !self.full {
            body["_source"] = json!(SUMMARY_FIELDS);
        }
        Ok(body)
    }
}

/// The aggregation body listing a game's categories with entry counts.
pub fn categories_body() -> Value {
    json!({
        "size": 0,
        "aggs": { "cats": { "terms": { "field": "category", "size": 500 } } }
    })
}

pub(crate) fn invalid<T>(msg: String) -> Result<T> {
    Err(Error::InvalidInput(msg))
}

/// The trimmed string, if the option holds a non-empty one.
pub(crate) fn non_empty(opt: &Option<String>) -> Option<&str> {
    opt.as_deref().map(str::trim).filter(|s| !s.is_empty())
}
