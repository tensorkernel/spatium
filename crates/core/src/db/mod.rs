//! SQLite persistence layer.
//!
//! Per `12-DATABASE-SCHEMA.md`.
//!
//! # Design
//!
//! - One writer thread. The Rust aggregator owns the connection; the renderer never writes.
//! - WAL mode. Allows concurrent readers while writing.
//! - Batched transactions: commit every 250 ms or 5000 rows, whichever first.
//! - Cache, not authority. If the DB is missing or corrupt, the app re-scans.

pub mod history;
pub mod migrate;
pub mod read;
pub mod schema;
pub mod write;

pub use history::{compute_diff, ScanDiff};
pub use read::{DbReader, FileRow, HistoryEntry, ScanSummary};
pub use schema::{open, DbError};
pub use write::{DbWriter, WriteBatch, WriteRow};
