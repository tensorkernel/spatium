//! Extension histogram. Per `05-RUST-CORE-ENGINE.md` §5.2.5.
//!
//! Maps lowercased file extension → total bytes + count. Used by the renderer's
//! file-type donut visualization.

use crate::aggregate::string_pool::{StringId, StringPool};
use ahash::RandomState;

/// Identifier for an interned extension.
pub type ExtensionId = u32;

/// Per `05-RUST-CORE-ENGINE.md` §5.2.5.
pub struct ExtensionHistogram {
    by_id: hashbrown::HashMap<StringId, u64, RandomState>,
    count_by_id: hashbrown::HashMap<StringId, u64, RandomState>,
}

impl ExtensionHistogram {
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_id: hashbrown::HashMap::default(),
            count_by_id: hashbrown::HashMap::default(),
        }
    }

    /// Observe a file's extension (interned) and its allocated size.
    pub fn observe(&mut self, extension_id: StringId, allocated: u64) {
        *self.by_id.entry(extension_id).or_insert(0) += allocated;
        *self.count_by_id.entry(extension_id).or_insert(0) += 1;
    }

    #[must_use]
    pub fn total_bytes(&self, extension_id: StringId) -> u64 {
        self.by_id.get(&extension_id).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn count(&self, extension_id: StringId) -> u64 {
        self.count_by_id.get(&extension_id).copied().unwrap_or(0)
    }

    /// Iterate over `(StringId, total_bytes, count)` triples, sorted by total_bytes descending.
    #[must_use]
    pub fn sorted_by_size(&self) -> Vec<(StringId, u64, u64)> {
        let mut v: Vec<_> = self
            .by_id
            .iter()
            .map(|(&id, &bytes)| (id, bytes, self.count_by_id.get(&id).copied().unwrap_or(0)))
            .collect();
        v.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        v
    }

    /// Look up the interned extension for a file name. Returns `None` if no extension.
    ///
    /// The caller is responsible for lowercasing (the pool stores lowercase).
    pub fn intern_for(pool: &mut StringPool, file_name: &str) -> Option<StringId> {
        // Extract the substring after the last '.'. Per `05-RUST-CORE-ENGINE.md`
        // §5.2.5 (extensions are lowercased). We treat files starting with '.'
        // (e.g. ".gitignore") as having no extension.
        let last_dot = file_name.rfind('.')?;
        if last_dot == 0 {
            return None; // hidden file like ".gitignore"
        }
        let ext = &file_name[last_dot + 1..];
        if ext.is_empty() {
            return None;
        }
        let lower = ext.to_ascii_lowercase();
        Some(pool.intern(&lower))
    }
}

impl Default for ExtensionHistogram {
    fn default() -> Self {
        Self::new()
    }
}
