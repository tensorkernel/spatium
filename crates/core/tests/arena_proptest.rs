//! Property tests for `ArenaTree` and `StringPool`.
//!
//! Per `04-PHASES-OVERVIEW.md` §4.3 deliverable: "Unit tests for `ArenaTree`
//! and `StringPool` with `proptest`."
//! Per `20-QUALITY-GATES-AND-POVS.md` §20.2.5 RD-9: 1000 cases per property,
//! no failures.

#![cfg(feature = "proptest")]

use proptest::prelude::*;

use disk_analyzer_core::aggregate::{ArenaTree, StringPool};

// ─────────────────────────────────────────────────────────────────────
// StringPool properties
// ─────────────────────────────────────────────────────────────────────

proptest! {
    #[test]
    /// `intern(s)` is idempotent: interning the same string twice yields the same ID.
    fn string_pool_intern_idempotent(s in ".{0,256}") {
        let mut pool = StringPool::new();
        let a = pool.intern(&s);
        let b = pool.intern(&s);
        prop_assert_eq!(a, b);
        prop_assert_eq!(pool.get(a), &s);
    }

    #[test]
    /// `intern(a) != intern(b)` for `a != b` (modulo hash collisions, which we handle).
    fn string_pool_distinct_ids_for_distinct_strings(
        a in "[a-z]{1,16}",
        b in "[a-z]{1,16}",
    ) {
        if a != b {
            let mut pool = StringPool::new();
            let id_a = pool.intern(&a);
            let id_b = pool.intern(&b);
            prop_assert_ne!(id_a, id_b);
            prop_assert_eq!(pool.get(id_a), &a);
            prop_assert_eq!(pool.get(id_b), &b);
        }
    }

    #[test]
    /// `len()` matches the number of unique interned strings (plus the reserved
    /// empty-string slot at ID 0, unless the input contains "" which reuses it).
    fn string_pool_len_matches_unique_strings(strings in proptest::collection::vec("[a-z]{0,8}", 1..100)) {
        let mut pool = StringPool::new();
        let unique: std::collections::HashSet<&String> = strings.iter().collect();
        for s in &strings {
            let _ = pool.intern(s);
        }
        // The pool starts with 1 reserved slot for "".
        // For each unique non-empty string in input, the pool adds one slot.
        // For "" in input, the pool reuses slot 0 (no new slot added).
        let empty_in_input = unique.contains(&"".to_string());
        let expected = unique.len() + if empty_in_input { 0 } else { 1 };
        prop_assert_eq!(pool.len(), expected);
    }

    #[test]
    /// `storage_bytes()` is the sum of byte lengths of unique strings.
    fn string_pool_storage_bytes_matches_sum(strings in proptest::collection::vec("[a-z]{0,8}", 1..100)) {
        let mut pool = StringPool::new();
        let mut seen: std::collections::HashSet<&String> = std::collections::HashSet::new();
        let mut total = 0usize;
        for s in &strings {
            if seen.insert(s) {
                total += s.len();
            }
            let _ = pool.intern(s);
        }
        prop_assert_eq!(pool.storage_bytes(), total);
    }

    #[test]
    /// Handles unicode paths of arbitrary BMP characters.
    fn string_pool_handles_unicode(strings in proptest::collection::vec("[\\u{0020}-\\u{FFFF}]{0,16}", 1..50)) {
        let mut pool = StringPool::new();
        for s in &strings {
            let id = pool.intern(s);
            prop_assert_eq!(pool.get(id), s);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// ArenaTree properties
// ─────────────────────────────────────────────────────────────────────

proptest! {
    #[test]
    /// `insert_file_or_dir` returns a unique NodeId for each call.
    fn arena_unique_ids_for_distinct_inserts(
        names in proptest::collection::vec("[a-z]{1,8}", 1..100)
    ) {
        let mut arena = ArenaTree::new();
        let mut seen = std::collections::HashSet::new();
        for name in &names {
            let (id, _name_id) = arena.insert_file_or_dir(name, 0, false, 100, 100, 0, 0, 0);
            prop_assert!(seen.insert(id), "duplicate NodeId returned for distinct insert: {}", id);
        }
    }

    #[test]
    /// `get(name_id) == intern(name)` round-trip.
    fn arena_name_roundtrips(
        names in proptest::collection::vec("[a-z]{1,8}", 1..100)
    ) {
        let mut arena = ArenaTree::new();
        for name in &names {
            let (_id, name_id) = arena.insert_file_or_dir(name, 0, false, 100, 100, 0, 0, 0);
            prop_assert_eq!(arena.get_string(name_id), name);
        }
    }

    #[test]
    /// `children_of(root)` returns the nodes inserted directly under root, in
    /// insertion order. This is the most important property for the renderer
    /// (which walks children to render the treemap).
    fn arena_children_of_root_matches_insertions(
        names in proptest::collection::vec("[a-z]{1,8}", 1..200)
    ) {
        let mut arena = ArenaTree::new();
        let mut expected: Vec<u32> = Vec::with_capacity(names.len());

        for name in &names {
            let (new_id, _) = arena.insert_file_or_dir(name, 0, false, 100, 100, 0, 0, 0);
            expected.push(new_id);
        }

        let actual = arena.children_of(0);
        prop_assert_eq!(actual.len(), expected.len());
        for (i, &e) in expected.iter().enumerate() {
            prop_assert_eq!(actual[i], e, "mismatch at index {}", i);
        }
    }

    #[test]
    /// `accumulate_upward` increases `subtree_size` of all ancestors by the given amount.
    fn arena_accumulate_upward_increments_ancestors(
        // Build a chain: root → a → b → c
        _chain_len in 1u32..5,
    ) {
        let mut arena = ArenaTree::new();
        // Root is ID 0 with parent 0 (self).
        let mut current = 0u32;
        for i in 0..5 {
            let (new_id, _) = arena.insert_file_or_dir(&format!("level-{i}"), current, false, 0, 0, 0, 0, 0);
            current = new_id;
        }
        // Now accumulate upward from the deepest node (current).
        arena.accumulate_upward(current, 100, 100);
        // Walk the parent chain; every node should have size == 100.
        let mut id = Some(current);
        while let Some(node) = id {
            let rec = arena.get(node).expect("node exists");
            prop_assert_eq!(rec.size, 100, "node {} size mismatch", node);
            prop_assert_eq!(rec.allocated, 100, "node {} allocated mismatch", node);
            if node == 0 || rec.parent == node {
                break;
            }
            id = Some(rec.parent);
        }
    }

    #[test]
    /// `reconstruct_path` walks the parent chain correctly.
    fn arena_reconstruct_path_walks_parents(
        names in proptest::collection::vec("[a-z]{1,8}", 1..10)
    ) {
        let mut arena = ArenaTree::new();
        let mut ids = vec![0u32]; // root
        for name in &names {
            let parent = *ids.last().unwrap();
            let (new_id, _) = arena.insert_file_or_dir(name, parent, false, 0, 0, 0, 0, 0);
            ids.push(new_id);
        }
        // Last ID's reconstructed path should be all names in order.
        let last_id = *ids.last().unwrap();
        let path = arena.reconstruct_path(last_id);
        let path_str = path.to_string_lossy().to_string();
        for name in &names {
            prop_assert!(path_str.contains(name), "expected path to contain {}", name);
        }
    }
}
