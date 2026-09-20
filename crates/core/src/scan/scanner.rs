//! The scan controller — start, cancel, status.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.3.1.
//!
//! Phase 1: single-threaded, synchronous traversal using `std::fs::read_dir`.
//! The worker is invoked on a dedicated thread (not rayon). Events flow
//! through a callback (rather than a tokio channel) because Phase 1 has no
//! async runtime.
//!
//! Phase 2 will replace this with a rayon worker pool + bounded tokio mpsc
//! channel + threadsafe-function callback to Node.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use parking_lot::Mutex;

use crate::aggregate::ArenaTree;
use crate::scan::event::{ReparseKind, ScanEvent, ScanPhase, ScanStatus};
use crate::scan::options::ScanOptions;
use crate::util::cancel::CancelToken;
use crate::util::paths;

/// Opaque scan identifier. Internally a u64 counter; serialized as a decimal
/// string when crossing the IPC boundary (per TS contract `ScanId = string`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ScanId(pub(crate) u64);

impl ScanId {
    #[must_use]
    pub fn to_string_value(self) -> String {
        self.0.to_string()
    }

    #[must_use]
    pub fn parse_str_lossy(s: &str) -> Option<Self> {
        s.parse::<u64>().ok().map(Self)
    }
}

impl std::fmt::Display for ScanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Errors returned by `ScanController::start`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScanStartError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("path not found: {0}")]
    PathNotFound(PathBuf),
    #[error("access denied: {0}")]
    AccessDenied(PathBuf),
    #[error("path requires elevation: {0}")]
    PathRequiresElevation(PathBuf),
    #[error("already scanning; cancel the active scan first")]
    AlreadyScanning,
    #[error("license required for this operation")]
    LicenseRequired,
    #[error("unknown error: {0}")]
    Unknown(String),
}

/// Callback invoked for each coalesced batch of events during a scan.
/// The aggregator invokes this from the worker thread.
///
/// Phase 1: the callback receives `Vec<ScanEvent>` (each event has an owned
/// `String` for the path component; the aggregator has NOT yet interned it).
/// Phase 2 will switch to interned `StringId`s + a separate "new strings"
/// channel.
pub type EventCallback = Box<dyn Fn(&[ScanEvent]) + Send + Sync>;

/// Progress callback invoked ~16 ms during enumeration.
pub type ProgressCallback = Box<dyn Fn(ProgressSnapshot) + Send + Sync>;

/// Snapshot of scan progress.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressSnapshot {
    pub scan_id: ScanId,
    pub phase: ScanPhase,
    pub files_scanned: u64,
    pub bytes_scanned: u64,
    pub bytes_allocated: u64,
    pub elapsed_ms: u64,
}

/// The scan controller. Owns the active scan's cancel token, arena tree,
/// and progress counters. Per `05-RUST-CORE-ENGINE.md` §5.3.1.
pub struct ScanController {
    active_id: Arc<AtomicU64>,
    cancel: Arc<CancelToken>,
    arena: Arc<Mutex<ArenaTree>>,
    files_scanned: Arc<AtomicU64>,
    bytes_scanned: Arc<AtomicU64>,
    bytes_allocated: Arc<AtomicU64>,
    started_at: Arc<Mutex<Option<Instant>>>,
    current_path: Arc<Mutex<Option<PathBuf>>>,
}

impl ScanController {
    /// Create a new controller with no active scan.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_id: Arc::new(AtomicU64::new(0)),
            cancel: Arc::new(CancelToken::new()),
            arena: Arc::new(Mutex::new(ArenaTree::new())),
            files_scanned: Arc::new(AtomicU64::new(0)),
            bytes_scanned: Arc::new(AtomicU64::new(0)),
            bytes_allocated: Arc::new(AtomicU64::new(0)),
            started_at: Arc::new(Mutex::new(None)),
            current_path: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns the active scan ID, if any.
    #[must_use]
    pub fn active_scan_id(&self) -> Option<ScanId> {
        let v = self.active_id.load(Ordering::Acquire);
        if v == 0 {
            None
        } else {
            Some(ScanId(v))
        }
    }

    /// Start a new scan. Returns the scan ID on success.
    ///
    /// Phase 1: synchronous — runs the scan on a dedicated thread; this method
    /// returns immediately with the scan ID, and the scan continues in the
    /// background. The provided `event_callback` is invoked from the worker
    /// thread for each batch.
    ///
    /// # Errors
    /// - `AlreadyScanning` if a scan is already active.
    /// - `PathNotFound` if `root` does not exist.
    /// - `AccessDenied` if `root` exists but is not readable.
    pub fn start(
        &self,
        root: PathBuf,
        options: ScanOptions,
        event_callback: EventCallback,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<ScanId, ScanStartError> {
        // Validate root path
        let canonical = paths::canonicalize(&root).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => ScanStartError::PathNotFound(root.clone()),
            std::io::ErrorKind::PermissionDenied => ScanStartError::AccessDenied(root.clone()),
            _ => ScanStartError::Unknown(format!("canonicalize failed: {e}")),
        })?;

        if !canonical.exists() {
            return Err(ScanStartError::PathNotFound(root));
        }

        // Atomically claim the active-scan slot.
        let prev = self
            .active_id
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| ScanStartError::AlreadyScanning)?;
        debug_assert_eq!(prev, 0);

        // Assign a real scan ID (monotonic from 1; the 1 we just claimed is a sentinel
        // to lock the slot. We bump once more to get a "real" ID.)
        let scan_id = ScanId(self.active_id.fetch_add(1, Ordering::AcqRel) + 1);

        // Reset state
        self.cancel.reset();
        {
            let mut arena = self.arena.lock();
            *arena = ArenaTree::new();
        }
        self.files_scanned.store(0, Ordering::Release);
        self.bytes_scanned.store(0, Ordering::Release);
        self.bytes_allocated.store(0, Ordering::Release);
        *self.started_at.lock() = Some(Instant::now());
        *self.current_path.lock() = Some(canonical.clone());

        // Spawn the worker thread.
        let cancel = self.cancel.clone();
        let arena = self.arena.clone();
        let files_scanned = self.files_scanned.clone();
        let bytes_scanned = self.bytes_scanned.clone();
        let bytes_allocated = self.bytes_allocated.clone();
        let started_at = self.started_at.clone();
        let current_path = self.current_path.clone();
        let active_id = self.active_id.clone();

        thread::Builder::new()
            .name(format!("diskanalyzer-scan-{scan_id}"))
            .spawn(move || {
                Self::run_worker(
                    scan_id,
                    canonical,
                    options,
                    cancel,
                    arena,
                    files_scanned,
                    bytes_scanned,
                    bytes_allocated,
                    started_at,
                    current_path,
                    event_callback,
                    progress_callback,
                );
                // Scan ended; release the slot.
                active_id.store(0, Ordering::Release);
            })
            .map_err(|e| ScanStartError::Unknown(format!("thread spawn failed: {e}")))?;

        Ok(scan_id)
    }

    /// Cancel the active scan (if any). Returns true if a scan was cancelled.
    ///
    /// Per `05-RUST-CORE-ENGINE.md` §5.5, cancellation latency is ≤10 ms in
    /// Phase 2 (rayon). Phase 1 (single worker) has cancellation latency
    /// bounded by one `read_dir` iteration (~ms).
    pub fn cancel(&self) -> bool {
        if self.active_id.load(Ordering::Acquire) == 0 {
            return false;
        }
        self.cancel.set();
        true
    }

    /// Snapshot of scan status. Returns `None` if no scan is active.
    #[must_use]
    pub fn status(&self) -> Option<ScanStatus> {
        let id = self.active_id.load(Ordering::Acquire);
        if id == 0 {
            return None;
        }
        let started = *self.started_at.lock();
        let elapsed_ms = started.map_or(0, |s| s.elapsed().as_millis() as u64);
        Some(ScanStatus {
            scan_id: ScanId(id),
            phase: if self.cancel.is_set() {
                ScanPhase::Aborted
            } else {
                ScanPhase::Enumerating
            },
            started_at: started.map_or(0, |_s| {
                // Approximate unix-millis; we don't track the wall clock here.
                // Phase 2 will use SystemTime::now() at start.
                0
            }),
            elapsed_ms,
            files_scanned: self.files_scanned.load(Ordering::Acquire),
            bytes_scanned: self.bytes_scanned.load(Ordering::Acquire),
            bytes_allocated: self.bytes_allocated.load(Ordering::Acquire),
            current_path: self.current_path.lock().clone(),
            error: if self.cancel.is_set() {
                Some("cancelled by user".to_string())
            } else {
                None
            },
        })
    }

    /// Get a shared handle to the arena tree (read-only).
    ///
    /// The arena is updated by the worker; readers should hold the lock briefly.
    #[must_use]
    pub fn arena_handle(&self) -> Arc<Mutex<ArenaTree>> {
        self.arena.clone()
    }

    // --- internal ---

    fn run_worker(
        scan_id: ScanId,
        root: PathBuf,
        options: ScanOptions,
        cancel: Arc<CancelToken>,
        arena: Arc<Mutex<ArenaTree>>,
        files_scanned: Arc<AtomicU64>,
        bytes_scanned: Arc<AtomicU64>,
        bytes_allocated: Arc<AtomicU64>,
        started_at: Arc<Mutex<Option<Instant>>>,
        current_path: Arc<Mutex<Option<PathBuf>>>,
        event_callback: EventCallback,
        progress_callback: Option<ProgressCallback>,
    ) {
        let started = *started_at.lock();
        let started_instant = started.unwrap_or_else(Instant::now);

        // Iterative traversal with a parent stack. Per `05-RUST-CORE-ENGINE.md` §5.3.2
        // (iterative not recursive — avoids stack overflow on deep trees).
        #[derive(Debug)]
        struct StackItem {
            path: PathBuf,
            parent_id: crate::aggregate::NodeId,
            depth: u32,
        }

        let mut stack: Vec<StackItem> = vec![StackItem {
            path: root.clone(),
            parent_id: 0,
            depth: 0,
        }];

        // Coalesced batch buffer. Flushes every ~16 ms or when full.
        let mut batch: Vec<ScanEvent> = Vec::with_capacity(1024);
        let mut last_flush = Instant::now();
        const FLUSH_INTERVAL_MS: u64 = 16;
        const BATCH_CAP: usize = 1024;

        // Ancestor set for loop detection (per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.2.1).
        let mut ancestors: Vec<PathBuf> = Vec::with_capacity(32);

        while let Some(item) = stack.pop() {
            if cancel.is_set() {
                break;
            }

            *current_path.lock() = Some(item.path.clone());

            let read_dir = match std::fs::read_dir(&item.path) {
                Ok(rd) => rd,
                Err(_) => {
                    // Per `05-RUST-CORE-ENGINE.md` §5.10: per-path errors are not fatal.
                    continue;
                }
            };

            for entry in read_dir {
                if cancel.is_set() {
                    break;
                }

                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let file_name = entry.file_name();
                let name = file_name.to_string_lossy().to_string();

                // Skip "." and ".."
                if name == "." || name == ".." {
                    continue;
                }

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let is_dir = metadata.is_dir();
                let logical = metadata.len();
                // Phase 1: allocated == logical. Phase 2: GetCompressedFileSizeName.
                let allocated = logical;

                // Phase 1: no reparse tag, no hard links, no sparse.
                let reparse_kind = ReparseKind::None;
                let hardlink_count = 1u16;
                let sparse = false;
                let file_id = 0u64;

                // Synthesize a minimal attrs bitmask (matches Win32 constants per `scan/event.rs`).
                let attrs: u32 = if is_dir { 0x0000_0010 } else { 0x0000_0000 };

                let mtime = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(0, |d| d.as_millis() as i64);
                let ctime = metadata
                    .created()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(0, |d| d.as_millis() as i64);

                // Apply include / exclude patterns (Phase 1: substring match)
                if !options.include_patterns.is_empty()
                    && !options
                        .include_patterns
                        .iter()
                        .any(|p| p.matches_path(&entry.path()))
                {
                    continue;
                }
                if options
                    .exclude_patterns
                    .iter()
                    .any(|p| p.matches_path(&entry.path()))
                {
                    continue;
                }

                // Apply min_size filter (files only)
                if !is_dir {
                    if let Some(min) = options.min_size_bytes {
                        if logical < min {
                            continue;
                        }
                    }
                }

                // Apply max_depth
                if let Some(max_depth) = options.max_depth {
                    if is_dir && item.depth + 1 > max_depth {
                        continue;
                    }
                }

                let event = ScanEvent {
                    scan_id,
                    parent_id: item.parent_id,
                    name,
                    logical,
                    allocated,
                    mtime,
                    ctime,
                    attrs,
                    reparse_kind,
                    hardlink_count,
                    sparse,
                    file_id,
                    name_id_hint: None, // Aggregator will intern and set this.
                };

                // Insert into arena (this also interns the name into the string pool).
                let (node_id, _name_id) = {
                    let mut a = arena.lock();
                    a.insert_file_or_dir(
                        &event.name,
                        event.parent_id,
                        is_dir,
                        logical,
                        allocated,
                        mtime,
                        ctime,
                        attrs,
                    )
                };

                files_scanned.fetch_add(1, Ordering::Relaxed);
                bytes_scanned.fetch_add(logical, Ordering::Relaxed);
                bytes_allocated.fetch_add(allocated, Ordering::Relaxed);

                // For directories, push to the traversal stack.
                if is_dir {
                    // Loop detection: only follow reparse points if options allow.
                    // Phase 1: no reparse detection via std::fs, so we always descend.
                    // We use the canonical-path ancestor set to detect OS-level loops
                    // (rare but possible via symlinks).
                    let child_path = entry.path();
                    let canonical = paths::canonicalize(&child_path).unwrap_or(child_path.clone());
                    if ancestors.iter().any(|a| a == &canonical) {
                        // Loop detected; skip this descent.
                        continue;
                    }
                    ancestors.push(canonical.clone());
                    stack.push(StackItem {
                        path: child_path,
                        parent_id: node_id,
                        depth: item.depth + 1,
                    });
                }

                batch.push(event);
                if batch.len() >= BATCH_CAP
                    || last_flush.elapsed().as_millis() >= FLUSH_INTERVAL_MS as u128
                {
                    event_callback(&batch);
                    if let Some(pc) = &progress_callback {
                        pc(ProgressSnapshot {
                            scan_id,
                            phase: ScanPhase::Enumerating,
                            files_scanned: files_scanned.load(Ordering::Relaxed),
                            bytes_scanned: bytes_scanned.load(Ordering::Relaxed),
                            bytes_allocated: bytes_allocated.load(Ordering::Relaxed),
                            elapsed_ms: started_instant.elapsed().as_millis() as u64,
                        });
                    }
                    batch.clear();
                    last_flush = Instant::now();
                }
            }

            // Pop the ancestor when we finish a subtree.
            if !ancestors.is_empty() {
                ancestors.pop();
            }
        }

        // Final flush.
        if !batch.is_empty() {
            event_callback(&batch);
        }
        if let Some(pc) = &progress_callback {
            pc(ProgressSnapshot {
                scan_id,
                phase: ScanPhase::Complete,
                files_scanned: files_scanned.load(Ordering::Relaxed),
                bytes_scanned: bytes_scanned.load(Ordering::Relaxed),
                bytes_allocated: bytes_allocated.load(Ordering::Relaxed),
                elapsed_ms: started_instant.elapsed().as_millis() as u64,
            });
        }
    }
}

impl Default for ScanController {
    fn default() -> Self {
        Self::new()
    }
}
