//! A persistent cache of AON responses, keyed by the exact request.
//!
//! Every search, lookup and category listing goes through it, so repeat
//! questions (common from an LLM) never reach Nethys twice within the TTL.
//! One SQLite file is shared by every wayfinder process (`wf`, the MCP
//! server); WAL mode lets them read and write it at the same time.

use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::Result;

/// Environment variable: a cache file path, or `off` to disable caching.
pub const CACHE_ENV: &str = "WAYFINDER_CACHE";

/// How long a response stays fresh. AON changes when books release, not by
/// the hour; a day keeps errata reasonably current.
pub const DEFAULT_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Writes delete expired rows at most this often (per handle), so a
/// long-running server's cache stays bounded without `wf cache purge`.
const PURGE_EVERY_SECS: i64 = 60 * 60;

/// Cached AON responses in SQLite.
pub struct ResponseCache {
    conn: Mutex<Connection>,
    path: PathBuf,
    ttl_secs: i64,
    /// When this handle last deleted expired rows (0: never).
    last_purge: AtomicI64,
}

impl ResponseCache {
    /// Open (creating if needed) the cache at `path`.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            // Best effort: if this fails, Connection::open reports it.
            let _ = std::fs::create_dir_all(dir);
        }
        let conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_secs(2))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(
            // `documents` was the pre-0.2 per-document cache; its rows are
            // superseded by cached responses.
            "DROP TABLE IF EXISTS documents;
             CREATE TABLE IF NOT EXISTS responses (
                 key TEXT PRIMARY KEY,
                 game TEXT NOT NULL,
                 body TEXT NOT NULL,
                 fetched_at INTEGER NOT NULL
             );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: path.to_path_buf(),
            ttl_secs: DEFAULT_TTL.as_secs() as i64,
            last_purge: AtomicI64::new(0),
        })
    }

    /// The cache file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Use a different freshness window.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl_secs = ttl.as_secs() as i64;
        self
    }

    /// The per-user cache file: `<data dir>/wayfinder/wayfinder_cache.db`.
    pub fn default_path() -> Option<PathBuf> {
        Some(
            dirs::data_local_dir()?
                .join("wayfinder")
                .join("wayfinder_cache.db"),
        )
    }

    /// The cache [`CACHE_ENV`] asks for: `None` when it is `off` (or `0`,
    /// `false`, `none`), the file it names, or else [`Self::default_path`].
    pub fn from_env() -> Result<Option<Self>> {
        let path = match std::env::var(CACHE_ENV) {
            Ok(v) if is_off(&v) => return Ok(None),
            Ok(v) if !v.trim().is_empty() => PathBuf::from(v.trim()),
            _ => match Self::default_path() {
                Some(p) => p,
                None => return Ok(None),
            },
        };
        Self::open(&path).map(Some)
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        // A panic mid-statement cannot corrupt SQLite; keep using it.
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn cutoff(&self) -> i64 {
        now() - self.ttl_secs
    }

    /// The fresh response stored under `key`, if any. Read errors count as a
    /// miss: the cache is an optimization, never a reason to fail.
    pub fn get(&self, key: &str) -> Option<Value> {
        let body: Option<String> = self
            .conn()
            .query_row(
                "SELECT body FROM responses WHERE key = ?1 AND fetched_at > ?2",
                params![key, self.cutoff()],
                |row| row.get(0),
            )
            .optional()
            .ok()
            .flatten();
        serde_json::from_str(&body?).ok()
    }

    /// Store `response` under `key` for `game` (a label for [`Self::status`]).
    /// The first write, and then one an hour, also deletes expired rows.
    pub fn put(&self, key: &str, game: &str, response: &Value) -> Result<()> {
        let now = now();
        let last = self.last_purge.load(Ordering::Relaxed);
        if now - last >= PURGE_EVERY_SECS
            && self
                .last_purge
                .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
        {
            // Best effort, like every cache write.
            let _ = self.purge_expired();
        }
        self.conn().execute(
            "INSERT OR REPLACE INTO responses (key, game, body, fetched_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![key, game, response.to_string(), now],
        )?;
        Ok(())
    }

    /// Fresh entries per game, e.g. `[("PF2e", 12)]`.
    pub fn status(&self) -> Result<Vec<(String, i64)>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT game, COUNT(*) FROM responses WHERE fetched_at > ?1
             GROUP BY game ORDER BY game",
        )?;
        let rows = stmt
            .query_map(params![self.cutoff()], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows)
    }

    /// Delete expired entries; returns how many.
    pub fn purge_expired(&self) -> Result<usize> {
        let conn = self.conn();
        let n = conn.execute(
            "DELETE FROM responses WHERE fetched_at <= ?1",
            params![self.cutoff()],
        )?;
        Ok(n)
    }

    /// Delete every entry; returns how many.
    pub fn clear(&self) -> Result<usize> {
        Ok(self.conn().execute("DELETE FROM responses", [])?)
    }
}

fn is_off(v: &str) -> bool {
    matches!(
        v.trim().to_ascii_lowercase().as_str(),
        "off" | "0" | "false" | "none" | "no"
    )
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}
