//! Scan events and status types.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.3.2 and §5.14 (frozen public enums).
//!
//! These types are the wire format between Rust and the napi binding. The TS
//! counterparts live in `packages/ipc/src/types.ts`. Per `02-SYSTEM-ARCHITECTURE.md`
//! §2.7, **the TS definitions win in case of conflict** because they are what
//! the renderer sees.
//!
//! Any change to a public item here is a breaking IPC change requiring a Senior
//! Dev review (per `04-PHASES-OVERVIEW.md` §4.2).

use std::path::PathBuf;

use crate::aggregate::{NodeId, StringId};

/// Scan phase (per `05-RUST-CORE-ENGINE.md` §5.14).
///
/// **Frozen at Phase 1.** Do not reorder or remove variants without a
/// breaking-version bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ScanPhase {
    Enumerating = 0,
    Aggregating = 1,
    Hashing = 2,
    Persisting = 3,
    Complete = 4,
    Aborted = 5,
}

impl ScanPhase {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enumerating => "enumerating",
            Self::Aggregating => "aggregating",
            Self::Hashing => "hashing",
            Self::Persisting => "persisting",
            Self::Complete => "complete",
            Self::Aborted => "aborted",
        }
    }
}

/// Reparse-point classification (per `05-RUST-CORE-ENGINE.md` §5.14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ReparseKind {
    None = 0,
    Junction = 1,
    Symlink = 2,
    MountPoint = 3,
    Hsm = 4,
    Sis = 5,
    Dedup = 6,
    AppxLink = 7,
    /// A reparse loop was detected and the path was not descended into.
    Loop = 8,
    Other = 255,
}

impl ReparseKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Junction => "junction",
            Self::Symlink => "symlink",
            Self::MountPoint => "mountpoint",
            Self::Hsm => "hsm",
            Self::Sis => "sis",
            Self::Dedup => "dedup",
            Self::AppxLink => "appxlink",
            Self::Loop => "loop",
            Self::Other => "other",
        }
    }
}

/// A single scan event emitted by the worker.
///
/// Per `05-RUST-CORE-ENGINE.md` §5.14:
/// - `name` is sent only the first time a path component is seen; subsequent
///   events reference the `StringId`. The renderer maintains its own string pool
///   mirroring Rust's.
/// - `mtime` / `ctime` are unix milliseconds (i64; FILETIME converted).
#[derive(Debug, Clone, PartialEq)]
pub struct ScanEvent {
    /// The scan session this event belongs to.
    pub scan_id: ScanId,
    /// Parent node in the arena tree. `0` = root.
    pub parent_id: NodeId,
    /// Path component name (file or directory leaf). Owned because the
    /// string pool may not have interned this component yet — the aggregator
    /// interns it and converts to `StringId` before storing.
    pub name: String,
    /// Logical file size (size the file appears to have).
    pub logical: u64,
    /// NTFS allocated size. In Phase 1 (no NTFS specifics), this equals `logical`.
    /// In Phase 2, populated via `GetCompressedFileSizeName`.
    pub allocated: u64,
    /// Modification time, unix milliseconds.
    pub mtime: i64,
    /// Creation time, unix milliseconds.
    pub ctime: i64,
    /// Win32 file attributes (e.g. `FILE_ATTRIBUTE_DIRECTORY`).
    /// On non-Windows targets in Phase 1, synthesized from `std::fs` metadata.
    pub attrs: u32,
    /// Reparse-point classification. Phase 1: always `None` (we don't inspect
    /// reparse tags via `std::fs`).
    pub reparse_kind: ReparseKind,
    /// Hard-link count. Phase 1: always `1` (we don't detect hard links via `std::fs`).
    pub hardlink_count: u16,
    /// Whether this file is sparse. Phase 1: always `false`.
    pub sparse: bool,
    /// NTFS file reference number; `0` if unavailable. Phase 1: always `0`.
    pub file_id: u64,
    /// The path component the worker just enumerated, for the aggregator to intern.
    /// Always `Some` in Phase 1 (the aggregator interns and discards the owned String).
    pub name_id_hint: Option<StringId>,
}

impl ScanEvent {
    /// True if this event represents a directory.
    ///
    /// FILE_ATTRIBUTE_DIRECTORY = 0x10 (16).
    /// Per `Win32_Foundation` in windows-rs.
    #[must_use]
    pub const fn is_directory(attrs: u32) -> bool {
        const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;
        (attrs & FILE_ATTRIBUTE_DIRECTORY) != 0
    }
}

/// Status snapshot for a scan.
#[derive(Debug, Clone, PartialEq)]
pub struct ScanStatus {
    pub scan_id: ScanId,
    pub phase: ScanPhase,
    pub started_at: i64, // unix millis
    pub elapsed_ms: u64,
    pub files_scanned: u64,
    pub bytes_scanned: u64,   // logical total
    pub bytes_allocated: u64, // allocated total (NTFS truth; == logical in Phase 1)
    pub current_path: Option<PathBuf>,
    pub error: Option<String>, // populated if phase == Aborted
}

/// Opaque scan identifier (per `02-SYSTEM-ARCHITECTURE.md` §2.7, the TS side is `string`;
/// internally we use a u64 counter that we serialize as a decimal string).
pub use crate::scan::scanner::ScanId;
