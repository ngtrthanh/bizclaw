//! SQLite memory backend with FTS5 full-text search and session support.

use async_trait::async_trait;
use bizclaw_core::error::Result;
use bizclaw_core::traits::memory::{MemoryBackend, MemoryEntry, MemorySearchResult};
use rusqlite::Connection;
use std::sync::Mutex;

pub struct SqliteMemory {
    conn: Mutex<Connection>,
}

impl SqliteMemory {
    pub fn new() -> Result<Self> {
        let db_path = bizclaw_core::config::BizClawConfig::home_dir().join("memory.db");
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&db_path)
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        // Main table with session support
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                session_id TEXT DEFAULT 'default',
                content TEXT NOT NULL,
                metadata TEXT DEFAULT '{}',
                embedding BLOB,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )
        .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        // Add session_id column if missing (migration from old schema)
        conn.execute_batch("ALTER TABLE memories ADD COLUMN session_id TEXT DEFAULT 'default';")
            .ok(); // Silently ignore if column already exists

        // FTS5 virtual table for fast full-text search with BM25 ranking
        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                id UNINDEXED,
                content,
                tokenize='unicode61'
            );",
        )
        .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        // Sessions table for tracking conversation threads
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now')),
                message_count INTEGER DEFAULT 0,
                summary TEXT DEFAULT ''
            );",
        )
        .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        // Ensure default session exists
        conn.execute(
            "INSERT OR IGNORE INTO sessions (id, name) VALUES ('default', 'Default')",
            [],
        )
        .ok();

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Get conversation count across all sessions.
    pub fn conversation_count(&self) -> usize {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get::<_, i64>(0))
            .unwrap_or(0) as usize
    }

    /// List sessions with their message counts.
    pub fn list_sessions(&self) -> Vec<(String, String, i64)> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, message_count FROM sessions ORDER BY updated_at DESC")
            .unwrap();
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    }

    /// Create a new session.
    pub fn create_session(&self, id: &str, name: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;
        conn.execute(
            "INSERT OR IGNORE INTO sessions (id, name) VALUES (?1, ?2)",
            rusqlite::params![id, name],
        )
        .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl MemoryBackend for SqliteMemory {
    fn name(&self) -> &str {
        "sqlite"
    }

    async fn save(&self, entry: MemoryEntry) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        // Extract session_id from metadata or use default
        let session_id = entry
            .metadata
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();

        conn.execute(
            "INSERT OR REPLACE INTO memories (id, session_id, content, metadata, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                entry.id,
                session_id,
                entry.content,
                entry.metadata.to_string(),
                entry.created_at.to_rfc3339(),
                entry.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        // Index in FTS5 for fast search
        conn.execute(
            "INSERT OR REPLACE INTO memories_fts (id, content) VALUES (?1, ?2)",
            rusqlite::params![entry.id, entry.content],
        )
        .ok(); // Don't fail on FTS insert error

        // Update session message count
        conn.execute(
            "UPDATE sessions SET message_count = message_count + 1, updated_at = datetime('now') WHERE id = ?1",
            rusqlite::params![session_id],
        ).ok();

        Ok(())
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemorySearchResult>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        // Clean query for FTS5
        let clean_query: String = query
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '_')
            .collect();

        if clean_query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Try FTS5 search first (faster, better ranking)
        let fts_results = {
            let mut stmt = conn.prepare(
                "SELECT f.id, m.content, m.metadata, m.created_at, m.updated_at, bm25(memories_fts) as score
                 FROM memories_fts f
                 JOIN memories m ON m.id = f.id
                 WHERE memories_fts MATCH ?1
                 ORDER BY score
                 LIMIT ?2"
            );
            match stmt {
                Ok(ref mut s) => {
                    let rows = s.query_map(rusqlite::params![clean_query, limit as i64], |row| {
                        Ok(MemorySearchResult {
                            entry: MemoryEntry {
                                id: row.get(0)?,
                                content: row.get(1)?,
                                metadata: row
                                    .get::<_, String>(2)
                                    .map(|s| serde_json::from_str(&s).unwrap_or_default())
                                    .unwrap_or_default(),
                                embedding: None,
                                created_at: row
                                    .get::<_, String>(3)
                                    .map(|s| {
                                        chrono::DateTime::parse_from_rfc3339(&s)
                                            .map(|d| d.with_timezone(&chrono::Utc))
                                            .unwrap_or_default()
                                    })
                                    .unwrap_or_default(),
                                updated_at: row
                                    .get::<_, String>(4)
                                    .map(|s| {
                                        chrono::DateTime::parse_from_rfc3339(&s)
                                            .map(|d| d.with_timezone(&chrono::Utc))
                                            .unwrap_or_default()
                                    })
                                    .unwrap_or_default(),
                            },
                            score: row.get::<_, f32>(5).unwrap_or(0.0).abs(), // BM25 returns negative scores
                        })
                    });
                    match rows {
                        Ok(r) => r.filter_map(|r| r.ok()).collect::<Vec<_>>(),
                        Err(_) => Vec::new(),
                    }
                }
                Err(_) => Vec::new(),
            }
        };

        // If FTS5 returned results, use them
        if !fts_results.is_empty() {
            return Ok(fts_results);
        }

        // Fallback to LIKE search (for queries that don't work well with FTS5)
        let mut stmt = conn.prepare(
            "SELECT id, content, metadata, created_at, updated_at FROM memories WHERE content LIKE ?1 ORDER BY created_at DESC LIMIT ?2"
        ).map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        let pattern = format!("%{}%", query.to_lowercase());
        let query_lower = query.to_lowercase();
        let rows = stmt
            .query_map(rusqlite::params![pattern, limit], |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    metadata: row
                        .get::<_, String>(2)
                        .map(|s| serde_json::from_str(&s).unwrap_or_default())
                        .unwrap_or_default(),
                    embedding: None,
                    created_at: row
                        .get::<_, String>(3)
                        .map(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .map(|d| d.with_timezone(&chrono::Utc))
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                    updated_at: row
                        .get::<_, String>(4)
                        .map(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .map(|d| d.with_timezone(&chrono::Utc))
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                })
            })
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        let results: Vec<MemorySearchResult> = rows
            .filter_map(|r| r.ok())
            .map(|entry| {
                let content_lower = entry.content.to_lowercase();
                let matches = content_lower.matches(&query_lower).count();
                let score = (matches as f32).min(5.0) / 5.0;
                MemorySearchResult {
                    entry,
                    score: score.max(0.1),
                }
            })
            .collect();
        Ok(results)
    }

    async fn get(&self, id: &str) -> Result<Option<MemoryEntry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, content, metadata, created_at, updated_at FROM memories WHERE id = ?1",
            )
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        let result = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    metadata: row
                        .get::<_, String>(2)
                        .map(|s| serde_json::from_str(&s).unwrap_or_default())
                        .unwrap_or_default(),
                    embedding: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
            })
            .ok();
        Ok(result)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;
        conn.execute("DELETE FROM memories WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;
        conn.execute(
            "DELETE FROM memories_fts WHERE id = ?1",
            rusqlite::params![id],
        )
        .ok();
        Ok(())
    }

    async fn list(&self, limit: Option<usize>) -> Result<Vec<MemoryEntry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;
        let lim = limit.unwrap_or(100) as i64;
        let mut stmt = conn.prepare(
            "SELECT id, content, metadata, created_at, updated_at FROM memories ORDER BY created_at DESC LIMIT ?1"
        ).map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        let results = stmt
            .query_map(rusqlite::params![lim], |row| {
                Ok(MemoryEntry {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    metadata: row
                        .get::<_, String>(2)
                        .map(|s| serde_json::from_str(&s).unwrap_or_default())
                        .unwrap_or_default(),
                    embedding: None,
                    created_at: row
                        .get::<_, String>(3)
                        .map(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .map(|d| d.with_timezone(&chrono::Utc))
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                    updated_at: row
                        .get::<_, String>(4)
                        .map(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .map(|d| d.with_timezone(&chrono::Utc))
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                })
            })
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;

        Ok(results.filter_map(|r| r.ok()).collect())
    }

    async fn clear(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;
        conn.execute("DELETE FROM memories", [])
            .map_err(|e| bizclaw_core::error::BizClawError::Memory(e.to_string()))?;
        conn.execute("DELETE FROM memories_fts", []).ok();
        Ok(())
    }
}
