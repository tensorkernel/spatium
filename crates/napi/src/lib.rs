//! Node native binding for the DiskAnalyzer Rust core engine.
//!
//! Per `02-SYSTEM-ARCHITECTURE.md` §2.2: the Electron main process loads the Rust
//! core via this napi-rs module. It owns the `ScanController`, exposes a tiny
//! typed surface to Node, and bridges events via a `ThreadsafeFunction`.
//!
//! Per `07-ELECTRON-APP-SHELL.md` §7.4: every IPC handler validates its input
//! and does capability checks at Layer B (NOT in the renderer).
//!
//! Public surface (mirrors `packages/ipc/src/types.ts`):
//! - `onScanEvent(callback)` — register a callback invoked per batch.
//! - `scanStart(root, options, progressCallback?)` — start a scan; returns scan ID string.
//! - `scanCancel()` — cancel the active scan.
//! - `getActiveScanId()` — current scan ID (or null).
//! - `getScanStatus()` — status snapshot.
//!
//! Phase 1: events flow through the threadsafe function synchronously. Phase 2
//! will switch to a tokio mpsc + coalesced batches.

#![deny(rust_2018_idioms)]

use std::path::PathBuf;
use std::sync::Arc;

use napi::bindgen_prelude::BigInt;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;

use disk_analyzer_core::scan::event::ScanEvent;
use disk_analyzer_core::scan::options::ScanOptions;
use disk_analyzer_core::scan::scanner::{ProgressSnapshot, ScanController};
use parking_lot::Mutex;

// ─────────────────────────────────────────────────────────────────────
// Global state (single ScanController + single event callback).
// ─────────────────────────────────────────────────────────────────────

// Per `02-SYSTEM-ARCHITECTURE.md` §2.2: the Electron main owns ONE controller.
// Phase 2 will support multiple concurrent scans; Phase 1 is single-scan.

static CONTROLLER: once_cell::sync::Lazy<Arc<Mutex<ScanController>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(ScanController::new())));

static EVENT_CALLBACK: once_cell::sync::Lazy<Mutex<Option<ThreadsafeFunction<ScanEventDto>>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

// ─────────────────────────────────────────────────────────────────────
// Wire types (mirror the TS types in packages/ipc/src/types.ts)
// ─────────────────────────────────────────────────────────────────────

/// Event payload delivered to the JS callback. Mirrors `ScanEvent` in `packages/ipc/src/types.ts`.
#[napi(object)]
pub struct ScanEventDto {
    pub scan_id: String,
    pub parent_id: u32,
    pub name: String,
    pub logical: BigInt,
    pub allocated: BigInt,
    pub mtime: i64,
    pub ctime: i64,
    pub attrs: u32,
    pub reparse_kind: String,
    pub hardlink_count: u16,
    pub sparse: bool,
    pub file_id: BigInt,
}

impl From<ScanEvent> for ScanEventDto {
    fn from(e: ScanEvent) -> Self {
        Self {
            scan_id: e.scan_id.to_string_value(),
            parent_id: e.parent_id,
            name: e.name,
            logical: BigInt::from(e.logical),
            allocated: BigInt::from(e.allocated),
            mtime: e.mtime,
            ctime: e.ctime,
            attrs: e.attrs,
            reparse_kind: e.reparse_kind.as_str().to_string(),
            hardlink_count: e.hardlink_count,
            sparse: e.sparse,
            file_id: BigInt::from(e.file_id),
        }
    }
}

/// Scan options. Passed from JS; we parse strings to enums inside Rust.
#[napi(object)]
pub struct ScanOptionsDto {
    pub follow_reparse_points: String, // "never" | "all" | "only-junctions-and-symlinks"
    pub count_hard_links: String,      // "none" | "logical" | "allocated-once"
    pub hash_duplicates: bool,
    pub respect_gitignore: bool,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub min_size_bytes: Option<BigInt>,
    pub max_depth: Option<u32>,
    pub incremental_usn: bool,
    pub prior_scan_id: Option<String>,
}

impl From<&ScanOptionsDto> for ScanOptions {
    fn from(d: &ScanOptionsDto) -> Self {
        use disk_analyzer_core::scan::options::{FollowReparse, GlobPattern, HardLinkMode};
        Self {
            follow_reparse_points: FollowReparse::from_str_lossy(&d.follow_reparse_points),
            count_hard_links: HardLinkMode::from_str_lossy(&d.count_hard_links),
            hash_duplicates: d.hash_duplicates,
            respect_gitignore: d.respect_gitignore,
            include_patterns: d
                .include_patterns
                .iter()
                .map(|s| GlobPattern::new(s.as_str()))
                .collect(),
            exclude_patterns: d
                .exclude_patterns
                .iter()
                .map(|s| GlobPattern::new(s.as_str()))
                .collect(),
            min_size_bytes: d.min_size_bytes.as_ref().map(|b| b.get_u64().1),
            max_depth: d.max_depth,
            incremental_usn: d.incremental_usn,
            prior_scan_id: d.prior_scan_id.clone(),
        }
    }
}

/// Scan status snapshot.
#[napi(object)]
pub struct ScanStatusDto {
    pub scan_id: String,
    pub phase: String,
    pub elapsed_ms: BigInt,
    pub files_scanned: BigInt,
    pub bytes_scanned: BigInt,
    pub bytes_allocated: BigInt,
    pub current_path: Option<String>,
    pub error: Option<String>,
}

impl From<disk_analyzer_core::scan::event::ScanStatus> for ScanStatusDto {
    fn from(s: disk_analyzer_core::scan::event::ScanStatus) -> Self {
        Self {
            scan_id: s.scan_id.to_string_value(),
            phase: s.phase.as_str().to_string(),
            elapsed_ms: BigInt::from(s.elapsed_ms),
            files_scanned: BigInt::from(s.files_scanned),
            bytes_scanned: BigInt::from(s.bytes_scanned),
            bytes_allocated: BigInt::from(s.bytes_allocated),
            current_path: s.current_path.map(|p| p.to_string_lossy().to_string()),
            error: s.error,
        }
    }
}

/// Progress DTO mirroring the TS `ScanProgressEvent`.
#[napi(object)]
pub struct ScanProgressDto {
    pub scan_id: String,
    pub phase: String,
    pub files_scanned: BigInt,
    pub bytes_scanned: BigInt,
    pub bytes_allocated: BigInt,
    pub elapsed_ms: BigInt,
}

impl From<ProgressSnapshot> for ScanProgressDto {
    fn from(s: ProgressSnapshot) -> Self {
        Self {
            scan_id: s.scan_id.to_string_value(),
            phase: s.phase.as_str().to_string(),
            files_scanned: BigInt::from(s.files_scanned),
            bytes_scanned: BigInt::from(s.bytes_scanned),
            bytes_allocated: BigInt::from(s.bytes_allocated),
            elapsed_ms: BigInt::from(s.elapsed_ms),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Exported napi functions
// ─────────────────────────────────────────────────────────────────────

/// Register a callback for scan events. The callback is invoked from the worker
/// thread for each coalesced batch. Per `08-RENDERER-REACT-APP.md` §8.14, the
/// renderer subscribes via the preload's `diskanalyzer.scan.onBatch(cb)`.
#[napi]
pub fn on_scan_event(callback: ThreadsafeFunction<ScanEventDto>) -> Result<()> {
    let mut slot = EVENT_CALLBACK.lock();
    *slot = Some(callback);
    Ok(())
}

/// Start a scan. Returns the scan ID (string) on success.
#[napi]
pub fn scan_start(
    root: String,
    options: ScanOptionsDto,
    progress_callback: Option<ThreadsafeFunction<ScanProgressDto>>,
) -> Result<String> {
    let opts = ScanOptions::from(&options);

    // Snapshot the threadsafe function (clone the Arc-like handle) so we can
    // invoke it from the worker thread. We hold the EVENT_CALLBACK lock only
    // briefly here.
    let tsfn = {
        let slot = EVENT_CALLBACK.lock();
        slot.as_ref()
            .ok_or_else(|| {
                Error::new(
                    Status::GenericFailure,
                    "Event callback not registered; call onScanEvent() first".to_string(),
                )
            })?
            .clone()
    };

    // Build the event callback that converts ScanEvent -> ScanEventDto and
    // invokes the threadsafe function.
    let event_callback: Box<dyn Fn(&[ScanEvent]) + Send + Sync> = Box::new(move |batch| {
        for ev in batch {
            let dto = ScanEventDto::from(ev.clone());
            tsfn.call(Ok(dto), ThreadsafeFunctionCallMode::NonBlocking);
        }
    });

    let progress_cb = progress_callback.map(|pc| {
        Box::new(move |snap: ProgressSnapshot| {
            let dto = ScanProgressDto::from(snap);
            pc.call(Ok(dto), ThreadsafeFunctionCallMode::NonBlocking);
        }) as Box<dyn Fn(ProgressSnapshot) + Send + Sync>
    });

    let controller = CONTROLLER.clone();
    let ctrl = controller.lock();
    match ctrl.start(PathBuf::from(&root), opts, event_callback, progress_cb) {
        Ok(scan_id) => Ok(scan_id.to_string_value()),
        Err(e) => Err(Error::new(
            Status::GenericFailure,
            format!("scan_start failed: {e}"),
        )),
    }
}

/// Cancel the active scan. Returns true if a scan was active.
#[napi]
pub fn scan_cancel() -> bool {
    CONTROLLER.lock().cancel()
}

/// Get the active scan ID (string) or null.
#[napi]
pub fn get_active_scan_id() -> Option<String> {
    CONTROLLER
        .lock()
        .active_scan_id()
        .map(|id| id.to_string_value())
}

/// Get the status snapshot of the active scan (or null).
#[napi]
pub fn get_scan_status() -> Option<ScanStatusDto> {
    CONTROLLER.lock().status().map(Into::into)
}

/// Engine version (per `05-RUST-CORE-ENGINE.md` §5.14).
#[napi]
pub fn engine_version() -> String {
    disk_analyzer_core::ENGINE_VERSION.to_string()
}
