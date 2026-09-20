//! Scan history + diff computation.
//!
//! Per `12-DATABASE-SCHEMA.md` §12.5.3 (diff two scans).

use rusqlite::{params, Connection};

use super::schema::DbError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanDiff {
    pub prior_scan_id: String,
    pub new_scan_id: String,
    pub added_files: u64,
    pub removed_files: u64,
    pub modified_files: u64,
    pub delta_bytes: i64,
}

impl ScanDiff {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added_files == 0 && self.removed_files == 0 && self.modified_files == 0
    }
}

/// Compute the diff between two scans of the same volume.
///
/// Per `12-DATABASE-SCHEMA.md` §12.5.3.
///
/// Returns:
/// - `Ok(Some(diff))` if both scans exist.
/// - `Ok(None)` if either scan is missing.
pub fn compute_diff(
    conn: &Connection,
    prior_scan_id: &str,
    new_scan_id: &str,
) -> Result<Option<ScanDiff>, DbError> {
    // Both scans must exist.
    let prior_exists: i64 = conn.query_row(
        "SELECT count(*) FROM scans WHERE id = ?",
        params![prior_scan_id],
        |r| r.get(0),
    )?;
    let new_exists: i64 = conn.query_row(
        "SELECT count(*) FROM scans WHERE id = ?",
        params![new_scan_id],
        |r| r.get(0),
    )?;
    if prior_exists == 0 || new_exists == 0 {
        return Ok(None);
    }

    // Files in the new scan not in the prior (added)
    let added: i64 = conn.query_row(
        "SELECT count(*) FROM files f
         WHERE f.scan_id = ?1
           AND NOT EXISTS (
             SELECT 1 FROM files p WHERE p.scan_id = ?2 AND p.path_id = f.path_id
           )",
        params![new_scan_id, prior_scan_id],
        |r| r.get(0),
    )?;

    // Files in the prior scan not in the new (removed)
    let removed: i64 = conn.query_row(
        "SELECT count(*) FROM files p
         WHERE p.scan_id = ?1
           AND NOT EXISTS (
             SELECT 1 FROM files f WHERE f.scan_id = ?2 AND f.path_id = p.path_id
           )",
        params![prior_scan_id, new_scan_id],
        |r| r.get(0),
    )?;

    // Files in both but size changed
    let modified: i64 = conn.query_row(
        "SELECT count(*) FROM files f
         JOIN files p ON f.path_id = p.path_id
         WHERE f.scan_id = ?1 AND p.scan_id = ?2
           AND f.logical_size != p.logical_size",
        params![new_scan_id, prior_scan_id],
        |r| r.get(0),
    )?;

    // Size delta (new total - prior total)
    let new_total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(logical_size), 0) FROM files WHERE scan_id = ?1",
        params![new_scan_id],
        |r| r.get(0),
    )?;
    let prior_total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(logical_size), 0) FROM files WHERE scan_id = ?1",
        params![prior_scan_id],
        |r| r.get(0),
    )?;

    Ok(Some(ScanDiff {
        prior_scan_id: prior_scan_id.to_string(),
        new_scan_id: new_scan_id.to_string(),
        added_files: added as u64,
        removed_files: removed as u64,
        modified_files: modified as u64,
        delta_bytes: new_total - prior_total,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrate::migrate;
    use crate::db::schema::open_in_memory;

    fn seed_two_scans(conn: &Connection) {
        // Volume + root path + child paths + scans + files for two scans.
        conn.execute(
            "INSERT INTO volumes (id, guid, serial, label, drive_letters, fs_type, total_bytes, free_bytes, first_seen_at, last_seen_at)
             VALUES (1, 'v1', 1, 'test', 'C:', 'NTFS', 1000000, 500000, 0, 0)",
            [],
        ).expect("vol");
        // Root path with NULL parent (no FK violation).
        conn.execute(
            "INSERT INTO paths (id, volume_id, parent_id, name, depth, is_dir) VALUES (0, 1, NULL, 'C:', 0, 1)",
            [],
        ).expect("root");
        // Child paths under root id=0.
        conn.execute(
            "INSERT INTO paths (id, volume_id, parent_id, name, depth, is_dir) VALUES (1, 1, 0, 'a.txt', 1, 0)",
            [],
        ).expect("p1");
        conn.execute(
            "INSERT INTO paths (id, volume_id, parent_id, name, depth, is_dir) VALUES (2, 1, 0, 'b.txt', 1, 0)",
            [],
        ).expect("p2");
        conn.execute(
            "INSERT INTO paths (id, volume_id, parent_id, name, depth, is_dir) VALUES (3, 1, 0, 'c.txt', 1, 0)",
            [],
        ).expect("p3");
        conn.execute(
            "INSERT INTO paths (id, volume_id, parent_id, name, depth, is_dir) VALUES (4, 1, 0, 'd.txt', 1, 0)",
            [],
        ).expect("p4");

        conn.execute(
            "INSERT INTO scans (id, volume_id, root_path, started_at, options_json, status, total_logical)
             VALUES ('prior', 1, 'C:\\', 0, '{}', 'complete', 60)",
            [],
        ).expect("prior");
        conn.execute(
            "INSERT INTO scans (id, volume_id, root_path, started_at, options_json, status, total_logical)
             VALUES ('new', 1, 'C:\\', 1, '{}', 'complete', 60)",
            [],
        ).expect("new");

        // Prior: a=10, b=20, c=30 (total 60)
        // New:   a=10, b=40,    d=10 (total 60)
        for (scan, path, size) in [
            ("prior", 1i64, 10i64),
            ("prior", 2, 20),
            ("prior", 3, 30),
            ("new", 1, 10),
            ("new", 2, 40), // modified (was 20, now 40)
            ("new", 4, 10),
        ] {
            conn.execute(
                "INSERT INTO files (scan_id, path_id, logical_size, allocated_size, mtime, ctime, attrs, reparse_tag, reparse_kind, hardlink_count, sparse, file_id)
                 VALUES (?, ?, ?, ?, 0, 0, 0, 0, 'none', 1, 0, 0)",
                params![scan, path, size, size],
            ).expect("file");
        }
    }

    #[test]
    fn computes_diff_correctly() {
        let conn = open_in_memory().expect("open");
        migrate(&conn).expect("migrate");
        seed_two_scans(&conn);

        let diff = compute_diff(&conn, "prior", "new").expect("compute");
        let diff = diff.expect("some");
        assert_eq!(diff.added_files, 1, "path 4 was added");
        assert_eq!(diff.removed_files, 1, "path 3 was removed");
        assert_eq!(diff.modified_files, 1, "path 2 changed size");
        assert_eq!(diff.delta_bytes, 0, "totals match (60 vs 60)");
    }

    #[test]
    fn returns_none_for_missing_scan() {
        let conn = open_in_memory().expect("open");
        migrate(&conn).expect("migrate");
        let diff = compute_diff(&conn, "missing", "also-missing").expect("compute");
        assert!(diff.is_none(), "should return None for missing scans");
    }
}
