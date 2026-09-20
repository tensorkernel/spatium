// Stub JS entry for @disk-analyzer/napi.
// The real implementation is a `.node` native module built by `napi build`.
// This stub lets the Electron main process load the module via require()
// even when the native build hasn't been done (Phase 1 dev iteration).
//
// Per 08-RENDERER-REACT-APP.md §8.14: Mock IPC mode for agent iterability.
// When the real `.node` is built alongside this file, Node prefers the native
// (because the binary `disk-analyzer-napi.<platform>.node` is the canonical
// exports of the napi module; this `index.js` is the fallback).

function notBuilt(feature) {
  throw new Error(
    `@disk-analyzer/napi: '${feature}' is not available; the native module has not been built yet. ` +
    `Run \`pnpm --filter @disk-analyzer/napi build:debug\` to build it, ` +
    `or use the Electron main's Mock IPC mode.`,
  );
}

module.exports = {
  onScanEvent: async (_callback) => { notBuilt('onScanEvent'); },
  scanStart: async (_root, _options, _progressCallback) => { notBuilt('scanStart'); },
  scanCancel: () => false,
  getActiveScanId: () => null,
  getScanStatus: () => null,
  engineVersion: () => '0.2.0-stub',
};
