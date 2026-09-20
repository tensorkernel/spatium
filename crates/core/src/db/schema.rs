//! SQLite connection setup.
//!
//! Per `12-DATABASE-SCHEMA.md` §12.2.

use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("migration failed: {0}")]
    Migration(String),
}

/// Open a SQLite connection at `path` with WAL mode and tuned pragmas.
///
/// Per `12-DATABASE-SCHEMA.md` §12.2.
pub fn open(path: &Path) -> Result<Connection, DbError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    conn.pragma_update(None, "mmap_size", 268_435_456_i64)?; // 256 MB mmap
    conn.pragma_update(None, "cache_size", -65_536_i64)?; // 64 MB page cache
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "encoding", "UTF-8")?;
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(conn)
}

/// Open an in-memory SQLite database (for tests).
pub fn open_in_memory() -> Result<Connection, DbError> {
    let conn = Connection::open_in_memory()?;
    conn.pragma_update(None, "journal_mode", "MEMORY")?;
    conn.pragma_update(None, "synchronous", "OFF")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "encoding", "UTF-8")?;
    Ok(conn)
}
