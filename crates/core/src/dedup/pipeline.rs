//! Duplicate detection pipeline — size → partial → full hash.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.7.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
// (Path is used in the test module below.)

use super::full::sha256_of_file;
use super::partial::partial_hash_of_file;

/// A group of duplicate files (≥2 paths, identical SHA-256).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size_bytes: u64,
    pub paths: Vec<PathBuf>,
    /// (paths.len() - 1) * size_bytes — the bytes that could be freed by deduplication.
    pub wasted_bytes: u64,
}

/// The dedup pipeline. Per `05-RUST-CORE-ENGINE.md` §5.7.
pub struct DedupPipeline {
    /// Stage 1: group by size (u64 size → Vec<PathBuf>).
    by_size: HashMap<u64, Vec<PathBuf>>,
}

impl DedupPipeline {
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_size: HashMap::new(),
        }
    }

    /// Stage 1: observe a file's (size, path). Per `05-RUST-CORE-ENGINE.md` §5.7.
    pub fn observe(&mut self, size: u64, path: PathBuf) {
        self.by_size.entry(size).or_default().push(path);
    }

    /// Stage 1 → 2 → 3: run the full pipeline and return duplicate groups.
    ///
    /// Per `05-RUST-CORE-ENGINE.md` §5.7. The pipeline:
    /// 1. Filter size-groups with ≥2 entries.
    /// 2. Compute partial hash for each; group by partial hash.
    /// 3. Compute full SHA-256 for sub-groups with ≥2 entries.
    /// 4. Return final duplicate groups.
    pub fn run(&self) -> Vec<DuplicateGroup> {
        let mut groups: Vec<DuplicateGroup> = Vec::new();

        // Stage 1: filter size-groups with ≥2 entries
        for (&size, paths) in &self.by_size {
            if paths.len() < 2 {
                continue;
            }
            // Stage 2: partial hash; group by (size, partial_hash)
            let mut by_partial: HashMap<u64, Vec<&PathBuf>> = HashMap::new();
            for p in paths {
                if let Some(ph) = partial_hash_of_file(p) {
                    by_partial.entry(ph).or_default().push(p);
                }
            }
            // Stage 3: full hash for sub-groups with ≥2 entries
            for (_, candidates) in by_partial {
                if candidates.len() < 2 {
                    continue;
                }
                let mut by_full: HashMap<String, Vec<PathBuf>> = HashMap::new();
                for c in candidates {
                    if let Some(h) = sha256_of_file(c) {
                        by_full.entry(h).or_default().push(c.clone());
                    }
                }
                // Final duplicate groups
                for (hash, paths) in by_full {
                    if paths.len() < 2 {
                        continue;
                    }
                    let wasted = (paths.len() as u64 - 1) * size;
                    groups.push(DuplicateGroup {
                        hash,
                        size_bytes: size,
                        paths,
                        wasted_bytes: wasted,
                    });
                }
            }
        }
        // Sort by wasted bytes descending
        groups.sort_unstable_by(|a, b| b.wasted_bytes.cmp(&a.wasted_bytes));
        groups
    }
}

impl Default for DedupPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_file(p: &Path, content: &[u8]) {
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        let mut f = std::fs::File::create(p).expect("create");
        f.write_all(content).expect("write");
    }

    #[test]
    fn finds_duplicates() {
        let dir = tempfile::tempdir().expect("tmp");
        let root = dir.path();
        write_file(&root.join("a.txt"), b"identical");
        write_file(&root.join("b.txt"), b"identical");
        write_file(&root.join("c.txt"), b"identical");
        write_file(&root.join("d.txt"), b"different");

        let mut pipeline = DedupPipeline::new();
        pipeline.observe(9, root.join("a.txt"));
        pipeline.observe(9, root.join("b.txt"));
        pipeline.observe(9, root.join("c.txt"));
        pipeline.observe(9, root.join("d.txt")); // different content, same size

        let groups = pipeline.run();
        assert_eq!(groups.len(), 1, "expected 1 duplicate group");
        assert_eq!(groups[0].paths.len(), 3, "3 identical files");
        assert_eq!(groups[0].wasted_bytes, 18, "2 files * 9 bytes wasted");
    }

    #[test]
    fn no_duplicates_when_all_distinct() {
        let dir = tempfile::tempdir().expect("tmp");
        let root = dir.path();
        write_file(&root.join("a.txt"), b"aaa");
        write_file(&root.join("b.txt"), b"bbb");
        write_file(&root.join("c.txt"), b"ccc");

        let mut pipeline = DedupPipeline::new();
        pipeline.observe(3, root.join("a.txt"));
        pipeline.observe(3, root.join("b.txt"));
        pipeline.observe(3, root.join("c.txt"));

        let groups = pipeline.run();
        assert!(groups.is_empty(), "no duplicates expected");
    }

    #[test]
    fn ignores_files_with_distinct_sizes() {
        let dir = tempfile::tempdir().expect("tmp");
        let root = dir.path();
        write_file(&root.join("a.txt"), b"identical");
        write_file(&root.join("b.txt"), b"identical-and-longer");

        let mut pipeline = DedupPipeline::new();
        pipeline.observe(9, root.join("a.txt"));
        pipeline.observe(21, root.join("b.txt"));

        let groups = pipeline.run();
        assert!(groups.is_empty(), "different sizes can't be duplicates");
    }
}
