// Quick smoke test for the @disk-analyzer/napi native module.
// Verifies the binding loads and exposes the expected functions.
//
// Per 02-SYSTEM-ARCHITECTURE.md §2.7, the TS definitions win; napi-derive
// auto-converts Rust snake_case to JS camelCase. The default `#[napi]` macro
// makes functions synchronous (the Rust core spawns its own thread internally,
// so scanStart returns immediately with the scan_id).

const napi = require('./index.js');

console.log('engineVersion():', napi.engineVersion());
console.log('getActiveScanId():', napi.getActiveScanId());
console.log('getScanStatus():', napi.getScanStatus());

// Register an event callback before starting a scan.
// Per crates/napi/index.d.ts: onScanEvent takes a callback whose signature is
// (err: Error | null, arg: ScanEventDto) => any — the first arg is always an
// error (null on success). This is the napi-rs ThreadsafeFunction convention.
napi.onScanEvent((_err, event) => {
  console.log('  [event]', event.name, Number(event.logical), 'bytes');
});

// scanStart is synchronous (returns the scan_id string immediately; the
// worker runs on a dedicated thread inside the Rust core).
let scanId;
try {
  scanId = napi.scanStart(
    process.cwd(),
    {
      followReparsePoints: 'only-junctions-and-symlinks',
      countHardLinks: 'none',
      hashDuplicates: false,
      respectGitignore: false,
      includePatterns: [],
      excludePatterns: [],
      // Per crates/napi/index.d.ts, minSizeBytes and priorScanId are optional;
      // we omit them. maxDepth is provided as a number.
      maxDepth: 2,
      incrementalUsn: false,
    },
    null,
  );
  console.log('scanStart returned scanId:', scanId);
} catch (err) {
  console.error('scanStart threw:', err?.message || err);
  process.exit(1);
}

// Poll until the scan finishes.
const poll = () => {
  const status = napi.getScanStatus();
  if (status === null) {
    console.log('scan complete.');
    process.exit(0);
  } else {
    console.log('status:', status.phase, 'files:', Number(status.filesScanned));
    setTimeout(poll, 100);
  }
};
setTimeout(poll, 100);
