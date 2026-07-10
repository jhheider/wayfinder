use std::path::Path;

use crate::aon::client::SearchClient;
use crate::aon::{Document, SearchQuery};
use crate::cache::CacheStore;
use crate::error::Result;

/// Generic search service parameterized over a `SearchClient` implementation.
pub struct SearchService<C: SearchClient> {
    client: C,
    cache_path: Box<Path>,
}

impl<C: SearchClient> SearchService<C> {
    pub fn new(client: C, cache_path: &Path) -> Self {
        Self {
            client,
            cache_path: cache_path.into(),
        }
    }

    /// Access the underlying client (useful for testing).
    pub fn client(&self) -> &C {
        &self.client
    }

    /// Search: always hits the client, opportunistically caches results.
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<Document>> {
        let docs = self.client.search(query).await?;
        cache_documents(&self.cache_path, &docs);
        Ok(docs)
    }

    /// Show a document by name: cache-first, then client fallback.
    pub async fn show(&self, name: &str, category: Option<&str>) -> Result<Vec<Document>> {
        if let Ok(store) = CacheStore::open(&self.cache_path) {
            let cached = store.get_by_name(name, category)?;
            if !cached.is_empty() {
                let docs: Vec<Document> = cached
                    .iter()
                    .filter_map(|s| serde_json::from_str(s).ok())
                    .collect();
                if !docs.is_empty() {
                    return Ok(docs);
                }
            }
        }

        let mut q = SearchQuery::new().name(name);
        if let Some(cat) = category {
            q = q.category(cat);
        }
        let docs = self.client.search(&q).await?;
        cache_documents(&self.cache_path, &docs);
        Ok(docs)
    }

    /// Show cache status (category counts).
    pub fn cache_status(&self) -> Result<Vec<(String, i64)>> {
        let store = CacheStore::open(&self.cache_path)?;
        store.status()
    }

    /// Purge expired cache entries.
    pub fn purge_expired(&self) -> Result<usize> {
        let store = CacheStore::open(&self.cache_path)?;
        store.purge_expired()
    }
}

/// Opportunistically cache documents (best-effort, ignores errors).
/// Skips documents missing id, category, or name.
fn cache_documents(cache_path: &Path, docs: &[Document]) -> usize {
    let Ok(store) = CacheStore::open(cache_path) else {
        return 0;
    };
    let mut count = 0;
    for doc in docs {
        let Some(id) = doc.id.as_deref() else {
            continue;
        };
        let Some(cat) = doc.category.as_deref() else {
            continue;
        };
        let Some(name) = doc.name.as_deref() else {
            continue;
        };
        if let Ok(json) = serde_json::to_string(doc)
            && store.put(id, cat, name, &json).is_ok()
        {
            count += 1;
        }
    }
    count
}
