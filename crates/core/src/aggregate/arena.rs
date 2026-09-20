//! Arena-allocated tree of file/directory records.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.2.1 and §5.2.3.
//!
//! # Memory discipline
//!
//! - **No `String` per record.** Path components live in `StringPool`.
//! - **No `PathBuf` per record.** Paths are reconstructed on demand from
//!   parent chain + name pool.
//! - **No `Box<T>` indirection.** Records are stored contiguously for cache
//!   locality. Preallocated in 1M-record chunks (Phase 2).

use smallvec::SmallVec;

use crate::aggregate::string_pool::{StringId, StringPool};

/// Identifier for a node in the arena tree. The root is always `0`.
/// `u32` (per `05-RUST-CORE-ENGINE.md` §5.2.1): supports up to 4 billion nodes
/// per scan, which exceeds the projected 10-million-file target by 400x.
pub type NodeId = u32;

/// One file or directory record, arena-allocated. Per `05-RUST-CORE-ENGINE.md`
/// §5.2.1. The size target is 72 bytes (padded).
///
/// Note: we use plain `u32`/`u64`/`i64` instead of newtypes for cache density;
/// the newtype wrappers are in `string_pool.rs` (`StringId`) and here (`NodeId`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct FileRecord {
    /// Parent node ID. `0` = root.
    pub parent: NodeId,
    /// Index into the StringPool for this node's name (path component).
    pub name_id: StringId,
    /// Logical file size (size the file appears to have).
    pub size: u64,
    /// NTFS allocated size (== size on non-NTFS; Phase 1: == size).
    pub allocated: u64,
    /// Modification time, unix milliseconds.
    pub mtime: i64,
    /// Creation time, unix milliseconds.
    pub ctime: i64,
    /// Win32 file attributes (e.g. `FILE_ATTRIBUTE_DIRECTORY = 0x10`).
    pub attrs: u32,
    /// Reparse tag (0 if not a reparse point). Phase 1: always 0.
    pub reparse_tag: u32,
    /// Bitfield: sparse? hardlink? symlink? junction?
    pub flags: u16,
    /// NTFS file reference number; 0 if unavailable. Phase 1: always 0.
    pub file_id: u64,
    /// Index into the extension pool (lowercased). 0 = no extension / directory.
    pub extension_id: u32,
    /// Precomputed sort key (e.g. allocated size) for top-N heap.
    pub sort_key: u64,
}

impl FileRecord {
    /// True if this record is a directory.
    ///
    /// FILE_ATTRIBUTE_DIRECTORY = 0x10 (16).
    #[must_use]
    pub const fn is_directory(&self) -> bool {
        const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;
        (self.attrs & FILE_ATTRIBUTE_DIRECTORY) != 0
    }

    #[must_use]
    pub const fn sparse(&self) -> bool {
        const SPARSE_BIT: u16 = 0x0001;
        (self.flags & SPARSE_BIT) != 0
    }
}

/// The arena tree. Per `05-RUST-CORE-ENGINE.md` §5.2.3.
pub struct ArenaTree {
    /// Indexed by NodeId. Records are stored contiguously.
    records: Vec<FileRecord>,
    /// Indexed by NodeId. Each entry is the list of child NodeIds.
    /// `SmallVec<[NodeId; 4]>` because most directories have ≤4 children of
    /// interest at the top level; spilling to the heap is rare.
    children: Vec<SmallVec<[NodeId; 4]>>,
    /// String pool for path components.
    names: StringPool,
}

impl ArenaTree {
    /// Create a new, empty arena. The root node (NodeId 0) is pre-inserted
    /// as a placeholder with name "" (the scan root path is stored separately).
    #[must_use]
    pub fn new() -> Self {
        let mut names = StringPool::new();
        // Root name is the empty string (the aggregator stores the scan-root
        // path separately). StringId 0 = "".
        let _root_name_id = names.intern("");
        let root = FileRecord {
            parent: 0,
            name_id: 0,
            size: 0,
            allocated: 0,
            mtime: 0,
            ctime: 0,
            attrs: 0x0000_0010, // root is a directory
            reparse_tag: 0,
            flags: 0,
            file_id: 0,
            extension_id: 0,
            sort_key: 0,
        };
        Self {
            records: vec![root],
            children: vec![SmallVec::new()],
            names,
        }
    }

    /// Number of nodes (including the root).
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Always `false` (we always have the root node).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Look up a record by ID.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&FileRecord> {
        self.records.get(id as usize)
    }

    /// Look up a record by ID (mutable).
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut FileRecord> {
        self.records.get_mut(id as usize)
    }

    /// Get the children of a node.
    #[must_use]
    pub fn children_of(&self, id: NodeId) -> &[NodeId] {
        self.children.get(id as usize).map_or(&[], |c| c.as_slice())
    }

    /// Get the name of a node (path component only, not the full path).
    #[must_use]
    pub fn name(&self, id: NodeId) -> &str {
        self.records
            .get(id as usize)
            .map_or("", |r| self.names.get(r.name_id))
    }

    /// Get the interned name id of a node.
    #[must_use]
    pub fn name_id(&self, id: NodeId) -> Option<StringId> {
        self.records.get(id as usize).map(|r| r.name_id)
    }

    /// Intern a string into the string pool.
    pub fn intern(&mut self, s: &str) -> StringId {
        self.names.intern(s)
    }

    /// Get a string from the pool by ID.
    #[must_use]
    pub fn get_string(&self, id: StringId) -> &str {
        self.names.get(id)
    }

    /// Insert a file or directory. Returns `(new_node_id, name_string_id)`.
    ///
    /// This is the primary insertion path called by the scanner worker.
    pub fn insert_file_or_dir(
        &mut self,
        name: &str,
        parent: NodeId,
        is_dir: bool,
        size: u64,
        allocated: u64,
        mtime: i64,
        ctime: i64,
        attrs: u32,
    ) -> (NodeId, StringId) {
        let name_id = self.names.intern(name);
        let record = FileRecord {
            parent,
            name_id,
            size,
            allocated,
            mtime,
            ctime,
            attrs: if is_dir {
                attrs | 0x0000_0010
            } else {
                attrs & !0x0000_0010
            },
            reparse_tag: 0,
            flags: 0,
            file_id: 0,
            extension_id: 0,
            sort_key: allocated,
        };
        let new_id = self.records.len() as NodeId;
        self.records.push(record);
        self.children.push(SmallVec::new());
        // Append to parent's children list (if parent exists).
        if let Some(parent_children) = self.children.get_mut(parent as usize) {
            parent_children.push(new_id);
        }
        (new_id, name_id)
    }

    /// Bump the running totals up the parent chain. Used by the aggregator
    /// after inserting a file (the file's bytes are added to all ancestors).
    ///
    /// Per `05-RUST-CORE-ENGINE.md` §5.2.3 (subtree aggregate size).
    pub fn accumulate_upward(&mut self, from_node: NodeId, logical: u64, allocated: u64) {
        let mut current = Some(from_node);
        while let Some(id) = current {
            if let Some(rec) = self.records.get_mut(id as usize) {
                rec.size += logical;
                rec.allocated += allocated;
                if id == rec.parent {
                    break; // root self-loop
                }
                current = Some(rec.parent);
            } else {
                break;
            }
        }
    }

    /// Get the total number of bytes (logical) under a node (the node's own
    /// `size` field, which has been accumulated upward).
    #[must_use]
    pub fn subtree_size(&self, id: NodeId) -> u64 {
        self.records.get(id as usize).map_or(0, |r| r.size)
    }

    /// Get the total number of bytes (allocated) under a node.
    #[must_use]
    pub fn subtree_allocated(&self, id: NodeId) -> u64 {
        self.records.get(id as usize).map_or(0, |r| r.allocated)
    }

    /// Iterator over all (NodeId, &FileRecord) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &FileRecord)> {
        self.records
            .iter()
            .enumerate()
            .map(|(i, r)| (i as NodeId, r))
    }

    /// Reconstruct the full path of a node by walking up the parent chain.
    /// Per `05-RUST-CORE-ENGINE.md` §5.2.1 — paths are reconstructed on demand.
    ///
    /// Returns an empty PathBuf if the node doesn't exist.
    #[must_use]
    pub fn reconstruct_path(&self, id: NodeId) -> std::path::PathBuf {
        use std::path::PathBuf;
        let mut parts: Vec<&str> = Vec::new();
        let mut current = id;
        while let Some(rec) = self.records.get(current as usize) {
            let name = self.names.get(rec.name_id);
            if !name.is_empty() {
                parts.push(name);
            }
            if current == 0 || rec.parent == current {
                break;
            }
            current = rec.parent;
        }
        parts.reverse();
        let mut path = PathBuf::new();
        for p in parts {
            path.push(p);
        }
        path
    }
}

impl Default for ArenaTree {
    fn default() -> Self {
        Self::new()
    }
}
