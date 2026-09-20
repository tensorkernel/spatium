//! Running totals per directory. Per `05-RUST-CORE-ENGINE.md` §5.2.3.
//!
//! In Phase 1 the `ArenaTree` itself accumulates upward; this module is for
//! Phase 2's parallel aggregator where multiple workers' totals need to be
//! merged. It's a simple struct for now.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RunningTotals {
    pub files: u64,
    pub directories: u64,
    pub logical_bytes: u64,
    pub allocated_bytes: u64,
}

impl RunningTotals {
    pub fn observe(&mut self, is_dir: bool, logical: u64, allocated: u64) {
        if is_dir {
            self.directories += 1;
        } else {
            self.files += 1;
        }
        self.logical_bytes += logical;
        self.allocated_bytes += allocated;
    }

    pub fn merge(&mut self, other: &Self) {
        self.files += other.files;
        self.directories += other.directories;
        self.logical_bytes += other.logical_bytes;
        self.allocated_bytes += other.allocated_bytes;
    }
}
