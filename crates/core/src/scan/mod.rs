//! Filesystem scanner.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.3.
//!
//! Phase 1 (this module): single-threaded traversal using `std::fs::read_dir`.
//! Per `04-PHASES-OVERVIEW.md` §4.3, Phase 1 is out of scope for: parallelism
//! (rayon), NTFS specifics (allocated size, reparse, hard links, sparse),
//! SQLite persistence, and license checks.
//!
//! Phase 2 will replace `worker` with a rayon-based parallel traversal and
//! `windows-rs` calls for NTFS-specific data.

pub mod event;
pub mod options;
pub mod reparse;
pub mod scanner;
#[cfg(feature = "default")]
pub mod worker;

pub use event::{ScanEvent, ScanPhase, ScanStatus};
pub use options::{FollowReparse, GlobPattern, HardLinkMode, ScanOptions};
pub use scanner::{ScanController, ScanId, ScanStartError};
