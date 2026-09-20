//! Database reader — query APIs for the renderer (read-only, untrusted).
//!
//! Per `12-DATABASE-SCHEMA.md` §12.5 (query examples) and §2.8 of
//! `02-SYSTEM-ARCHITECTURE.md` (renderer treats DB as untrusted).

use rusqlite::{params, Connection};

use super::schema::DbError;

#[derive(Debug, Clone)]
pub struct FileRow {
    pub node_id: i64, // path_id in DB
    pub parent_id: i64,
    pub name: String,
    pub is_directory: bool,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub mtime: i64,
    pub ctime: i64,
    pub attrs: u32,
    pub reparse_kind: String,
    pub hardlink_count: u16,
    pub sparse: bool,
    pub extension_id: Option<i64>,
    pub hash_sha256: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScanSummary {
    pub scan_id: String,
    pub root_path: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub duration_ms: Option<i64>,
    pub file_count: Option<i64>,
    pub dir_count: Option<i64>,
    pub total_logical: Option<i64>,
    pub total_allocated: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub scan_id: String,
    pub root_path: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub duration_ms: Option<i64>,
    pub file_count: Option<i64>,
    pub total_logical: Option<i64>,
    pub total_allocated: Option<i64>,
}

/// Read-only queries. Per `12-DATABASE-SCHEMA.md` §12.5.
pub struct DbReader {
    conn: Connection,
}

impl DbReader {
    /// Open a read-only connection.
    pub fn open(path: &std::path::Path) -> Result<Self, DbError> {
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        Ok(Self { conn })
    }

    /// Open a reader over an existing (already-open) connection.
    pub fn from_connection(conn: Connection) -> Self {
        Self { conn }
    }

    /// Top-N largest files in a scan. Per §12.5.1.
    pub fn top_n_largest(&self, scan_id: &str, limit: u32) -> Result<Vec<FileRow>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT f.path_id, COALESCE(p.parent_id, 0), p.name, p.is_dir,
                    f.logical_size, f.allocated_size, f.mtime, f.ctime, f.attrs,
                    f.reparse_kind, f.hardlink_count, f.sparse, f.extension_id, f.hash_sha256
             FROM files f
             JOIN paths p ON f.path_id = p.id
             WHERE f.scan_id = ?
             ORDER BY f.logical_size DESC
             LIMIT ?",
        )?;
        let rows = stmt.query_map(params![scan_id, limit], |row| {
            Ok(FileRow {
                node_id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                is_directory: row.get::<_, i64>(3)? != 0,
                logical_size: row.get::<_, i64>(4)? as u64,
                allocated_size: row.get::<_, i64>(5)? as u64,
                mtime: row.get(6)?,
                ctime: row.get(7)?,
                attrs: row.get::<_, i64>(8)? as u32,
                reparse_kind: row.get(9)?,
                hardlink_count: row.get::<_, i64>(10)? as u16,
                sparse: row.get::<_, i64>(11)? != 0,
                extension_id: row.get(12)?,
                hash_sha256: row.get(13)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    /// Directory children with sizes (aggregated). Per §12.5.2.
    pub fn directory_children(
        &self,
        scan_id: &str,
        parent_id: i64,
    ) -> Result<Vec<FileRow>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT f.path_id, p.parent_id, p.name, p.is_dir,
                    f.logical_size, f.allocated_size, f.mtime, f.ctime, f.attrs,
                    f.reparse_kind, f.hardlink_count, f.sparse, f.extension_id, f.hash_sha256
             FROM files f
             JOIN paths p ON f.path_id = p.id
             WHERE f.scan_id = ? AND p.parent_id = ?",
        )?;
        let rows = stmt.query_map(params![scan_id, parent_id], |row| {
            Ok(FileRow {
                node_id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                is_directory: row.get::<_, i64>(3)? != 0,
                logical_size: row.get::<_, i64>(4)? as u64,
                allocated_size: row.get::<_, i64>(5)? as u64,
                mtime: row.get(6)?,
                ctime: row.get(7)?,
                attrs: row.get::<_, i64>(8)? as u32,
                reparse_kind: row.get(9)?,
                hardlink_count: row.get::<_, i64>(10)? as u16,
                sparse: row.get::<_, i64>(11)? != 0,
                extension_id: row.get(12)?,
                hash_sha256: row.get(13)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    /// List recent scans (descending by started_at). Per §12.5.4 listing scans.
    pub fn list_scans(&self, limit: u32) -> Result<Vec<ScanSummary>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, root_path, started_at, completed_at, duration_ms,
                    file_count, dir_count, total_logical, total_allocated, status
             FROM scans ORDER BY started_at DESC LIMIT ?",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(ScanSummary {
                scan_id: row.get(0)?,
                root_path: row.get(1)?,
                started_at: row.get(2)?,
                completed_at: row.get(3)?,
                duration_ms: row.get(4)?,
                file_count: row.get(5)?,
                dir_count: row.get(6)?,
                total_logical: row.get(7)?,
                total_allocated: row.get(8)?,
                status: row.get(9)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    /// Look up a single scan's metadata. Per §12.5 (history).
    pub fn get_scan(&self, scan_id: &str) -> Result<Option<HistoryEntry>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, root_path, started_at, completed_at, duration_ms,
                    file_count, total_logical, total_allocated
             FROM scans WHERE id = ?",
        )?;
        let mut rows = stmt.query_map(params![scan_id], |row| {
            Ok(HistoryEntry {
                scan_id: row.get(0)?,
                root_path: row.get(1)?,
                started_at: row.get(2)?,
                completed_at: row.get(3)?,
                duration_ms: row.get(4)?,
                file_count: row.get(5)?,
                total_logical: row.get(6)?,
                total_allocated: row.get(7)?,
            })
        })?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    /// File-type breakdown for a scan. Per §12.5.4.
    pub fn file_type_breakdown(&self, scan_id: &str) -> Result<Vec<(String, i64, i64)>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(e.category, 'other') AS cat,
                    COUNT(*) AS cnt,
                    COALESCE(SUM(f.logical_size), 0) AS total
             FROM files f
             LEFT JOIN extensions e ON f.extension_id = e.id
             WHERE f.scan_id = ?
             GROUP BY cat
             ORDER BY total DESC",
        )?;
        let rows = stmt.query_map(params![scan_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    /// Get a reference to the underlying connection (for ad-hoc queries like history diff).
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}
