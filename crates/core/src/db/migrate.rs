//! Database migrations.
//!
//! Per `12-DATABASE-SCHEMA.md` §12.3 (Migrations 001–004).

use rusqlite::Connection;

use super::DbError;

/// The current schema version. Bump when adding a new migration.
pub const CURRENT_VERSION: u32 = 4;

/// Migrations in order. Each migration runs `PRAGMA user_version = N` after applying.
const MIGRATIONS: &[(u32, &str)] = &[
    (1, MIGRATION_001),
    (2, MIGRATION_002),
    (3, MIGRATION_003),
    (4, MIGRATION_004),
];

/// Run all pending migrations. Idempotent — re-running a no-op if already at current.
pub fn migrate(conn: &Connection) -> Result<(), DbError> {
    let current: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

    for &(target, sql) in MIGRATIONS {
        if current >= target {
            continue;
        }
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(&format!("PRAGMA user_version = {target}"), [])?;
        tx.commit()?;
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────
// Migration 001 — initial schema (per §12.3.1)
// ─────────────────────────────────────────────────────────────────────

const MIGRATION_001: &str = r#"
CREATE TABLE IF NOT EXISTS volumes (
    id              INTEGER PRIMARY KEY,
    guid            TEXT NOT NULL UNIQUE,
    serial          INTEGER NOT NULL,
    label           TEXT,
    drive_letters   TEXT NOT NULL,
    fs_type         TEXT NOT NULL,
    total_bytes     INTEGER NOT NULL,
    free_bytes      INTEGER NOT NULL,
    first_seen_at   INTEGER NOT NULL,
    last_seen_at    INTEGER NOT NULL,
    usn_journal_id  INTEGER,
    usn_cursor      INTEGER
);
CREATE INDEX IF NOT EXISTS idx_volumes_guid ON volumes(guid);
CREATE INDEX IF NOT EXISTS idx_volumes_serial ON volumes(serial);

CREATE TABLE IF NOT EXISTS scans (
    id              TEXT PRIMARY KEY,
    volume_id       INTEGER NOT NULL REFERENCES volumes(id),
    root_path       TEXT NOT NULL,
    started_at      INTEGER NOT NULL,
    completed_at    INTEGER,
    duration_ms     INTEGER,
    file_count      INTEGER,
    dir_count       INTEGER,
    total_logical   INTEGER,
    total_allocated INTEGER,
    options_json    TEXT NOT NULL,
    status          TEXT NOT NULL CHECK (status IN
                    ('running','complete','cancelled','failed','aborted')),
    error_msg       TEXT,
    parent_scan_id  TEXT REFERENCES scans(id)
);
CREATE INDEX IF NOT EXISTS idx_scans_volume ON scans(volume_id, started_at);
CREATE INDEX IF NOT EXISTS idx_scans_completed ON scans(completed_at);

CREATE TABLE IF NOT EXISTS paths (
    id              INTEGER PRIMARY KEY,
    volume_id       INTEGER NOT NULL REFERENCES volumes(id),
    parent_id       INTEGER REFERENCES paths(id),
    name            TEXT NOT NULL,
    depth           INTEGER NOT NULL,
    is_dir          INTEGER NOT NULL CHECK (is_dir IN (0,1)),
    UNIQUE(volume_id, parent_id, name)
);
CREATE INDEX IF NOT EXISTS idx_paths_parent ON paths(volume_id, parent_id);
CREATE INDEX IF NOT EXISTS idx_paths_name ON paths(name);

CREATE TABLE IF NOT EXISTS files (
    scan_id         TEXT NOT NULL REFERENCES scans(id),
    path_id         INTEGER NOT NULL REFERENCES paths(id),
    logical_size    INTEGER NOT NULL,
    allocated_size  INTEGER NOT NULL,
    mtime           INTEGER NOT NULL,
    ctime           INTEGER NOT NULL,
    attrs           INTEGER NOT NULL,
    reparse_tag     INTEGER NOT NULL DEFAULT 0,
    reparse_kind    TEXT NOT NULL DEFAULT 'none' CHECK (reparse_kind IN
                    ('none','junction','symlink','mountpoint','hsm','sis','dedup','appxlink','loop','other')),
    hardlink_count  INTEGER NOT NULL DEFAULT 1,
    sparse          INTEGER NOT NULL DEFAULT 0 CHECK (sparse IN (0,1)),
    file_id         INTEGER NOT NULL,
    extension_id    INTEGER REFERENCES extensions(id),
    hash_sha256     TEXT,
    hash_partial    INTEGER,
    PRIMARY KEY (scan_id, path_id)
);
CREATE INDEX IF NOT EXISTS idx_files_scan_size ON files(scan_id, logical_size DESC);
CREATE INDEX IF NOT EXISTS idx_files_scan_alloc ON files(scan_id, allocated_size DESC);
CREATE INDEX IF NOT EXISTS idx_files_scan_ext  ON files(scan_id, extension_id);
CREATE INDEX IF NOT EXISTS idx_files_scan_mtime ON files(scan_id, mtime);
CREATE INDEX IF NOT EXISTS idx_files_hash ON files(scan_id, hash_sha256) WHERE hash_sha256 IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_files_hardlink ON files(scan_id, file_id) WHERE hardlink_count > 1;
CREATE INDEX IF NOT EXISTS idx_files_sparse ON files(scan_id, sparse) WHERE sparse = 1;

CREATE TABLE IF NOT EXISTS extensions (
    id              INTEGER PRIMARY KEY,
    ext             TEXT NOT NULL UNIQUE,
    category        TEXT NOT NULL,
    description     TEXT
);
CREATE INDEX IF NOT EXISTS idx_extensions_ext ON extensions(ext);

CREATE TABLE IF NOT EXISTS settings (
    key             TEXT PRIMARY KEY,
    value_json      TEXT NOT NULL,
    updated_at      INTEGER NOT NULL
);
"#;

// ─────────────────────────────────────────────────────────────────────
// Migration 002 — cleanup staging (per §12.3.2)
// ─────────────────────────────────────────────────────────────────────

const MIGRATION_002: &str = r#"
CREATE TABLE IF NOT EXISTS cleanup_items (
    id              TEXT PRIMARY KEY,
    added_at        INTEGER NOT NULL,
    path_id         INTEGER REFERENCES paths(id),
    scan_id         TEXT REFERENCES scans(id),
    display_path    TEXT NOT NULL,
    logical_size    INTEGER,
    allocated_size  INTEGER,
    is_dir          INTEGER NOT NULL CHECK (is_dir IN (0,1)),
    source          TEXT NOT NULL,
    committed_at    INTEGER,
    commit_status   TEXT
);
CREATE INDEX IF NOT EXISTS idx_cleanup_pending ON cleanup_items(committed_at) WHERE committed_at IS NULL;
"#;

// ─────────────────────────────────────────────────────────────────────
// Migration 003 — quick wins (per §12.3.3)
// ─────────────────────────────────────────────────────────────────────

const MIGRATION_003: &str = r#"
CREATE TABLE IF NOT EXISTS quick_wins (
    id              INTEGER PRIMARY KEY,
    scan_id         TEXT NOT NULL REFERENCES scans(id),
    rule_id         TEXT NOT NULL,
    path_id         INTEGER REFERENCES paths(id),
    display_path    TEXT NOT NULL,
    logical_size    INTEGER,
    reason          TEXT NOT NULL,
    severity        TEXT NOT NULL CHECK (severity IN ('low','medium','high')),
    confidence      REAL NOT NULL CHECK (confidence BETWEEN 0 AND 1)
);
CREATE INDEX IF NOT EXISTS idx_quickwins_scan ON quick_wins(scan_id, logical_size DESC);
"#;

// ─────────────────────────────────────────────────────────────────────
// Migration 004 — duplicate groups (per §12.3.4)
// ─────────────────────────────────────────────────────────────────────

const MIGRATION_004: &str = r#"
CREATE TABLE IF NOT EXISTS duplicate_groups (
    id              TEXT PRIMARY KEY,
    scan_id         TEXT NOT NULL REFERENCES scans(id),
    hash_sha256     TEXT NOT NULL,
    size_bytes      INTEGER NOT NULL,
    file_count      INTEGER NOT NULL,
    total_wasted    INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS duplicate_files (
    group_id        TEXT NOT NULL REFERENCES duplicate_groups(id),
    path_id         INTEGER NOT NULL REFERENCES paths(id),
    scan_id         TEXT NOT NULL REFERENCES scans(id),
    PRIMARY KEY (group_id, path_id)
);
CREATE INDEX IF NOT EXISTS idx_dup_groups_scan ON duplicate_groups(scan_id, total_wasted DESC);
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::open_in_memory;

    #[test]
    fn migrates_fresh_db_to_current_version() {
        let conn = open_in_memory().expect("open");
        migrate(&conn).expect("migrate");
        let v: u32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .expect("query");
        assert_eq!(v, CURRENT_VERSION);
    }

    #[test]
    fn migration_is_idempotent() {
        let conn = open_in_memory().expect("open");
        migrate(&conn).expect("first migrate");
        migrate(&conn).expect("second migrate (no-op)");
        let v: u32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .expect("query");
        assert_eq!(v, CURRENT_VERSION);
    }

    #[test]
    fn all_tables_exist_after_migration() {
        let conn = open_in_memory().expect("open");
        migrate(&conn).expect("migrate");
        for table in [
            "volumes",
            "scans",
            "paths",
            "files",
            "extensions",
            "settings",
            "cleanup_items",
            "quick_wins",
            "duplicate_groups",
            "duplicate_files",
        ] {
            let n: i64 = conn
                .query_row(
                    &format!(
                        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='{table}'"
                    ),
                    [],
                    |r| r.get(0),
                )
                .expect("query");
            assert_eq!(n, 1, "table {table} should exist after migration");
        }
    }
}
