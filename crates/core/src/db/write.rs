//! Database writer — single-writer with batched transactions.
//!
//! Per `12-DATABASE-SCHEMA.md` §12.1 (Design Principle 1 & 3) and
//! `05-RUST-CORE-ENGINE.md` §5.3.4.
//!
//! # Pattern
//!
//! - The `DbWriter` owns a single `Connection`. It is the ONLY writer.
//! - Batches are enqueued via `enqueue(...)`. The writer accumulates batches
//!   and commits a transaction every 250 ms OR every 5000 rows (whichever first).
//! - WAL mode allows concurrent readers while writing.
//! - The DB is a cache, not authority. If the DB is corrupt, the app re-scans.

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use rusqlite::{params, Connection};

use super::schema::DbError;

/// One row to insert into the `files` table.
#[derive(Debug, Clone)]
pub struct WriteRow {
    pub scan_id: String,
    pub path_id: i64,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub mtime: i64,
    pub ctime: i64,
    pub attrs: u32,
    pub reparse_tag: u32,
    pub reparse_kind: &'static str,
    pub hardlink_count: u16,
    pub sparse: bool,
    pub file_id: u64,
    pub extension_id: Option<i64>,
}

impl WriteRow {
    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        // ORDER MUST MATCH the INSERT statement in `flush_buffer`.
        vec![
            &self.scan_id,
            &self.path_id,
            &self.logical_size,
            &self.allocated_size,
            &self.mtime,
            &self.ctime,
            &self.attrs,
            &self.reparse_tag,
            &self.reparse_kind,
            &self.hardlink_count,
            &self.sparse,
            &self.file_id,
            &self.extension_id,
        ]
    }
}

/// A batch of rows to be inserted together.
#[derive(Debug, Default)]
pub struct WriteBatch {
    pub rows: Vec<WriteRow>,
}

impl WriteBatch {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, row: WriteRow) {
        self.rows.push(row);
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// The single-writer DB writer. Owns one `Connection`. Per `05-RUST-CORE-ENGINE.md` §5.3.4.
pub struct DbWriter {
    conn: Arc<Mutex<Connection>>,
    buffer: Mutex<Vec<WriteRow>>,
    last_flush: Mutex<Instant>,
    flush_interval: Duration,
    flush_row_threshold: usize,
}

impl DbWriter {
    /// Open the writer at the given DB path. Runs migrations on first open.
    pub fn open(path: &std::path::Path) -> Result<Self, DbError> {
        let conn = super::schema::open(path)?;
        super::migrate::migrate(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            buffer: Mutex::new(Vec::with_capacity(8192)),
            last_flush: Mutex::new(Instant::now()),
            flush_interval: Duration::from_millis(250),
            flush_row_threshold: 5000,
        })
    }

    /// Open a writer over an existing (already-migrated) connection.
    pub fn from_connection(conn: Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            buffer: Mutex::new(Vec::with_capacity(8192)),
            last_flush: Mutex::new(Instant::now()),
            flush_interval: Duration::from_millis(250),
            flush_row_threshold: 5000,
        }
    }

    /// Enqueue a batch of rows for writing.
    /// Returns true if the buffer was flushed after this enqueue.
    pub fn enqueue(&self, batch: WriteBatch) -> Result<bool, DbError> {
        if batch.is_empty() {
            return Ok(false);
        }
        let should_flush = {
            let mut buf = self.buffer.lock();
            buf.extend(batch.rows);
            let threshold_reached = buf.len() >= self.flush_row_threshold;
            let time_reached = self.last_flush.lock().elapsed() >= self.flush_interval;
            threshold_reached || time_reached
        };
        if should_flush {
            self.flush()?;
            *self.last_flush.lock() = Instant::now();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Force a flush of any buffered rows.
    pub fn flush(&self) -> Result<(), DbError> {
        let mut buf = self.buffer.lock();
        if buf.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock();
        let drained: Vec<_> = std::mem::take(&mut *buf);
        drop(buf); // release the lock early so other threads can enqueue

        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = conn.prepare(
                "INSERT OR REPLACE INTO files
                 (scan_id, path_id, logical_size, allocated_size, mtime, ctime,
                  attrs, reparse_tag, reparse_kind, hardlink_count, sparse, file_id, extension_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            )?;
            for row in &drained {
                let p = row.to_params();
                stmt.execute(rusqlite::params_from_iter(p.iter()))?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Get a handle to the underlying connection (read-only access).
    /// Per `12-DATABASE-SCHEMA.md` §12.1, only the writer thread should mutate
    /// via this connection; for reads, open a separate read-only connection.
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrate::migrate;
    use crate::db::schema::open_in_memory;

    fn setup() -> DbWriter {
        let conn = open_in_memory().expect("open");
        migrate(&conn).expect("migrate");
        // Insert a fake volume + scan + root path so foreign-key constraints are satisfied.
        conn.execute(
            "INSERT INTO volumes (id, guid, serial, label, drive_letters, fs_type, total_bytes, free_bytes, first_seen_at, last_seen_at) VALUES (1, 'v1', 1, 'test', 'C:', 'NTFS', 1000000, 500000, 0, 0)",
            [],
        ).expect("vol");
        conn.execute(
            "INSERT INTO scans (id, volume_id, root_path, started_at, options_json, status) VALUES ('scan-1', 1, 'C:\\', 0, '{}', 'running')",
            [],
        ).expect("scan");
        // Root path: parent_id is NULL (no parent for the root).
        conn.execute(
            "INSERT INTO paths (id, volume_id, parent_id, name, depth, is_dir) VALUES (1, 1, NULL, 'C:', 0, 1)",
            [],
        ).expect("path");
        DbWriter::from_connection(conn)
    }

    #[test]
    fn writes_a_row() {
        let writer = setup();
        // Insert a child path under the root (id=1) so the FK is satisfied.
        writer.connection().lock().execute(
            "INSERT INTO paths (id, volume_id, parent_id, name, depth, is_dir) VALUES (10, 1, 1, 'a.txt', 1, 0)",
            [],
        ).expect("child path");
        let mut batch = WriteBatch::new();
        batch.push(WriteRow {
            scan_id: "scan-1".to_string(),
            path_id: 10,
            logical_size: 100,
            allocated_size: 100,
            mtime: 0,
            ctime: 0,
            attrs: 0,
            reparse_tag: 0,
            reparse_kind: "none",
            hardlink_count: 1,
            sparse: false,
            file_id: 0,
            extension_id: None,
        });
        writer.enqueue(batch).expect("enqueue");
        writer.flush().expect("flush");

        let count: i64 = writer
            .connection()
            .lock()
            .query_row(
                "SELECT count(*) FROM files WHERE scan_id = 'scan-1'",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(count, 1);
    }

    #[test]
    fn batched_writes_merge() {
        let writer = setup();
        for i in 0..3 {
            // Each iteration creates a distinct child path row first (under root id=1).
            writer.connection().lock().execute(
                "INSERT OR REPLACE INTO paths (id, volume_id, parent_id, name, depth, is_dir) VALUES (?, 1, 1, ?, 1, 0)",
                params![i + 100, format!("file-{i}")],
            ).expect("path");
            let mut b = WriteBatch::new();
            b.push(WriteRow {
                scan_id: "scan-1".to_string(),
                path_id: i + 100,
                logical_size: i as u64 * 10,
                allocated_size: i as u64 * 10,
                mtime: i as i64,
                ctime: 0,
                attrs: 0,
                reparse_tag: 0,
                reparse_kind: "none",
                hardlink_count: 1,
                sparse: false,
                file_id: 0,
                extension_id: None,
            });
            writer.enqueue(b).expect("enqueue");
        }
        writer.flush().expect("flush");

        let count: i64 = writer
            .connection()
            .lock()
            .query_row(
                "SELECT count(*) FROM files WHERE scan_id = 'scan-1'",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(count, 3);
    }
}
