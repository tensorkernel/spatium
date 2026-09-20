//! DiskAnalyzer Core Engine
//!
//! Per `02-SYSTEM-ARCHITECTURE.md` §2.1, this is **Layer C** (Windows / Filesystem).
//! It is the only layer that touches the filesystem, the database, or cryptography.
//!
//! # Module layout (per `05-RUST-CORE-ENGINE.md` §5.1)
//!
//! - `scan` — filesystem traversal (single-threaded Phase 1 + rayon parallel Phase 2)
//! - `aggregate` — arena tree, string pool, top-N, extension histogram
//! - `ntfs` — NTFS-specific (allocated size, reparse, hard links, sparse, USN)
//! - `db` — SQLite persistence
//! - `dedup` — duplicate detection pipeline
//! - `crypto` — license signature verify (ed25519-dalek)
//! - `util` — cancellation, path normalization, loop detection
//!
//! # Memory discipline (per §5.6)
//!
//! - No `String` per `FileRecord`. Path components live in `StringPool`.
//! - No `PathBuf` per `FileRecord`. Paths are reconstructed on demand.
//! - No `Arc<Mutex<...>>` in the hot path.
//! - `SmallVec` for short child lists.
//!
//! # Panic policy (per §5.10 and §15.8)
//!
//! - `panic = "abort"` in release.
//! - No `unwrap()` / `expect()` in scan or aggregate code paths (CI rule).
//! - Every `unsafe` block has a `// SAFETY:` comment justifying correctness.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(rust_2018_idioms, unused_lifetimes)]
#![warn(clippy::all, clippy::pedantic)]
// Per `20-QUALITY-GATES-AND-POVS.md` §20.2.5 RD-6: no unwrap/expect/panic in
// production scan/aggregate paths. Test code is exempt.
#![cfg_attr(
    not(test),
    warn(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::todo,
        clippy::dbg_macro,
        clippy::print_stdout,
        clippy::print_stderr,
    )
)]
// Phase 1 pragmatism: silence a few pedantic lints that don't help on this codebase.
#![allow(
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::match_same_arms,
    clippy::struct_excessive_bools,
    clippy::fn_params_excessive_bools,
    clippy::if_not_else,
    clippy::len_without_is_empty,
    clippy::manual_string_new,
    clippy::redundant_closure_for_method_calls,
    clippy::collapsible_if,
    clippy::single_match_else,
    clippy::new_without_default,
    clippy::return_self_not_must_use
)]

pub mod aggregate;
pub mod scan;
pub mod util;

// Phase 2+: default-on modules
#[cfg(feature = "crypto")]
pub mod crypto;
#[cfg(feature = "db")]
pub mod db;
#[cfg(feature = "dedup")]
pub mod dedup;
#[cfg(feature = "ntfs")]
pub mod ntfs;

/// Semantic version of the public API. Per `05-RUST-CORE-ENGINE.md` §5.14,
/// the public enums are frozen at Phase 1; changes require a Senior Dev review.
pub const ENGINE_VERSION: &str = "0.2.0";
