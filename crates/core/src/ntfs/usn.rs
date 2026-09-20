//! USN Journal reader — incremental rescan.
//!
//! Per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.5.
//!
//! On Windows (Phase 2 real): `FSCTL_READ_USN_JOURNAL` reads the NTFS change
//! log since a persisted cursor. This requires `SE_MANAGE_VOLUME_NAME`
//! privilege; the privileged helper handles it.
//!
//! On non-Windows (Phase 2 simulation): returns `UsnReadResult::Unsupported`,
//! meaning "no incremental rescan available — do a full rescan".

use std::path::Path;

/// One USN record (a single filesystem change).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsnRecord {
    /// USN (offset into the journal).
    pub usn: i64,
    /// Time of the change (FILETIME as unix millis).
    pub timestamp: i64,
    /// Type of change (CREATE / DELETE / RENAME / DATA_EXTEND / DATA_TRUNCATION / etc.).
    pub reason: UsnReason,
    /// NTFS file reference number of the changed file.
    pub file_id: u64,
    /// Parent directory's NTFS file reference number.
    pub parent_file_id: u64,
    /// Source file name (for renames; empty otherwise).
    pub source_name: String,
    /// New file name.
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsnReason {
    Create,
    Delete,
    DataExtend,
    DataTruncation,
    RenameNewName,
    RenameOldName,
    SecurityChange,
    Other,
}

/// Result of a USN read.
#[derive(Debug, Clone, PartialEq)]
pub enum UsnReadResult {
    /// The journal was read successfully. Records are in chronological order.
    Ok {
        records: Vec<UsnRecord>,
        new_cursor: i64,
    },
    /// The journal is not available (disabled, not NTFS, or unsupported on this OS).
    /// Caller should fall back to a full rescan.
    Unsupported,
    /// The cursor was too old; the journal has rolled over. Caller should fall back
    /// to a full rescan.
    RolledOver,
    /// I/O error reading the journal.
    Error(String),
}

/// Read USN records since the given cursor.
///
/// Per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.5.
#[must_use]
pub fn read_usn_records(volume_root: &Path, journal_id: Option<i64>, cursor: i64) -> UsnReadResult {
    let _ = (volume_root, journal_id, cursor);
    #[cfg(windows)]
    {
        // Phase 2 production: call FSCTL_READ_USN_JOURNAL via the privileged helper.
        // For the simulated Windows build, return Unsupported.
    }
    UsnReadResult::Unsupported
}
