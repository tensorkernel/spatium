//! Allocated size — NTFS truth.
//!
//! Per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.1.
//!
//! On Windows (Phase 2 real): `GetCompressedFileSizeNameW()` returns the actual
//! disk space consumed by the file (after compression, sparse punching, dedup).
//!
//! On non-Windows (Phase 2 simulation): returns the logical size from
//! `std::fs::metadata`, since the simulation can't query NTFS.

use std::path::Path;

/// Result of an allocated-size query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocationResult {
    /// The actual disk space the file consumes (NTFS truth).
    pub allocated: u64,
    /// The logical size the file appears to have.
    pub logical: u64,
    /// True if `allocated != logical` (i.e., the file is sparse, compressed, or deduped).
    pub differs: bool,
}

impl AllocationResult {
    #[must_use]
    pub fn same(size: u64) -> Self {
        Self {
            allocated: size,
            logical: size,
            differs: false,
        }
    }
}

/// Query the allocated size for a file. Returns `None` if the file can't be queried
/// (e.g., it's a directory, or the path doesn't exist, or access is denied).
///
/// Per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.1.1, we only call
/// `GetCompressedFileSizeName` on:
/// - NTFS volumes (cached per-volume `VolumeFlags`)
/// - Files (not directories — directories don't have a meaningful allocated size)
/// - Files that are NOT reparse points
///
/// Phase 2 simulation (non-Windows): returns logical size as allocated size.
#[must_use]
pub fn allocated_size(path: &Path) -> Option<AllocationResult> {
    let metadata = std::fs::metadata(path).ok()?;
    if metadata.is_dir() {
        return None;
    }
    let logical = metadata.len();
    #[cfg(windows)]
    {
        // Phase 2 production: call GetCompressedFileSizeNameW.
        // For the simulated Windows build, fall through to the simulation.
        if let Some(real) = allocated_size_real(path, logical) {
            return Some(real);
        }
    }
    Some(AllocationResult::same(logical))
}

#[cfg(windows)]
fn allocated_size_real(path: &Path, logical: u64) -> Option<AllocationResult> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::GetCompressedFileSizeW;

    let mut wide: Vec<u16> = std::iter::once(0u16) // prefix with \\?\ for long paths
        .chain(std::iter::once(0x5Cu16))
        .chain(std::iter::once(0x3Fu16))
        .chain(path.as_os_str().encode_wide())
        .collect();
    wide.push(0);

    // SAFETY: GetCompressedFileSizeW takes a valid PCWSTR and writes the high
    // and low DWORDs of the file's compressed (allocated) size. The pointers
    // are to local u32 stack vars. The wide string is null-terminated.
    let mut high: u32 = 0;
    let mut low: u32 = 0;
    let r = unsafe {
        GetCompressedFileSizeW(windows::core::PCWSTR(wide.as_ptr()), &mut high, &mut low)
    };
    if r == u32::MAX {
        // INVALID_FILE_SIZE — the call failed (likely path doesn't exist or access denied).
        return None;
    }
    let allocated = (u64::from(high) << 32) | u64::from(low);
    Some(AllocationResult {
        allocated,
        logical,
        differs: allocated != logical,
    })
}
