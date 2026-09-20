//! Bounded min-heap for top-N largest files. Per `05-RUST-CORE-ENGINE.md` §5.2.4.

use crate::aggregate::arena::NodeId;

/// A bounded min-heap of (size, NodeId) pairs. Keeps the top-N largest entries.
///
/// Per `05-RUST-CORE-ENGINE.md` §5.2.4, observation is O(log cap).
pub struct TopNHeap {
    cap: usize,
    heap: std::collections::BinaryHeap<std::cmp::Reverse<(u64, NodeId)>>,
}

impl TopNHeap {
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            cap: cap.max(1),
            heap: std::collections::BinaryHeap::with_capacity(cap.max(1) + 1),
        }
    }

    /// Observe a file's (size, NodeId). If the heap is full and the observed size
    /// is smaller than the min, the entry is dropped.
    pub fn observe(&mut self, size: u64, id: NodeId) {
        // Use Reverse to turn BinaryHeap (max-heap) into a min-heap.
        let entry = std::cmp::Reverse((size, id));
        if self.heap.len() < self.cap {
            self.heap.push(entry);
        } else if let Some(&std::cmp::Reverse((min_size, _))) = self.heap.peek() {
            if size > min_size {
                self.heap.pop();
                self.heap.push(entry);
            }
        }
    }

    /// Drain into a sorted-descending vector of (size, NodeId).
    #[must_use]
    pub fn into_sorted_vec(self) -> Vec<(u64, NodeId)> {
        let mut v: Vec<_> = self.heap.into_iter().map(|r| r.0).collect();
        v.sort_unstable_by(|a, b| b.cmp(a)); // descending by size
        v
    }

    /// Current number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}
