use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// SQLite-backed document cache.
pub struct CacheStore {
    conn: Connection,
    ttl_secs: i64,
}

impl CacheStore {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open cache DB")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                data TEXT NOT NULL,
                fetched_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_cat_name ON documents(category, name);",
        )
        .context("Failed to init cache schema")?;
        Ok(Self {
            conn,
            ttl_secs: 86400 * 7,
        })
    }

    pub fn set_ttl(&mut self, secs: i64) {
        self.ttl_secs = secs;
    }

    pub fn get(&self, id: &str) -> Result<Option<String>> {
        let cutoff = now_epoch() - self.ttl_secs;
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM documents WHERE id = ?1 AND fetched_at > ?2")?;
        match stmt.query_row(params![id, cutoff], |row| row.get::<_, String>(0)) {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to read from cache"),
        }
    }

    pub fn get_by_category(&self, category: &str) -> Result<Vec<String>> {
        let cutoff = now_epoch() - self.ttl_secs;
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM documents WHERE category = ?1 AND fetched_at > ?2")?;
        let rows = stmt
            .query_map(params![category, cutoff], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn put(&self, id: &str, category: &str, name: &str, data: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO documents (id, category, name, data, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, category, name, data, now_epoch()],
        )?;
        Ok(())
    }

    pub fn bulk_put(&mut self, docs: &[(String, String, String, String)]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;
        for (id, category, name, data) in docs {
            tx.execute(
                "INSERT OR REPLACE INTO documents (id, category, name, data, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, category, name, data, now_epoch()],
            )?;
            count += 1;
        }
        tx.commit()?;
        Ok(count)
    }

    /// Look up documents by name (case-insensitive), optionally scoped to a category.
    pub fn get_by_name(&self, name: &str, category: Option<&str>) -> Result<Vec<String>> {
        let cutoff = now_epoch() - self.ttl_secs;
        let (sql, params_vec): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match category {
            Some(cat) => (
                "SELECT data FROM documents WHERE name = ?1 COLLATE NOCASE AND category = ?2 AND fetched_at > ?3",
                vec![
                    Box::new(name.to_string()),
                    Box::new(cat.to_string()),
                    Box::new(cutoff),
                ],
            ),
            None => (
                "SELECT data FROM documents WHERE name = ?1 COLLATE NOCASE AND fetched_at > ?2",
                vec![Box::new(name.to_string()), Box::new(cutoff)],
            ),
        };
        let mut stmt = self.conn.prepare(sql)?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(params_refs.as_slice(), |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// Delete expired documents from the cache.
    pub fn purge_expired(&self) -> Result<usize> {
        let cutoff = now_epoch() - self.ttl_secs;
        let deleted = self.conn.execute(
            "DELETE FROM documents WHERE fetched_at <= ?1",
            params![cutoff],
        )?;
        Ok(deleted)
    }

    pub fn status(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT category, COUNT(*) FROM documents GROUP BY category ORDER BY category",
        )?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }
}

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_secs() as i64
}
