//! [`Wayfinder`]: the one entry point frontends use for a game's data.
//!
//! It owns the policy the `wf` CLI and the MCP server share: what a search
//! or lookup sends to AON, response caching, remaster handling, and category
//! resolution against the live index. Frontends only parse input and format
//! output.

use std::sync::Arc;

use serde_json::Value;
use tokio::sync::OnceCell;

use crate::aon::categories::ALL_CATEGORIES;
use crate::aon::{
    AonClient, CategoryError, Document, GameSystem, Lookup, Pick, Search, categories_body,
    parse_documents, parse_total, pick, resolve_category_in,
};
use crate::cache::ResponseCache;
use crate::error::{Error, Result};

/// One page of search results.
#[derive(Debug, Clone)]
pub struct Page {
    /// Matches in the whole index (not just this page).
    pub total: u64,
    /// How many matches precede this page.
    pub offset: u32,
    pub docs: Vec<Document>,
}

impl Page {
    /// The offset of the next page, if there are more matches.
    pub fn next_offset(&self) -> Option<u32> {
        let end = self.offset as u64 + self.docs.len() as u64;
        (!self.docs.is_empty() && end < self.total).then_some(end as u32)
    }
}

/// Searches, lookups and categories for one game, cached.
pub struct Wayfinder {
    client: AonClient,
    cache: Option<Arc<ResponseCache>>,
    /// `(category, entry count)`, most entries first; fetched once.
    categories: OnceCell<Vec<(String, i64)>>,
}

impl Wayfinder {
    /// A service over `client`, caching through `cache` when given.
    pub fn new(client: AonClient, cache: Option<Arc<ResponseCache>>) -> Self {
        Self {
            client,
            cache,
            categories: OnceCell::new(),
        }
    }

    pub fn system(&self) -> GameSystem {
        self.client.system
    }

    pub fn cache(&self) -> Option<&ResponseCache> {
        self.cache.as_deref()
    }

    /// Post a raw Elasticsearch body, answering from the cache when it can.
    /// Only successful responses are cached.
    pub async fn raw(&self, body: &Value) -> Result<Value> {
        let system = self.client.system;
        // The endpoint is part of the key so a mirror or test server never
        // serves (or poisons) live-AON entries. serde_json sorts object keys,
        // so equal bodies serialize identically.
        let key = format!("{} {} {body}", self.client.endpoint(), system.index());
        if let Some(hit) = self.cache.as_ref().and_then(|c| c.get(&key)) {
            return Ok(hit);
        }
        let response = self.client.search_raw(body).await?;
        if let Some(cache) = &self.cache {
            // Best effort: a failed write just means a later miss.
            let _ = cache.put(&key, system.label(), &response);
        }
        Ok(response)
    }

    /// Run a search.
    pub async fn search(&self, search: &Search) -> Result<Page> {
        let raw = self.raw(&search.body()?).await?;
        Ok(Page {
            total: parse_total(&raw).unwrap_or(0).max(0) as u64,
            offset: search.effective_offset(),
            docs: parse_documents(&raw)?,
        })
    }

    /// Find one entry. By URL, the page at that URL; by name, the best
    /// candidate (see [`pick`]) and the others sharing its name.
    pub async fn lookup(&self, lookup: &Lookup) -> Result<Option<Pick>> {
        let raw = self.raw(&lookup.body(self.system().base_url())?).await?;
        let docs = parse_documents(&raw)?;
        Ok(match (lookup.by_url(), &lookup.name) {
            (None, Some(name)) => pick(name, docs),
            _ => docs.into_iter().next().map(|best| Pick {
                best,
                same_name: Vec::new(),
            }),
        })
    }

    /// The game's live categories with entry counts, most entries first.
    pub async fn categories(&self) -> Result<&[(String, i64)]> {
        let cats = self
            .categories
            .get_or_try_init(|| async {
                let raw = self.raw(&categories_body()).await?;
                parse_category_buckets(&raw)
            })
            .await?;
        Ok(cats)
    }

    /// Resolve user input to a category, forgiving case and plurals ("Spells"
    /// is "spell"), against the live list, or the built-in one when AON
    /// cannot be reached. An unknown category is an error carrying the
    /// closest match, since filtering on it would silently find nothing.
    pub async fn resolve_category(
        &self,
        input: &str,
    ) -> std::result::Result<String, CategoryError> {
        match self.categories().await {
            Ok(cats) => {
                let names: Vec<&str> = cats.iter().map(|(n, _)| n.as_str()).collect();
                resolve_category_in(input, &names)
            }
            Err(_) => resolve_category_in(input, ALL_CATEGORIES),
        }
    }
}

/// Parse the terms-aggregation buckets of [`categories_body`]'s response.
fn parse_category_buckets(response: &Value) -> Result<Vec<(String, i64)>> {
    let buckets = response
        .pointer("/aggregations/cats/buckets")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::UnexpectedResponse("missing aggregations.cats".into()))?;
    Ok(buckets
        .iter()
        .filter_map(|b| {
            let key = b.get("key")?.as_str()?.to_string();
            let count = b.get("doc_count")?.as_i64().unwrap_or(0);
            Some((key, count))
        })
        .collect())
}

/// Reorder broad search results for display: entries whose name contains the
/// query first, then the rest grouped by category in first-appearance order.
pub fn group_broad_results(results: Vec<Document>, query: &str) -> Vec<Document> {
    let q = query.to_lowercase();
    let (mut named, rest): (Vec<_>, Vec<_>) = results
        .into_iter()
        .partition(|d| d.name.as_deref().unwrap_or("").to_lowercase().contains(&q));
    let mut by_cat: indexmap::IndexMap<String, Vec<Document>> = indexmap::IndexMap::new();
    for doc in rest {
        let cat = doc.category.clone().unwrap_or_default();
        by_cat.entry(cat).or_default().push(doc);
    }
    named.extend(by_cat.into_values().flatten());
    named
}
