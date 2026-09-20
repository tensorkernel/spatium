//! Path-component interning.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.2.2.
//!
//! Path components like "Windows" appear thousands of times in a scan. Interning
//! means we store the bytes once and reference them by a 4-byte `StringId`.
//!
//! # Implementation
//!
//! `StringPool` is a packed UTF-8 byte buffer + offset table. Hash collisions
//! are resolved by linear comparison on the bytes. The hash is ahash (fast,
//! non-cryptographic; we're not protecting against adversarial input here —
//! the user is scanning their own disk).

use ahash::AHasher;
use std::hash::{Hash, Hasher};

/// Identifier for an interned string. `u32` (per `05-RUST-CORE-ENGINE.md`
/// §5.2.2). Supports up to 4 billion unique path components per scan —
/// far more than the realistic worst case (~1M unique components on a 10M-file scan).
pub type StringId = u32;

/// A deduplicating string store. Per `05-RUST-CORE-ENGINE.md` §5.2.2.
pub struct StringPool {
    /// Packed UTF-8 bytes for all interned strings.
    storage: Vec<u8>,
    /// Start offset + end offset of each interned string. Indexed by `StringId`.
    spans: Vec<(u32, u32)>,
    /// Hash → StringId. Collisions resolved by linear comparison on the bytes.
    /// We use `AHasher::default()` directly (deterministic per-process) so two
    /// calls with the same string produce the same hash.
    hash_to_id: hashbrown::HashMap<u64, StringId>,
}

impl StringPool {
    #[must_use]
    pub fn new() -> Self {
        // Slot 0 is reserved for "" (empty string). The arena root uses this.
        let mut pool = Self {
            storage: Vec::with_capacity(4096),
            spans: Vec::with_capacity(1024),
            hash_to_id: hashbrown::HashMap::default(),
        };
        let empty_id = pool.intern("");
        debug_assert_eq!(empty_id, 0);
        pool
    }

    /// Intern a string. Returns the existing ID if the string is already interned.
    pub fn intern(&mut self, s: &str) -> StringId {
        let mut hasher = AHasher::default();
        s.hash(&mut hasher);
        let hash = hasher.finish();

        // Fast path: hit in cache.
        if let Some(&id) = self.hash_to_id.get(&hash) {
            // Verify (hash collisions are rare but possible).
            let (start, end) = self.spans[id as usize];
            if &self.storage[start as usize..end as usize] == s.as_bytes() {
                return id;
            }
            // Collision: fall through and do linear search.
        }
        // Slow path: insert.
        let start = self.storage.len() as u32;
        self.storage.extend_from_slice(s.as_bytes());
        let end = self.storage.len() as u32;
        let id = self.spans.len() as StringId;
        self.spans.push((start, end));
        self.hash_to_id.insert(hash, id);
        id
    }

    /// Look up a string by ID. Returns `""` for an invalid ID.
    #[must_use]
    pub fn get(&self, id: StringId) -> &str {
        let (start, end) = match self.spans.get(id as usize) {
            Some(s) => *s,
            None => return "",
        };
        // SAFETY: We only ever insert valid UTF-8 bytes into `storage` (via
        // `s.as_bytes()` where `s: &str`). Slicing within the bounds we
        // recorded is therefore safe.
        let bytes = &self.storage[start as usize..end as usize];
        std::str::from_utf8(bytes).unwrap_or("")
    }

    /// Number of interned strings (including the reserved empty slot).
    #[must_use]
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    /// Always `false` (we always have the empty-string slot).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Total bytes used by `storage` (packed UTF-8 buffer).
    #[must_use]
    pub fn storage_bytes(&self) -> usize {
        self.storage.len()
    }

    /// Iterator over `(StringId, &str)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (StringId, &str)> {
        (0..self.spans.len() as StringId).map(|id| (id, self.get(id)))
    }
}

impl Default for StringPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interns_empty_first() {
        let pool = StringPool::new();
        assert_eq!(pool.len(), 1);
        assert_eq!(pool.get(0), "");
    }

    #[test]
    fn dedupes_identical_strings() {
        let mut pool = StringPool::new();
        let a = pool.intern("Windows");
        let b = pool.intern("Windows");
        assert_eq!(a, b);
        assert_eq!(pool.get(a), "Windows");
    }

    #[test]
    fn distinct_strings_get_distinct_ids() {
        let mut pool = StringPool::new();
        let a = pool.intern("Users");
        let b = pool.intern("Windows");
        let c = pool.intern("System32");
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
        assert_eq!(pool.get(a), "Users");
        assert_eq!(pool.get(b), "Windows");
        assert_eq!(pool.get(c), "System32");
    }

    #[test]
    fn handles_unicode_paths() {
        let mut pool = StringPool::new();
        let a = pool.intern("数据");
        let b = pool.intern("データ");
        let c = pool.intern("📄");
        assert_eq!(pool.get(a), "数据");
        assert_eq!(pool.get(b), "データ");
        assert_eq!(pool.get(c), "📄");
    }

    #[test]
    fn storage_grows_packed() {
        let mut pool = StringPool::new();
        let a = pool.intern("abc");
        let b = pool.intern("defg");
        let c = pool.intern("h");
        // Storage should be exactly 3+4+1 = 8 bytes (plus the empty 0 bytes).
        // Empty string takes 0 bytes.
        assert_eq!(pool.storage_bytes(), 8);
        // Re-interning should not grow storage.
        let _ = pool.intern("abc");
        assert_eq!(pool.storage_bytes(), 8);
        // 4 entries total: empty + abc + defg + h.
        assert_eq!(pool.len(), 4);
        // Ensure IDs are correct.
        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_eq!(c, 3);
    }
}
