//! Scan options — the request shape passed from the renderer.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.4. This type is the Rust mirror of the TS
//! `ScanOptions` in `packages/ipc/src/types.ts`. Per `02-SYSTEM-ARCHITECTURE.md`
//! §2.7, the TS definitions win in case of conflict.

use std::path::PathBuf;

/// Whether to descend into reparse points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FollowReparse {
    Never = 0,
    All = 1,
    OnlyJunctionsAndSymlinks = 2,
}

impl FollowReparse {
    #[must_use]
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "never" | "Never" => Self::Never,
            "all" | "All" => Self::All,
            "only-junctions-and-symlinks" | "OnlyJunctionsAndSymlinks" => {
                Self::OnlyJunctionsAndSymlinks
            }
            _ => Self::Never,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::All => "all",
            Self::OnlyJunctionsAndSymlinks => "only-junctions-and-symlinks",
        }
    }
}

/// How to account for hard links (per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HardLinkMode {
    /// Don't detect hard links; report logical size for every directory entry. (Free tier default.)
    None = 0,
    /// Detect hard links; report logical size for every entry but mark duplicates in the inspector.
    /// Subtree totals are logical (may double-count).
    Logical = 1,
    /// Detect hard links; report allocated size only for the first occurrence per `file_id`.
    /// Subtree totals are allocated-once. (Pro tier.)
    AllocatedOnce = 2,
}

impl HardLinkMode {
    #[must_use]
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "none" | "None" => Self::None,
            "logical" | "Logical" => Self::Logical,
            "allocated-once" | "AllocatedOnce" => Self::AllocatedOnce,
            _ => Self::None,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Logical => "logical",
            Self::AllocatedOnce => "allocated-once",
        }
    }
}

/// Glob pattern (e.g. `*.tmp` or `node_modules/**`).
///
/// Phase 1: stored as a plain string; matching is substring-based for simplicity.
/// Phase 2: switch to `globset` crate for proper glob matching.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobPattern(pub String);

impl GlobPattern {
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Lossy substring match (Phase 1 only). Phase 2: use `globset`.
    #[must_use]
    pub fn matches_path(&self, p: &PathBuf) -> bool {
        let s = p.to_string_lossy();
        s.contains(&self.0)
    }
}

/// The full scan-options request, mirroring the TS `ScanOptions`.
#[derive(Debug, Clone, PartialEq)]
pub struct ScanOptions {
    pub follow_reparse_points: FollowReparse,
    pub count_hard_links: HardLinkMode,
    pub hash_duplicates: bool,
    pub respect_gitignore: bool,
    pub include_patterns: Vec<GlobPattern>,
    pub exclude_patterns: Vec<GlobPattern>,
    pub min_size_bytes: Option<u64>,
    pub max_depth: Option<u32>,
    /// Pro-only (per `01-PRODUCT-VISION.md` §1.5 capability matrix).
    pub incremental_usn: bool,
    pub prior_scan_id: Option<String>, // for diff (Pro-only)
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            follow_reparse_points: FollowReparse::OnlyJunctionsAndSymlinks,
            count_hard_links: HardLinkMode::None,
            hash_duplicates: false,
            respect_gitignore: false,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            min_size_bytes: None,
            max_depth: Some(32),
            incremental_usn: false,
            prior_scan_id: None,
        }
    }
}
