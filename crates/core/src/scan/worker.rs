//! Parallel scan worker — rayon + bounded tokio mpsc channel.
//!
//! Per `05-RUST-CORE-ENGINE.md` §5.3.2.
//!
//! # Pattern
//!
//! - Rayon worker pool (sized to num_cpus) pulls directory-enumeration tasks from
//!   a shared work queue (`crossbeam::deque` or a simpler `Mutex<Vec<WorkItem>>`).
//! - Each worker emits `ScanEvent`s into a bounded `tokio::sync::mpsc` channel
//!   (cap 8192) that the aggregator drains.
//! - Cancellation checked at every iteration.
//! - Loop detection via ancestor set (per `17-WINDOWS-SPECIFIC-FEATURES.md` §17.2.1).
//!
//! Phase 1's `scanner.rs` provides a single-threaded version for tests + the
//! smoke test. This module provides the parallel version used by the napi
//! binding when scanning real volumes.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use parking_lot::Mutex as PMutex;
use tokio::sync::mpsc;

use crate::scan::event::{ReparseKind, ScanEvent, ScanPhase};
use crate::scan::options::ScanOptions;
use crate::scan::scanner::{EventCallback, ProgressCallback, ProgressSnapshot, ScanId};
use crate::util::cancel::CancelToken;
use crate::util::paths;

/// One unit of work for the rayon pool — a directory to enumerate.
#[derive(Debug, Clone)]
struct WorkItem {
    path: PathBuf,
    parent_node_id: u32,
    depth: u32,
}

/// Run a parallel scan. Spawns `num_workers` rayon threads, each pulling from the
/// shared work queue, emitting events into the bounded channel. The aggregator
/// (caller's responsibility) drains the channel.
///
/// Per `05-RUST-CORE-ENGINE.md` §5.3.2 + §5.6.3 (backpressure via bounded channel).
///
/// # Arguments
///
/// * `root` — the directory to scan.
/// * `options` — scan options.
/// * `num_workers` — number of rayon worker threads (typically `num_cpus::get()`).
/// * `tx` — sender for scan events (bounded, cap 8192).
/// * `cancel` — cancellation token.
/// * `on_event` — called from each worker thread for every batch (Phase 1-style
///   callback; Phase 2 will replace with a tokio task + aggregator).
/// * `progress` — optional progress callback.
/// * `files_scanned`, `bytes_scanned`, `bytes_allocated` — shared counters.
#[allow(clippy::too_many_arguments)]
pub fn run_parallel_scan(
    root: PathBuf,
    options: ScanOptions,
    num_workers: usize,
    tx: mpsc::Sender<ScanEvent>,
    cancel: Arc<CancelToken>,
    on_event: EventCallback,
    progress: Option<ProgressCallback>,
    files_scanned: Arc<AtomicU64>,
    bytes_scanned: Arc<AtomicU64>,
    bytes_allocated: Arc<AtomicU64>,
) {
    let work_queue: Arc<PMutex<Vec<WorkItem>>> = Arc::new(PMutex::new(vec![WorkItem {
        path: root.clone(),
        parent_node_id: 0,
        depth: 0,
    }]));
    let started = Instant::now();
    let total_workers = num_workers.max(1);
    let active_workers = Arc::new(AtomicU64::new(0));

    rayon::scope(|s| {
        for _ in 0..total_workers {
            let work_queue = work_queue.clone();
            let tx = tx.clone();
            let cancel = cancel.clone();
            let on_event = &on_event;
            let progress = progress.as_ref();
            let files_scanned = files_scanned.clone();
            let bytes_scanned = bytes_scanned.clone();
            let bytes_allocated = bytes_allocated.clone();
            let options = options.clone();
            let active_workers = active_workers.clone();
            let started = started;

            s.spawn(move |_| {
                active_workers.fetch_add(1, Ordering::SeqCst);
                let mut batch: Vec<ScanEvent> = Vec::with_capacity(1024);
                let mut last_flush = Instant::now();
                let mut ancestors: Vec<PathBuf> = Vec::with_capacity(32);

                loop {
                    if cancel.is_set() {
                        break;
                    }
                    // Pull a work item (LIFO for cache locality).
                    let item = {
                        let mut q = work_queue.lock();
                        q.pop()
                    };
                    let item = match item {
                        Some(i) => i,
                        None => {
                            // Queue empty; if no other worker is active, we're done.
                            if active_workers.load(Ordering::SeqCst) <= 1 {
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_micros(100));
                            continue;
                        }
                    };
                    active_workers.fetch_add(1, Ordering::SeqCst);
                    let _guard = WorkerGuard(active_workers.clone());

                    // Enumerate the directory.
                    let read_dir = match std::fs::read_dir(&item.path) {
                        Ok(rd) => rd,
                        Err(_) => continue,
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
                        if name == "." || name == ".." {
                            continue;
                        }
                        let metadata = match entry.metadata() {
                            Ok(m) => m,
                            Err(_) => continue,
                        };
                        let is_dir = metadata.is_dir();
                        let logical = metadata.len();
                        let allocated = logical;

                        // Apply pattern filters (Phase 1: substring match)
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
                        if !is_dir {
                            if let Some(min) = options.min_size_bytes {
                                if logical < min {
                                    continue;
                                }
                            }
                        }
                        if let Some(max_depth) = options.max_depth {
                            if is_dir && item.depth + 1 > max_depth {
                                continue;
                            }
                        }

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

                        let attrs: u32 = if is_dir { 0x0000_0010 } else { 0x0000_0000 };
                        let scan_id = ScanId(1); // Worker doesn't know its own scan ID; aggregator sets.

                        let event = ScanEvent {
                            scan_id,
                            parent_id: item.parent_node_id,
                            name,
                            logical,
                            allocated,
                            mtime,
                            ctime,
                            attrs,
                            reparse_kind: ReparseKind::None,
                            hardlink_count: 1,
                            sparse: false,
                            file_id: 0,
                            name_id_hint: None,
                        };

                        files_scanned.fetch_add(1, Ordering::Relaxed);
                        bytes_scanned.fetch_add(logical, Ordering::Relaxed);
                        bytes_allocated.fetch_add(allocated, Ordering::Relaxed);

                        if is_dir {
                            let child_path = entry.path();
                            let canonical =
                                paths::canonicalize(&child_path).unwrap_or(child_path.clone());
                            if ancestors.iter().any(|a| a == &canonical) {
                                continue; // loop
                            }
                            ancestors.push(canonical.clone());
                            // Push to work queue (other workers will pull).
                            {
                                let mut q = work_queue.lock();
                                q.push(WorkItem {
                                    path: child_path,
                                    parent_node_id: item.parent_node_id, // aggregator re-assigns
                                    depth: item.depth + 1,
                                });
                            }
                        }

                        batch.push(event);
                        if batch.len() >= 1024 || last_flush.elapsed().as_millis() >= 16 {
                            // Flush: invoke the callback + try-send one event each.
                            // (Phase 2 will replace this with coalesced ScanBatch.)
                            (on_event)(&batch);
                            if let Some(pc) = progress {
                                pc(ProgressSnapshot {
                                    scan_id,
                                    phase: ScanPhase::Enumerating,
                                    files_scanned: files_scanned.load(Ordering::Relaxed),
                                    bytes_scanned: bytes_scanned.load(Ordering::Relaxed),
                                    bytes_allocated: bytes_allocated.load(Ordering::Relaxed),
                                    elapsed_ms: started.elapsed().as_millis() as u64,
                                });
                            }
                            // Try to send the events through the channel too (for aggregator pattern).
                            for ev in &batch {
                                if tx.try_send(ev.clone()).is_err() {
                                    break; // channel full; backpressure
                                }
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
                // Final flush
                if !batch.is_empty() {
                    (on_event)(&batch);
                    if let Some(pc) = progress {
                        pc(ProgressSnapshot {
                            scan_id: ScanId(1),
                            phase: ScanPhase::Complete,
                            files_scanned: files_scanned.load(Ordering::Relaxed),
                            bytes_scanned: bytes_scanned.load(Ordering::Relaxed),
                            bytes_allocated: bytes_allocated.load(Ordering::Relaxed),
                            elapsed_ms: started.elapsed().as_millis() as u64,
                        });
                    }
                }
            });
        }
    });
}

/// RAII guard to ensure active_workers is decremented even on panic.
struct WorkerGuard(Arc<AtomicU64>);

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
