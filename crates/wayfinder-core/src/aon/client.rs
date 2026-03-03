use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::future::Future;
use std::time::Duration;

use super::models::Document;
use super::query::SearchQuery;

/// Trait for searching AON documents, enabling mock implementations for tests.
pub trait SearchClient: Send + Sync {
    fn search(&self, query: &SearchQuery) -> impl Future<Output = Result<Vec<Document>>> + Send;
}

/// Game system endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameSystem {
    #[default]
    Pathfinder,
    Starfinder,
}

impl GameSystem {
    pub fn endpoint(&self) -> &'static str {
        match self {
            Self::Pathfinder => "https://elasticsearch.aonprd.com/aon/_search",
            Self::Starfinder => "https://elasticsearch.aonprd.com/aonsf/_search",
        }
    }

    pub fn index(&self) -> &'static str {
        match self {
            Self::Pathfinder => "aon70",
            Self::Starfinder => "aonsf10",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Pathfinder => "PF2e",
            Self::Starfinder => "SF2e",
        }
    }

    pub fn base_url(&self) -> &'static str {
        match self {
            Self::Pathfinder => "https://2e.aonprd.com",
            Self::Starfinder => "https://2e.aonsrd.com",
        }
    }
}

/// HTTP client for AON's Elasticsearch backend.
pub struct AonClient {
    http: reqwest::Client,
    pub system: GameSystem,
}

impl AonClient {
    pub fn new(system: GameSystem) -> Result<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { http, system })
    }

    /// Execute a search query and return parsed documents.
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<Document>> {
        let body = query.build();
        let raw = self.search_raw(&body).await?;
        parse_hits(&raw)
    }

    /// Execute a raw JSON query body, returning the full response.
    /// Retries once on 429/503 after a short delay.
    pub async fn search_raw(&self, body: &Value) -> Result<Value> {
        let url = format!("{}?index={}", self.system.endpoint(), self.system.index());
        let resp = self.do_post(&url, body).await?;
        let status = resp.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS
            || status == reqwest::StatusCode::SERVICE_UNAVAILABLE
        {
            let delay = resp
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(2)
                .min(30);
            tokio::time::sleep(Duration::from_secs(delay)).await;
            let resp = self.do_post(&url, body).await?;
            let status = resp.status();
            if !status.is_success() {
                bail!("AON returned HTTP {status} after retry");
            }
            return resp.json().await.context("Failed to parse AON response");
        }

        if !status.is_success() {
            bail!("AON returned HTTP {status}");
        }
        resp.json().await.context("Failed to parse AON response")
    }

    async fn do_post(&self, url: &str, body: &Value) -> Result<reqwest::Response> {
        self.http
            .post(url)
            .json(body)
            .send()
            .await
            .context("AON request failed")
    }
}

impl SearchClient for AonClient {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<Document>> {
        AonClient::search(self, query).await
    }
}

fn parse_hits(response: &Value) -> Result<Vec<Document>> {
    let hits = response
        .pointer("/hits/hits")
        .and_then(|v| v.as_array())
        .context("Missing hits.hits in response")?;

    hits.iter()
        .filter_map(|hit| hit.get("_source"))
        .map(|src| {
            serde_json::from_value::<Document>(src.clone()).context("Failed to parse document")
        })
        .collect()
}
