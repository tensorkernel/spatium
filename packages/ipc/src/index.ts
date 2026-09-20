// Public entry point for @disk-analyzer/ipc.
// Per 02-SYSTEM-ARCHITECTURE.md §2.7, this is the canonical IPC contract.

export * from './types.js';

// Re-export the version constant for runtime sanity checks.
export { IPC_CONTRACT_VERSION } from './types.js';
export type { DiskAnalyzerAPI } from './types.js';
