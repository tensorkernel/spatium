//! Aggregator: arena tree, string pool, top-N, extension histogram.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.2.

pub mod arena;
pub mod extensions;
pub mod string_pool;
pub mod topn;
pub mod totals;

pub use arena::{ArenaTree, FileRecord, NodeId};
pub use extensions::{ExtensionHistogram, ExtensionId};
pub use string_pool::{StringId, StringPool};
pub use topn::TopNHeap;
pub use totals::RunningTotals;
