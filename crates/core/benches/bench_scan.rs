//! Phase 1 benchmark for the single-threaded scanner.
//!
//! Per `04-PHASES-OVERVIEW.md` §4.3 deliverable: a `criterion` benchmark
//! scanning a 100k-file fixture, measuring throughput, RSS.
//!
//! Phase 1: we synthesize a tempdir with N nested files and time the scan.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use disk_analyzer_core::aggregate::ArenaTree;
use disk_analyzer_core::scan::{ScanController, ScanOptions};
use parking_lot::Mutex;

fn make_fixture(n_files: usize) -> std::path::PathBuf {
    let dir = tempfile::tempdir().expect("tempdir").into_path();
    for i in 0..n_files {
        let subdir = dir.join(format!("sub-{}", i / 1000));
        std::fs::create_dir_all(&subdir).expect("mkdir");
        let path = subdir.join(format!("file-{i}.txt"));
        std::fs::write(&path, "x".repeat(16)).expect("write");
    }
    dir
}

fn bench_scan_1k(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan");
    group.throughput(Throughput::Elements(1000));
    group.bench_function("1k-files", |b| {
        let fixture = make_fixture(1000);
        b.iter(|| {
            let controller = ScanController::new();
            let counter = Arc::new(AtomicU64::new(0));
            let c2 = counter.clone();
            let opts = ScanOptions::default();
            controller
                .start(
                    fixture.clone(),
                    opts,
                    Box::new(move |_| {}),
                    Some(Box::new(move |_| {
                        c2.fetch_add(1, Ordering::Relaxed);
                    })),
                )
                .expect("scan start");
            // Wait for completion (Phase 1: synchronous on the worker thread).
            // The worker joins on drop; we poll until the active slot is released.
            while controller.active_scan_id().is_some() {
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench_scan_1k);
criterion_main!(benches);
