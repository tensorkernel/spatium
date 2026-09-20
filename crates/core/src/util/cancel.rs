//! Cancellation token. Per `05-RUST-CORE-ENGINE.md` §5.5.
//!
//! A simple `AtomicBool` + a `parking_lot::Mutex` for the rare case where
//! someone wants to `await` cancellation. Phase 1 uses only the synchronous
//! `is_set()` path; Phase 2 adds an async-friendly `cancelled()` future.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Cancellation token. Cheap to clone (Arc-shared). Per `05-RUST-CORE-ENGINE.md` §5.5.
pub struct CancelToken {
    set: Arc<AtomicBool>,
}

impl CancelToken {
    #[must_use]
    pub fn new() -> Self {
        Self {
            set: Arc::new(AtomicBool::new(false)),
        }
    }

    /// True if `set()` has been called.
    #[must_use]
    pub fn is_set(&self) -> bool {
        self.set.load(Ordering::Acquire)
    }

    /// Trigger cancellation. Idempotent.
    pub fn set(&self) {
        self.set.store(true, Ordering::Release);
    }

    /// Reset the token to unset (used before starting a new scan on the same controller).
    pub fn reset(&self) {
        self.set.store(false, Ordering::Release);
    }
}

impl Clone for CancelToken {
    fn clone(&self) -> Self {
        Self {
            set: self.set.clone(),
        }
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}
