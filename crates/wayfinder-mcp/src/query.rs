//! Pure builders that translate tool parameters into Elasticsearch query DSL.
//!
//! These functions perform no I/O so they can be unit-tested directly. The
//! bodies they return are posted through `wayfinder_core`'s `AonClient`.

use serde_json::{Value, json};

use crate::params::{GetParams, SearchParams};

/// Fields returned for list-style `search` results (kept small for compact output).
const SEARCH_SOURCE_FIELDS: &[&str] = &[
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
];

/// Fields returned for a full `get` (adds the body text/markdown).
const DETAIL_SOURCE_FIELDS: &[&str] = &[
    "name",
    "url",
    "category",
    "type",
    "level",
    "rarity",
    "pfs",
    "trait",
    "source",
    "source_raw",
    "summary",
    "actions",
    "legacy_name",
    "text",
    "markdown",
];

/// Build the Elasticsearch request body for a `search`.
pub fn build_search_query(params: &SearchParams) -> Value {
    let mut filters: Vec<Value> = Vec::new();

    if let Some(category) = non_empty(&params.category) {
        filters.push(json!({ "term": { "category": category.to_lowercase() } }));
    }

    for trait_name in &params.traits {
        let t = trait_name.trim();
        if !t.is_empty() {
            filters.push(json!({ "term": { "trait": t.to_lowercase() } }));
        }
    }

    if params.min_level.is_some() || params.max_level.is_some() {
        let mut range = serde_json::Map::new();
        if let Some(min) = params.min_level {
            range.insert("gte".to_string(), json!(min));
        }
        if let Some(max) = params.max_level {
            range.insert("lte".to_string(), json!(max));
        }
        filters.push(json!({ "range": { "level": range } }));
    }

    if let Some(source) = non_empty(&params.source) {
        filters.push(json!({ "match": { "source": source } }));
    }

    if let Some(rarity) = non_empty(&params.rarity) {
        filters.push(json!({ "term": { "rarity": rarity.to_lowercase() } }));
    }

    let must = match non_empty(&params.query) {
        Some(query) => json!([{
            "multi_match": {
                "query": query,
                "fields": ["name^10", "summary^3", "text"],
                "type": "best_fields"
            }
        }]),
        None => json!([{ "match_all": {} }]),
    };

    let sort = match params.sort.as_deref() {
        Some("level") => json!([{ "level": "asc" }, "_score"]),
        Some("name") => json!([{ "name.keyword": "asc" }]),
        _ => json!(["_score"]),
    };

    json!({
        "size": params.effective_limit(),
        "_source": SEARCH_SOURCE_FIELDS,
        "query": { "bool": { "must": must, "filter": filters } },
        "sort": sort
    })
}

/// Build the Elasticsearch request body for a `get`.
///
/// `base_url` is the target site's base (e.g. `https://2e.aonprd.com`), used to
/// reduce a full document URL to the relative path the index stores. Returns
/// `Err` if neither `name` nor `url` is provided.
pub fn build_get_query(params: &GetParams, base_url: &str) -> Result<Value, String> {
    if let Some(url) = non_empty(&params.url) {
        return Ok(json!({
            "size": 1,
            "_source": DETAIL_SOURCE_FIELDS,
            "query": { "term": { "url": normalize_url(url, base_url) } }
        }));
    }

    let name = non_empty(&params.name)
        .ok_or_else(|| "either `name` or `url` must be provided".to_string())?;

    let mut filters: Vec<Value> = Vec::new();
    if let Some(category) = non_empty(&params.category) {
        filters.push(json!({ "term": { "category": category.to_lowercase() } }));
    }

    // Match current and legacy names as a phrase; rank exact names highest.
    let must = json!([{
        "multi_match": {
            "query": name,
            "fields": ["name^10", "legacy_name^5"],
            "type": "phrase"
        }
    }]);

    Ok(json!({
        "size": 1,
        "_source": DETAIL_SOURCE_FIELDS,
        "query": { "bool": { "must": must, "filter": filters } }
    }))
}

/// Build the aggregation body for `list_categories`: the set of categories and
/// their live document counts, ordered by frequency.
pub fn build_categories_query() -> Value {
    json!({
        "size": 0,
        "aggs": { "cats": { "terms": { "field": "category", "size": 200 } } }
    })
}

/// Reduce a full URL to the relative path the index stores (strip the host).
fn normalize_url(url: &str, base_url: &str) -> String {
    let trimmed = url.trim();
    let http = base_url.replacen("https://", "http://", 1);
    if let Some(rest) = trimmed.strip_prefix(base_url) {
        rest.to_string()
    } else if let Some(rest) = trimmed.strip_prefix(http.as_str()) {
        rest.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Return the trimmed string if the option holds a non-empty value.
fn non_empty(opt: &Option<String>) -> Option<&str> {
    opt.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{GetParams, SearchParams};

    #[test]
    fn empty_search_is_match_all_with_default_limit() {
        let q = build_search_query(&SearchParams::default());
        assert_eq!(q["size"], json!(10));
        assert_eq!(q["query"]["bool"]["must"][0], json!({ "match_all": {} }));
        assert_eq!(q["query"]["bool"]["filter"], json!([]));
        assert_eq!(q["sort"], json!(["_score"]));
    }

    #[test]
    fn text_category_level_and_sort_compose() {
        let params = SearchParams {
            query: Some("fireball".to_string()),
            category: Some("Spell".to_string()),
            traits: vec!["Fire".to_string(), " evocation ".to_string()],
            min_level: Some(1),
            max_level: Some(5),
            rarity: Some("Common".to_string()),
            limit: Some(3),
            sort: Some("level".to_string()),
            ..Default::default()
        };
        let q = build_search_query(&params);

        assert_eq!(q["size"], json!(3));
        assert_eq!(
            q["query"]["bool"]["must"][0]["multi_match"]["query"],
            json!("fireball")
        );
        let filters = q["query"]["bool"]["filter"].as_array().unwrap();
        assert!(filters.contains(&json!({ "term": { "category": "spell" } })));
        assert!(filters.contains(&json!({ "term": { "trait": "fire" } })));
        assert!(filters.contains(&json!({ "term": { "trait": "evocation" } })));
        assert!(filters.contains(&json!({ "range": { "level": { "gte": 1, "lte": 5 } } })));
        assert!(filters.contains(&json!({ "term": { "rarity": "common" } })));
        assert_eq!(q["sort"], json!([{ "level": "asc" }, "_score"]));
    }

    #[test]
    fn name_sort_uses_keyword_field() {
        let params = SearchParams {
            sort: Some("name".to_string()),
            ..Default::default()
        };
        assert_eq!(
            build_search_query(&params)["sort"],
            json!([{ "name.keyword": "asc" }])
        );
    }

    #[test]
    fn get_by_name_matches_current_and_legacy() {
        let params = GetParams {
            name: Some("Magic Missile".to_string()),
            category: Some("spell".to_string()),
            ..Default::default()
        };
        let q = build_get_query(&params, "https://2e.aonprd.com").unwrap();
        assert_eq!(q["size"], json!(1));
        assert_eq!(
            q["query"]["bool"]["must"][0]["multi_match"]["type"],
            json!("phrase")
        );
        assert_eq!(
            q["query"]["bool"]["filter"],
            json!([{ "term": { "category": "spell" } }])
        );
    }

    #[test]
    fn get_by_url_strips_host() {
        let params = GetParams {
            url: Some("https://2e.aonprd.com/Spells.aspx?ID=119".to_string()),
            ..Default::default()
        };
        let q = build_get_query(&params, "https://2e.aonprd.com").unwrap();
        assert_eq!(
            q["query"],
            json!({ "term": { "url": "/Spells.aspx?ID=119" } })
        );
    }

    #[test]
    fn get_requires_name_or_url() {
        assert!(build_get_query(&GetParams::default(), "https://2e.aonprd.com").is_err());
    }
}
