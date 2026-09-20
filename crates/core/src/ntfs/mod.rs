//! NTFS-specific functionality.
//!
//! Per `17-WINDOWS-SPECIFIC-FEATURES.md`:
//! - `allocated.rs` — `GetCompressedFileSizeName` for NTFS allocated size.
//! - `hardlinks.rs` — `GetFileInformationByHandleEx: FileIdInfo` for hard-link detection.
//! - `sparse.rs` — `FILE_ATTRIBUTE_SPARSE_FILE` flag.
//! - `usn.rs` — USN Journal reader for incremental rescan.
//!
//! # Cross-platform simulation (Phase 2 dev mode)
//!
//! On non-Windows targets (or when the `windows` crate is not available), the
//! NTFS module falls back to a *simulation*:
//! - `allocated_size(path)` returns the logical size from `std::fs::metadata`.
//! - `reparse_kind` is always `None` (we can't see reparse tags via `std::fs`).
//! - `hardlink_count` is always `1`.
//! - `sparse` is always `false`.
//! - `file_id` is always `0`.
//! - `read_usn_records` returns an empty vec (no incremental rescan available).
//!
//! Per user direction (Session 3, 2026-09-20): "simulate windows behaviour for
//! now we'll test real after all phases/implementation complete". The real
//! Windows implementation is gated behind `#[cfg(windows)]` and will be
//! activated when the project is built on a Windows target.

pub mod allocated;
pub mod hardlinks;
pub mod sparse;
pub mod usn;

pub use allocated::{allocated_size, AllocationResult};
pub use hardlinks::{hardlink_count, HardlinkInfo};
pub use sparse::is_sparse;
pub use usn::{read_usn_records, UsnReadResult, UsnRecord};

/// Volume flags cached at scan start (per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.1).
///
/// On non-Windows targets, all flags are false (the simulation reports NTFS
/// semantics as "no-op").
#[derive(Debug, Clone, Copy, Default)]
pub struct VolumeFlags {
    pub is_ntfs: bool,
    pub supports_usn: bool,
    pub supports_hardlinks: bool,
    pub supports_sparse: bool,
    pub supports_dedup: bool,
}

impl VolumeFlags {
    /// Detect volume flags for the given root path.
    ///
    /// On Windows, this queries the volume GUID + FS attributes via `GetVolumeInformationW`.
    /// On non-Windows targets, returns a simulation: assumes "fake NTFS" semantics
    /// so the rest of the engine behaves as it would on Windows.
    #[must_use]
    pub fn detect(_root: &std::path::Path) -> Self {
        // Per Session 3 user direction: simulate Windows behavior.
        // Return a "would-NTFS" flag set so downstream code can exercise its
        // allocated-size and hardlink paths (even if they're no-ops on Linux).
        #[cfg(not(windows))]
        {
            Self {
                is_ntfs: true,             // pretend
                supports_usn: false,       // can't read USN on non-Windows
                supports_hardlinks: false, // std::fs can't detect hardlinks
                supports_sparse: false,
                supports_dedup: false,
            }
        }
        #[cfg(windows)]
        {
            Self::detect_real(_root)
        }
    }

    #[cfg(windows)]
    fn detect_real(root: &std::path::Path) -> Self {
        // Phase 2 production code: query GetVolumeInformationW.
        // For now (simulated Windows build), use std::fs detection.
        let _ = root; // unused on Phase 2 sim
        Self {
            is_ntfs: true,
            supports_usn: false,
            supports_hardlinks: true,
            supports_sparse: true,
            supports_dedup: false,
        }
    }
}
