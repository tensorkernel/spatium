//! Duplicate detection pipeline.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.7.
//!
//! Three stages:
//! 1. Group by size (cheap, exact): two files with different sizes cannot be duplicates.
//! 2. For each group of size-2+, compute a partial hash (xxhash64 of head + tail + middle).
//! 3. For each sub-group of partial-hash-2+, compute a full hash (SHA-256).
//!
//! Why this works: size alone eliminates ~98% of files. Partial hash eliminates
//! another ~90% of the remainder. Full hash runs on <0.1% of files.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.7, NEVER immediately SHA-256 every file.

pub mod full;
pub mod partial;
pub mod pipeline;

pub use full::sha256_of_file;
pub use partial::partial_hash_of_file;
pub use pipeline::{DedupPipeline, DuplicateGroup};
