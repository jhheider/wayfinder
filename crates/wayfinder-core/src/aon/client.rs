use serde_json::Value;
use std::future::Future;
use std::time::Duration;

use super::models::Document;
use super::query::SearchQuery;
use crate::error::{Error, Result};

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
#[derive(Clone)]
pub struct AonClient {
    http: reqwest::Client,
    pub system: GameSystem,
}

impl AonClient {
    pub fn new(system: GameSystem) -> Result<Self> {
        // reqwest is built with rustls-no-provider; install ring process-wide
        // (idempotent -- Err just means a provider is already set).
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = reqwest::Client::builder()
            // Identify ourselves to Archives of Nethys (good citizenship; an
            // unnamed client behind Cloudflare is the first thing throttled).
            .user_agent(concat!(
                "wayfinder-core/",
                env!("CARGO_PKG_VERSION"),
                " (+https://github.com/jhheider/wayfinder)"
            ))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { http, system })
    }

    /// Execute a search query and return parsed documents.
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<Document>> {
        let body = query.build();
        let raw = self.search_raw(&body).await?;
        parse_documents(&raw)
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
                return Err(Error::HttpStatus {
                    status: status.as_u16(),
                });
            }
            return Ok(resp.json().await?);
        }

        if !status.is_success() {
            return Err(Error::HttpStatus {
                status: status.as_u16(),
            });
        }
        Ok(resp.json().await?)
    }

    async fn do_post(&self, url: &str, body: &Value) -> Result<reqwest::Response> {
        Ok(self.http.post(url).json(body).send().await?)
    }
}

impl SearchClient for AonClient {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<Document>> {
        AonClient::search(self, query).await
    }
}

/// Parse the `_source` of every hit in a raw AON/Elasticsearch response into
/// [`Document`]s. Public so consumers that post custom query bodies via
/// [`AonClient::search_raw`] can reuse the same parsing.
pub fn parse_documents(response: &Value) -> Result<Vec<Document>> {
    let hits = response
        .pointer("/hits/hits")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Error::UnexpectedResponse("missing hits.hits".to_string()))?;

    hits.iter()
        .filter_map(|hit| hit.get("_source"))
        .map(|src| serde_json::from_value::<Document>(src.clone()).map_err(Error::from))
        .collect()
}

/// Total number of matches reported by a raw AON/Elasticsearch response
/// (`hits.total.value`), independent of how many hits were returned.
pub fn parse_total(response: &Value) -> Option<i64> {
    response
        .pointer("/hits/total/value")
        .and_then(serde_json::Value::as_i64)
}
