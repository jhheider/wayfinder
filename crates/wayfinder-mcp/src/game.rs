//! One game's AON client plus its live category list, fetched once per
//! process and reused to validate `category` arguments.

use std::sync::Arc;

use anyhow::bail;
use serde_json::Value;
use tokio::sync::OnceCell;
use wayfinder_core::aon::{AonClient, CategoryError, normalize_category, resolve_category_in};

use crate::query::build_categories_query;

#[derive(Clone)]
pub struct Game {
    pub client: AonClient,
    /// `(category, entry count)`, most entries first. Unset until the first
    /// successful fetch; a failed fetch is retried on the next call.
    categories: Arc<OnceCell<Vec<(String, i64)>>>,
}

impl Game {
    pub fn new(client: AonClient) -> Self {
        Self {
            client,
            categories: Arc::default(),
        }
    }

    /// The live categories and their entry counts.
    pub async fn categories(&self) -> anyhow::Result<&[(String, i64)]> {
        let cats = self
            .categories
            .get_or_try_init(|| async {
                let raw = self.client.search_raw(&build_categories_query()).await?;
                parse_category_buckets(&raw)
            })
            .await?;
        Ok(cats)
    }

    /// Resolve a `category` argument against this game's live categories,
    /// forgiving case and plurals ("Spells" is "spell"). Blank is `None`. An
    /// unknown category is an error naming the closest match, since filtering
    /// on it would silently find nothing. If AON cannot list its categories,
    /// the normalized input is passed through unchecked.
    pub async fn resolve_category(&self, input: &str) -> anyhow::Result<Option<String>> {
        if input.trim().is_empty() {
            return Ok(None);
        }
        let Ok(cats) = self.categories().await else {
            return Ok(Some(normalize_category(input.trim())));
        };
        let names: Vec<&str> = cats.iter().map(|(name, _)| name.as_str()).collect();
        let label = self.client.system.label();
        match resolve_category_in(input, &names) {
            Ok(cat) => Ok(Some(cat)),
            Err(CategoryError::Suggested { input, suggestion }) => bail!(
                "unknown {label} category {input:?}; did you mean {suggestion:?}? \
                 `list_categories` lists them all"
            ),
            Err(CategoryError::Unknown(input)) => {
                bail!("unknown {label} category {input:?}; `list_categories` lists them all")
            }
        }
    }
}

/// Parse the terms-aggregation buckets from a categories query.
fn parse_category_buckets(response: &Value) -> anyhow::Result<Vec<(String, i64)>> {
    let Some(buckets) = response
        .pointer("/aggregations/cats/buckets")
        .and_then(Value::as_array)
    else {
        bail!("unexpected aggregation response from AoN");
    };
    Ok(buckets
        .iter()
        .filter_map(|b| {
            let key = b.get("key")?.as_str()?.to_string();
            let count = b.get("doc_count")?.as_i64().unwrap_or(0);
            Some((key, count))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::parse_category_buckets;
    use serde_json::json;

    #[test]
    fn parses_aggregation_buckets() {
        let resp = json!({"aggregations": {"cats": {"buckets": [
            {"key": "spell", "doc_count": 405},
            {"key": "feat", "doc_count": 2130}
        ]}}});
        let cats = parse_category_buckets(&resp).unwrap();
        assert_eq!(
            cats,
            vec![("spell".to_string(), 405), ("feat".to_string(), 2130)]
        );
    }

    #[test]
    fn errors_on_missing_aggregation() {
        assert!(parse_category_buckets(&json!({"hits": {}})).is_err());
    }
}
