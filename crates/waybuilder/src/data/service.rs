use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use wayfinder_core::aon::{AonClient, Document, GameSystem, SearchQuery};
use wayfinder_core::search::SearchService;

pub struct DataService {
    search: SearchService<AonClient>,
    memo: HashMap<String, Vec<Document>>,
}

impl DataService {
    pub fn new(system: GameSystem) -> Result<Self> {
        let client = AonClient::new(system)?;
        let cache_path = cache_dir();
        let search = SearchService::new(client, &cache_path);
        Ok(Self {
            search,
            memo: HashMap::new(),
        })
    }

    pub async fn search_category(
        &mut self,
        category: &str,
        filters: &[(String, String)],
    ) -> Result<Vec<Document>> {
        let key = memo_key(category, filters);
        if let Some(cached) = self.memo.get(&key) {
            return Ok(cached.clone());
        }
        let size = if category == "heritage" || category == "feat" {
            500
        } else {
            200
        };
        let mut q = SearchQuery::new().category(category).size(size);
        for (field, value) in filters {
            q = q.filter(field, value);
        }
        let docs = self.search.search(&q).await?;
        self.memo.insert(key, docs.clone());
        Ok(docs)
    }

    pub async fn show(&mut self, name: &str, category: Option<&str>) -> Result<Vec<Document>> {
        let key = format!("show:{}:{}", name, category.unwrap_or(""));
        if let Some(cached) = self.memo.get(&key) {
            return Ok(cached.clone());
        }
        let docs = self.search.show(name, category).await?;
        self.memo.insert(key, docs.clone());
        Ok(docs)
    }
}

fn memo_key(category: &str, filters: &[(String, String)]) -> String {
    let mut key = format!("cat:{category}");
    for (f, v) in filters {
        key.push_str(&format!(":{f}={v}"));
    }
    key
}

fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("waybuilder")
}
