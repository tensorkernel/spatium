//! Hard-link detection.
//!
//! Per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.3.
//!
//! On Windows (Phase 2 real): `GetFileInformationByHandleEx(FileIdInfo)` returns
//! the file's 128-bit NTFS file reference number. Two paths with the same
//! `file_id` are hard links to the same content.
//!
//! On non-Windows (Phase 2 simulation): always returns `HardlinkInfo { count: 1 }`
//! since `std::fs` can't detect hard links.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardlinkInfo {
    /// Number of hard links pointing to the same content. 1 = no extra links.
    pub count: u16,
    /// NTFS file reference number (0 on non-Windows).
    pub file_id: u64,
}

impl Default for HardlinkInfo {
    fn default() -> Self {
        Self {
            count: 1,
            file_id: 0,
        }
    }
}

/// Query the hard-link count for a file. Returns `None` if the file can't be queried.
///
/// Per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.3.1.
#[must_use]
pub fn hardlink_count(path: &Path) -> Option<HardlinkInfo> {
    let metadata = std::fs::metadata(path).ok()?;
    if metadata.is_dir() {
        return None;
    }
    #[cfg(windows)]
    {
        if let Some(real) = hardlink_count_real(path) {
            return Some(real);
        }
    }
    // Non-Windows simulation: hard links aren't detectable.
    Some(HardlinkInfo::default())
}

#[cfg(windows)]
fn hardlink_count_real(path: &Path) -> Option<HardlinkInfo> {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Storage::FileSystem::{FileIdInfo, GetFileInformationByHandleEx};

    let file = std::fs::File::open(path).ok()?;
    let handle = file.as_raw_handle() as windows::Win32::Foundation::HANDLE;
    let mut info: FileIdInfo = unsafe { std::mem::zeroed() };
    // SAFETY: GetFileInformationByHandleEx writes into a properly-aligned buffer
    // we just zeroed. The handle is valid (just opened) and shared read-only.
    let ok = unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileIdInfo,
            Some(&mut info as *mut _ as *mut std::ffi::c_void),
            std::mem::size_of::<FileIdInfo>() as u32,
        )
    };
    if !ok.as_bool() {
        return None;
    }
    // FileId is 128 bits; we use the lower 64 as a stable hash.
    let file_id = u64::from_le_bytes(info.FileId.Identifier[0..8].try_into().unwrap_or([0u8; 8]));
    let count = info.NumberOfLinks as u16;
    Some(HardlinkInfo { count, file_id })
}
