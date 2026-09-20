//! Reparse-point interpretation.
//!
//! Per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.2.
//!
//! Phase 1: stub. We use `std::fs` which doesn't expose reparse tags, so
//! every file is treated as `ReparseKind::None`. Phase 2 will replace this
//! with real `windows-rs` calls (`FSCTL_GET_REPARSE_POINT`) and the full
//! tag-classification table from `17-WINDOWS-SPECIFIC-FEATURES.md` §17.2.

use crate::scan::event::ReparseKind;

/// Classify a reparse tag. Phase 1: always `None`.
///
/// Phase 2 implementation:
/// ```ignore
/// use windows::Win32::Storage::FileSystem::*;
/// match tag {
///     IO_REPARSE_TAG_MOUNT_POINT => ReparseKind::Junction,
///     IO_REPARSE_TAG_SYMLINK => ReparseKind::Symlink,
///     IO_REPARSE_TAG_HSM => ReparseKind::Hsm,
///     IO_REPARSE_TAG_SIS => ReparseKind::Sis,
///     IO_REPARSE_TAG_DEDUP => ReparseKind::Dedup,
///     IO_REPARSE_TAG_APPXLINK => ReparseKind::AppxLink,
///     0 => ReparseKind::None,
///     _ => ReparseKind::Other,
/// }
/// ```
#[must_use]
pub const fn classify_reparse_tag(_tag: u32) -> ReparseKind {
    // Phase 1: std::fs doesn't give us reparse tags. Always None.
    ReparseKind::None
}
