//! Path normalization and loop detection.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.1 and `17-WINDOWS-SPECIFIC-FEATURES.md
//! §17.2.1 (loop detection via ancestor set).
//!
//! Phase 1: uses `std::fs::canonicalize` (cross-platform). Phase 2 will
//! switch to a Windows-native canonicalization that respects `\\?\` long-path
//! prefix and does NOT resolve junctions (we want to keep junctions visible
//! in the scan, not collapse them).

use std::path::{Path, PathBuf};

/// Canonicalize a path. Phase 1: wraps `std::fs::canonicalize`.
///
/// Per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.10, Phase 2 will use the
/// `\\?\` prefix for long paths on Windows.
pub fn canonicalize(p: &Path) -> std::io::Result<PathBuf> {
    std::fs::canonicalize(p)
}

/// Detect whether `child` is an ancestor of `parent` (i.e., whether descending
/// into `child` would create a loop).
///
/// Per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.2.1.
#[must_use]
pub fn is_ancestor_of(ancestor: &Path, candidate: &Path) -> bool {
    // Compare canonical paths. If `ancestor` is a prefix of `candidate`'s
    // canonical form AND the reparse point points back into an ancestor,
    // we have a loop.
    let Ok(a) = canonicalize(ancestor) else {
        return false;
    };
    let Ok(c) = canonicalize(candidate) else {
        return false;
    };
    c.starts_with(&a) && c != a
}

/// Detect whether `candidate`'s canonical form already exists in the `ancestors` set.
#[must_use]
pub fn creates_loop(ancestors: &[PathBuf], candidate: &Path) -> bool {
    let Ok(c) = canonicalize(candidate) else {
        return false;
    };
    ancestors.iter().any(|a| a == &c)
}
