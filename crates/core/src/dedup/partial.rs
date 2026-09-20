//! Partial hash — fast non-cryptographic fingerprint.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.7. xxhash64 of head (4 KB) + middle (4 KB) +
//! tail (4 KB), or the full file if < 12 KB.
//!
//! Non-cryptographic: collisions are rare but possible; we re-check with full SHA-256.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use xxhash_rust::xxh64;

/// Compute the partial hash of a file. Returns `None` if the file can't be read.
///
/// Per `05-RUST-CORE-ENGINE.md` §5.7.
#[must_use]
pub fn partial_hash_of_file(path: &Path) -> Option<u64> {
    let mut file = File::open(path).ok()?;
    let metadata = std::fs::metadata(path).ok()?;
    let size = metadata.len();

    if size == 0 {
        return Some(0);
    }

    let chunk: usize = 4 * 1024; // 4 KB
    let mut hasher = xxh64::Xxh64::new(0);

    // If small enough, hash the whole file.
    if size <= (chunk as u64) * 3 {
        let mut buf = Vec::with_capacity(size as usize);
        file.read_to_end(&mut buf).ok()?;
        hasher.update(&buf);
        return Some(hasher.digest());
    }

    // Hash head (4 KB)
    let mut head = vec![0u8; chunk];
    file.read_exact(&mut head).ok()?;
    hasher.update(&head);

    // Hash tail (4 KB) — seek to end-chunk
    file.seek(SeekFrom::End(-(chunk as i64))).ok()?;
    let mut tail = vec![0u8; chunk];
    file.read_exact(&mut tail).ok()?;
    hasher.update(&tail);

    // Hash middle (4 KB) — seek to size/2 - chunk/2
    let mid_start = size / 2 - (chunk as u64) / 2;
    file.seek(SeekFrom::Start(mid_start)).ok()?;
    let mut mid = vec![0u8; chunk];
    file.read_exact(&mut mid).ok()?;
    hasher.update(&mid);

    Some(hasher.digest())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn small_file_hashes_full_content() {
        let dir = tempfile::tempdir().expect("tmp");
        let p = dir.path().join("small.txt");
        let mut f = std::fs::File::create(&p).expect("create");
        f.write_all(b"hello world").expect("write");
        drop(f);
        let h = partial_hash_of_file(&p).expect("hash");
        assert_ne!(h, 0);
    }

    #[test]
    fn large_file_hashes_head_tail_middle() {
        let dir = tempfile::tempdir().expect("tmp");
        let p = dir.path().join("large.bin");
        let mut f = std::fs::File::create(&p).expect("create");
        // 32 KB of distinct bytes so head/middle/tail differ
        let content: Vec<u8> = (0..32_768).map(|i| (i % 256) as u8).collect();
        f.write_all(&content).expect("write");
        drop(f);
        let h = partial_hash_of_file(&p).expect("hash");
        assert_ne!(h, 0);
    }

    #[test]
    fn identical_files_get_identical_hashes() {
        let dir = tempfile::tempdir().expect("tmp");
        let p1 = dir.path().join("a.bin");
        let p2 = dir.path().join("b.bin");
        let content: Vec<u8> = (0..50_000).map(|i| (i % 256) as u8).collect();
        std::fs::write(&p1, &content).expect("w");
        std::fs::write(&p2, &content).expect("w");
        let h1 = partial_hash_of_file(&p1).expect("h1");
        let h2 = partial_hash_of_file(&p2).expect("h2");
        assert_eq!(h1, h2, "identical files should hash identically");
    }

    #[test]
    fn differing_files_get_different_hashes() {
        let dir = tempfile::tempdir().expect("tmp");
        let p1 = dir.path().join("a.bin");
        let p2 = dir.path().join("b.bin");
        let mut content1: Vec<u8> = (0..50_000).map(|i| (i % 256) as u8).collect();
        content1[25_000] = 0xFF; // perturb the middle
        let content2: Vec<u8> = (0..50_000).map(|i| (i % 256) as u8).collect();
        std::fs::write(&p1, &content1).expect("w");
        std::fs::write(&p2, &content2).expect("w");
        let h1 = partial_hash_of_file(&p1).expect("h1");
        let h2 = partial_hash_of_file(&p2).expect("h2");
        assert_ne!(h1, h2, "differing files should hash differently");
    }

    #[test]
    fn empty_file_hashes_zero() {
        let dir = tempfile::tempdir().expect("tmp");
        let p = dir.path().join("empty");
        std::fs::File::create(&p).expect("create");
        let h = partial_hash_of_file(&p).expect("hash");
        assert_eq!(h, 0);
    }
}
