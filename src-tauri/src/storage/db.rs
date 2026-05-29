//! SQLite-backed usage logging via `rusqlite`.
//!
//! Every completed enhancement request is recorded in the `usage_log` table so
//! the Settings → Usage view can show per-month and per-provider cost. Writes
//! are intended to be fire-and-forget (spawned off the user-facing path) and a
//! logging failure must never surface as an enhancement error.

use crate::error::AppError;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

/// A single usage record. Note: **no text content is stored** — only metadata —
/// to keep the log privacy-preserving (see `docs/PRIVACY.md`).
#[derive(Debug, Clone, PartialEq)]
pub struct UsageEntry {
    /// RFC 3339 UTC timestamp.
    pub timestamp: String,
    pub mode: String,
    pub provider: String,
    pub tokens: u32,
    pub cost_usd: f64,
    pub input_len: u32,
    pub output_len: u32,
}

/// Aggregated per-provider usage for a period.
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderUsage {
    pub provider: String,
    pub requests: u32,
    pub tokens: u32,
    pub cost_usd: f64,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Opens (creating if necessary) the database at `path` and ensures the
    /// schema exists.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let conn = Connection::open(path).map_err(|e| AppError::Storage(e.to_string()))?;
        Self::from_conn(conn)
    }

    /// Opens an in-memory database (used by tests).
    pub fn open_in_memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory().map_err(|e| AppError::Storage(e.to_string()))?;
        Self::from_conn(conn)
    }

    fn from_conn(conn: Connection) -> Result<Self, AppError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS usage_log (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp  TEXT    NOT NULL,
                mode       TEXT    NOT NULL,
                provider   TEXT    NOT NULL,
                tokens     INTEGER NOT NULL,
                cost_usd   REAL    NOT NULL,
                input_len  INTEGER NOT NULL,
                output_len INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_usage_timestamp ON usage_log(timestamp);",
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Inserts a usage record.
    pub fn log_usage(&self, e: &UsageEntry) -> Result<(), AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| AppError::Storage("usage database lock poisoned".to_string()))?;
        conn.execute(
            "INSERT INTO usage_log
                (timestamp, mode, provider, tokens, cost_usd, input_len, output_len)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                e.timestamp,
                e.mode,
                e.provider,
                e.tokens,
                e.cost_usd,
                e.input_len,
                e.output_len,
            ],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Total estimated cost (USD) for rows whose timestamp starts with
    /// `year_month` (e.g. `"2026-05"`).
    pub fn monthly_total_usd(&self, year_month: &str) -> Result<f64, AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| AppError::Storage("usage database lock poisoned".to_string()))?;
        let pattern = format!("{year_month}%");
        let total: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(cost_usd), 0.0) FROM usage_log WHERE timestamp LIKE ?1",
                [pattern],
                |row| row.get(0),
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(total)
    }

    /// Per-provider breakdown for rows whose timestamp starts with `year_month`.
    pub fn provider_breakdown(&self, year_month: &str) -> Result<Vec<ProviderUsage>, AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| AppError::Storage("usage database lock poisoned".to_string()))?;
        let pattern = format!("{year_month}%");
        let mut stmt = conn
            .prepare(
                "SELECT provider, COUNT(*), COALESCE(SUM(tokens),0), COALESCE(SUM(cost_usd),0.0)
                 FROM usage_log WHERE timestamp LIKE ?1
                 GROUP BY provider ORDER BY provider",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([pattern], |row| {
                Ok(ProviderUsage {
                    provider: row.get(0)?,
                    requests: row.get(1)?,
                    tokens: row.get(2)?,
                    cost_usd: row.get(3)?,
                })
            })
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| AppError::Storage(e.to_string()))?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(month: &str, provider: &str, tokens: u32, cost: f64) -> UsageEntry {
        UsageEntry {
            timestamp: format!("{month}-15T10:00:00Z"),
            mode: "fix_grammar".into(),
            provider: provider.into(),
            tokens,
            cost_usd: cost,
            input_len: 100,
            output_len: 90,
        }
    }

    #[test]
    fn test_log_and_monthly_total() {
        let db = Database::open_in_memory().unwrap();
        db.log_usage(&entry("2026-05", "openai", 100, 0.01))
            .unwrap();
        db.log_usage(&entry("2026-05", "groq", 200, 0.0)).unwrap();
        db.log_usage(&entry("2026-04", "openai", 50, 0.99)).unwrap();

        let may = db.monthly_total_usd("2026-05").unwrap();
        assert!(
            (may - 0.01).abs() < 1e-9,
            "May total should be 0.01, got {may}"
        );
        let apr = db.monthly_total_usd("2026-04").unwrap();
        assert!((apr - 0.99).abs() < 1e-9);
    }

    #[test]
    fn test_provider_breakdown() {
        let db = Database::open_in_memory().unwrap();
        db.log_usage(&entry("2026-05", "openai", 100, 0.01))
            .unwrap();
        db.log_usage(&entry("2026-05", "openai", 100, 0.02))
            .unwrap();
        db.log_usage(&entry("2026-05", "groq", 200, 0.0)).unwrap();

        let breakdown = db.provider_breakdown("2026-05").unwrap();
        assert_eq!(breakdown.len(), 2);
        let openai = breakdown.iter().find(|p| p.provider == "openai").unwrap();
        assert_eq!(openai.requests, 2);
        assert_eq!(openai.tokens, 200);
        assert!((openai.cost_usd - 0.03).abs() < 1e-9);
    }

    #[test]
    fn test_empty_month_is_zero() {
        let db = Database::open_in_memory().unwrap();
        assert_eq!(db.monthly_total_usd("2099-01").unwrap(), 0.0);
        assert!(db.provider_breakdown("2099-01").unwrap().is_empty());
    }
}
