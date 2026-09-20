//! Sparse-file detection.
//!
//! Per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.4.
//!
//! On Windows: `FILE_ATTRIBUTE_SPARSE_FILE = 0x00000200` from `WIN32_FIND_DATAW.dwFileAttributes`.
//! On non-Windows: always `false` (std::fs doesn't expose this).

use std::path::Path;

/// True if the file is sparse (NTFS sparse-file flag set).
///
/// Per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.4.
#[must_use]
pub fn is_sparse(path: &Path) -> bool {
    #[cfg(windows)]
    {
        // Phase 2 production: check FILE_ATTRIBUTE_SPARSE_FILE.
        // For the simulated Windows build, fall through to false.
        if let Some(true) = is_sparse_real(path) {
            return true;
        }
    }
    let _ = path; // unused on non-Windows
    false
}

#[cfg(windows)]
fn is_sparse_real(path: &Path) -> Option<bool> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::{GetFileAttributesW, FILE_ATTRIBUTE_SPARSE_FILE};

    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);
    let attrs = unsafe { GetFileAttributesW(windows::core::PCWSTR(wide.as_ptr())) };
    if attrs == u32::MAX {
        return None;
    }
    Some((attrs & FILE_ATTRIBUTE_SPARSE_FILE.0) != 0)
}
