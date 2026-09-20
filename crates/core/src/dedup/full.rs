//! Full hash — SHA-256 for the final duplicate-group check.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.7. We use SHA-256 (audited, collision-resistant)
//! only for files that survived the partial-hash grouping.

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Compute the SHA-256 hex digest of a file. Returns `None` if the file can't be read.
///
/// Per `05-RUST-CORE-ENGINE.md` §5.7. Use sparingly — only on partial-hash candidates.
#[must_use]
pub fn sha256_of_file(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024]; // 64 KB chunks
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let result = hasher.finalize();
    Some(hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn hashes_small_file() {
        let dir = tempfile::tempdir().expect("tmp");
        let p = dir.path().join("a.txt");
        let mut f = std::fs::File::create(&p).expect("create");
        f.write_all(b"hello").expect("write");
        drop(f);
        let h = sha256_of_file(&p).expect("hash");
        // SHA-256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        assert_eq!(
            h,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn identical_files_get_identical_hashes() {
        let dir = tempfile::tempdir().expect("tmp");
        let p1 = dir.path().join("a.txt");
        let p2 = dir.path().join("b.txt");
        std::fs::write(&p1, b"identical content").expect("w");
        std::fs::write(&p2, b"identical content").expect("w");
        let h1 = sha256_of_file(&p1).expect("h1");
        let h2 = sha256_of_file(&p2).expect("h2");
        assert_eq!(h1, h2);
    }

    #[test]
    fn differing_files_get_different_hashes() {
        let dir = tempfile::tempdir().expect("tmp");
        let p1 = dir.path().join("a.txt");
        let p2 = dir.path().join("b.txt");
        std::fs::write(&p1, b"content a").expect("w");
        std::fs::write(&p2, b"content b").expect("w");
        let h1 = sha256_of_file(&p1).expect("h1");
        let h2 = sha256_of_file(&p2).expect("h2");
        assert_ne!(h1, h2);
    }
}
