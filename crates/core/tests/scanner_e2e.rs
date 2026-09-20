//! End-to-end scanner test — Phase 1.
//!
//! Builds a small fixture directory, runs the scanner, verifies counts.

#![cfg(test)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use disk_analyzer_core::scan::{ScanController, ScanOptions};

#[test]
fn scans_a_fixture_directory() {
    // Build a fixture: /tmp/da-test/ + 3 files + 2 subdirs (with 3 files total).
    // Total entries scanned = 6 files + 2 dirs = 8.
    let root = tempfile::tempdir().expect("tempdir");
    let root_path = root.path().to_path_buf();
    std::fs::write(root_path.join("a.txt"), "a").expect("write");
    std::fs::write(root_path.join("b.txt"), "bb").expect("write");
    std::fs::write(root_path.join("c.txt"), "ccc").expect("write");
    let sub1 = root_path.join("sub1");
    std::fs::create_dir_all(&sub1).expect("mkdir");
    std::fs::write(sub1.join("d.txt"), "dddd").expect("write");
    std::fs::write(sub1.join("e.txt"), "eeeee").expect("write");
    let sub2 = root_path.join("sub2");
    std::fs::create_dir_all(&sub2).expect("mkdir");
    std::fs::write(sub2.join("f.txt"), "ffffff").expect("write");

    let controller = ScanController::new();
    let entries_seen = Arc::new(AtomicU64::new(0));
    let counter = entries_seen.clone();

    let scan_id = controller
        .start(
            root_path.clone(),
            ScanOptions::default(),
            Box::new(move |batch| {
                counter.fetch_add(batch.len() as u64, Ordering::Relaxed);
            }),
            None,
        )
        .expect("scan start");

    // Wait for the worker to finish (Phase 1: synchronous worker thread).
    while controller.active_scan_id().is_some() {
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    assert!(!scan_id.to_string_value().is_empty());
    let final_count = entries_seen.load(Ordering::Relaxed);
    assert_eq!(
        final_count, 8,
        "expected 8 entries (6 files + 2 dirs); got {}",
        final_count
    );
}

#[test]
fn cancels_mid_scan() {
    // Build a large fixture so we have time to cancel. We use an artificial
    // sleep in the event callback to guarantee the scan is still in-flight when
    // we cancel (avoids flakiness on fast tmpfs).
    let root = tempfile::tempdir().expect("tempdir");
    let root_path = root.path().to_path_buf();
    for i in 0..10_000 {
        let subdir = root_path.join(format!("sub-{}", i / 100));
        std::fs::create_dir_all(&subdir).expect("mkdir");
        std::fs::write(subdir.join(format!("file-{i}.txt")), "x").expect("write");
    }

    let controller = ScanController::new();
    let _scan_id = controller
        .start(
            root_path.clone(),
            ScanOptions::default(),
            Box::new(|_batch| {
                // Slow down the scan so we have time to cancel mid-flight.
                std::thread::sleep(std::time::Duration::from_millis(1));
            }),
            None,
        )
        .expect("scan start");

    // Wait for the worker to actually start (active_scan_id is Some).
    while controller.active_scan_id().is_none() {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    // Cancel immediately — the scan should still be in-flight thanks to the sleep.
    let cancelled = controller.cancel();
    assert!(cancelled, "cancel should return true");

    // Wait for the worker to wind down.
    while controller.active_scan_id().is_some() {
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

#[test]
fn returns_already_scanning_when_active() {
    let root = tempfile::tempdir().expect("tempdir");
    let root_path = root.path().to_path_buf();
    for i in 0..10_000 {
        let subdir = root_path.join(format!("sub-{}", i / 100));
        std::fs::create_dir_all(&subdir).expect("mkdir");
        std::fs::write(subdir.join(format!("file-{i}.txt")), "x").expect("write");
    }

    let controller = ScanController::new();
    let _first = controller
        .start(
            root_path.clone(),
            ScanOptions::default(),
            Box::new(|_batch| {
                // Slow down so the first scan is still active when we try the second.
                std::thread::sleep(std::time::Duration::from_millis(1));
            }),
            None,
        )
        .expect("scan start");

    // Wait for the worker to start.
    while controller.active_scan_id().is_none() {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    // Second start should fail because the first is active.
    let result = controller.start(
        root_path.clone(),
        ScanOptions::default(),
        Box::new(|_batch| {}),
        None,
    );
    use disk_analyzer_core::scan::ScanStartError;
    assert!(matches!(result, Err(ScanStartError::AlreadyScanning)));

    // Wait for the first to finish so we don't leave threads hanging.
    while controller.active_scan_id().is_some() {
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}
