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
    endpoint: String,
}

impl AonClient {
    pub fn new(system: GameSystem) -> Result<Self> {
        Self::with_endpoint(system, system.endpoint())
    }

    /// Construct a client pointed at a specific `_search` endpoint instead of
    /// the game's default (e.g. a caching proxy, a mirror, or a test server).
    pub fn with_endpoint(system: GameSystem, endpoint: impl Into<String>) -> Result<Self> {
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
        Ok(Self {
            http,
            system,
            endpoint: endpoint.into(),
        })
    }

    /// Execute a search query and return parsed documents.
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<Document>> {
        let body = query.build();
        let raw = self.search_raw(&body).await?;
        parse_documents(&raw)
    }

    /// Execute a raw JSON query body, returning the full response. Retries on
    /// 429/503 with exponential backoff (honoring `Retry-After` when present),
    /// up to [`MAX_ATTEMPTS`] total requests.
    pub async fn search_raw(&self, body: &Value) -> Result<Value> {
        let url = format!("{}?index={}", self.endpoint, self.system.index());
        let mut attempt = 0u32;
        loop {
            let resp = self.do_post(&url, body).await?;
            let status = resp.status();

            let retriable = status == reqwest::StatusCode::TOO_MANY_REQUESTS
                || status == reqwest::StatusCode::SERVICE_UNAVAILABLE;
            if retriable && attempt + 1 < MAX_ATTEMPTS {
                let retry_after = resp
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok());
                tokio::time::sleep(backoff_delay(retry_after, attempt)).await;
                attempt += 1;
                continue;
            }

            if !status.is_success() {
                return Err(Error::HttpStatus {
                    status: status.as_u16(),
                });
            }
            return Ok(resp.json().await?);
        }
    }

    async fn do_post(&self, url: &str, body: &Value) -> Result<reqwest::Response> {
        Ok(self.http.post(url).json(body).send().await?)
    }
}

/// Maximum number of requests per `search_raw` call (initial try plus retries).
const MAX_ATTEMPTS: u32 = 3;

/// Backoff before the next retry. A server-sent `Retry-After` (whole seconds)
/// wins; otherwise exponential from a 500ms base (500ms, 1s, 2s, ...). Capped
/// at 30s either way.
fn backoff_delay(retry_after: Option<&str>, attempt: u32) -> Duration {
    const CAP: Duration = Duration::from_secs(30);
    if let Some(secs) = retry_after.and_then(|v| v.trim().parse::<u64>().ok()) {
        return Duration::from_secs(secs).min(CAP);
    }
    let ms = 500u64.saturating_mul(1u64 << attempt.min(6));
    Duration::from_millis(ms).min(CAP)
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

#[cfg(test)]
mod tests {
    use super::backoff_delay;
    use std::time::Duration;

    #[test]
    fn retry_after_seconds_wins() {
        assert_eq!(backoff_delay(Some("5"), 0), Duration::from_secs(5));
        assert_eq!(backoff_delay(Some(" 3 "), 2), Duration::from_secs(3));
    }

    #[test]
    fn retry_after_is_capped_at_30s() {
        assert_eq!(backoff_delay(Some("120"), 0), Duration::from_secs(30));
    }

    #[test]
    fn exponential_without_retry_after() {
        assert_eq!(backoff_delay(None, 0), Duration::from_millis(500));
        assert_eq!(backoff_delay(None, 1), Duration::from_millis(1000));
        assert_eq!(backoff_delay(None, 2), Duration::from_millis(2000));
    }

    #[test]
    fn exponential_is_capped_at_30s() {
        assert_eq!(backoff_delay(None, 20), Duration::from_secs(30));
    }

    #[test]
    fn invalid_retry_after_falls_back_to_exponential() {
        assert_eq!(backoff_delay(Some("soon"), 0), Duration::from_millis(500));
    }
}
