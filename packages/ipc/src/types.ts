// ============================================================================
// DiskAnalyzer — Canonical IPC Contract (v0.1)
// ============================================================================
// Per 02-SYSTEM-ARCHITECTURE.md §2.7:
//   This file is the single source of truth for the IPC contract between the
//   renderer (Layer A) and the Electron main + Rust core (Layer B/C).
//   The renderer NEVER imports Electron directly; it only sees `window.diskanalyzer`.
//   The Rust core mirrors these types via napi-rs; in case of conflict, the TS
//   definitions win because they are what the renderer sees.
//
// Per 14-LICENSING-CLIENT.md §14.2 (capability check pattern):
//   Capability checks happen at Layer B (Electron main), NOT in the renderer.
//   The renderer only DISPLAYS the current license status; it does not gate.
//
// Version: 0.1.0 (Phase 0)
// Status:   FROZEN at this version. Any change requires a Senior Dev +
//           Security Researcher review per §4.2 of 04-PHASES-OVERVIEW.md.
// ============================================================================

export const IPC_CONTRACT_VERSION = '0.1.0' as const;

// ============================================================================
// Common types
// ============================================================================

/** Identifier for a scan session. UUIDv4 string. */
export type ScanId = string;

/** Identifier for a node in the Rust arena tree (u32 in Rust; number in TS). */
export type NodeId = number;

/** Identifier for an interned path component (u32 in Rust; number in TS). */
export type StringId = number;

/** Identifier for an interned file extension (u32 in Rust; number in TS). */
export type ExtensionId = number;

/** File-type category for color-coding (see 09-DESIGN-SYSTEM.md §9.2.3). */
export type FileCategory =
  | 'documents'
  | 'code'
  | 'system'
  | 'media'
  | 'caches'
  | 'downloads'
  | 'userdata'
  | 'apps'
  | 'other';

/** Volume / drive information (per 17-WINDOWS-SPECIFIC-FEATURES.md §17.9). */
export interface VolumeInfo {
  guid: string; // Windows volume GUID, e.g. "\\\\?\\Volume{...}"
  driveLetters: string[]; // ['C:'] (can be empty for unmounted volumes)
  label: string; // user-assigned volume label
  fsType: 'NTFS' | 'ReFS' | 'FAT32' | 'exFAT' | 'CDFS' | 'UDF' | 'Network' | 'Unknown';
  totalBytes: bigint; // total disk size
  freeBytes: bigint; // free space (at time of query)
  isRemovable: boolean;
  isNetwork: boolean; // surface "scan not supported" gracefully per 00 §0.7
  isSystem: boolean; // contains the boot partition
}

// ============================================================================
// Scan — request/response + events
// ============================================================================

/** Scan options passed from the renderer. Mirrors Rust `ScanOptions`. */
export interface ScanOptions {
  followReparsePoints: FollowReparse;
  countHardLinks: HardLinkMode;
  hashDuplicates: boolean;
  respectGitignore: boolean;
  includePatterns: GlobPattern[];
  excludePatterns: GlobPattern[];
  minSizeBytes: number | null;
  maxDepth: number | null;
  incrementalUsn: boolean; // Pro-only
  priorScanId: ScanId | null; // for diff (Pro-only)
}

/** Whether to descend into reparse points (junctions, symlinks, etc.). */
export type FollowReparse = 'never' | 'all' | 'only-junctions-and-symlinks';

/** How to account for hard links (see 17-WINDOWS-SPECIFIC-FEATURES.md §17.3). */
export type HardLinkMode = 'none' | 'logical' | 'allocated-once';

/** Glob pattern, e.g. '*.tmp' or 'node_modules/**'. */
export type GlobPattern = string;

/** Request to start a new scan. */
export interface ScanStartRequest {
  root: string; // e.g. 'C:\\' or 'C:\\Users\\jane'
  options: ScanOptions;
}

/** Response from scan.start. */
export type ScanStartResponse =
  | { ok: true; scanId: ScanId }
  | { ok: false; error: ScanStartError; details?: unknown };

/** Errors from scan.start (per 02-SYSTEM-ARCHITECTURE.md §2.7). */
export type ScanStartError =
  | 'INVALID_REQUEST' // failed Zod validation
  | 'PATH_NOT_FOUND'
  | 'ACCESS_DENIED'
  | 'PATH_REQUIRES_ELEVATION' // needs privileged helper
  | 'ALREADY_SCANNING' // active scan; cancel first
  | 'LICENSE_REQUIRED' // capability check at Layer B failed
  | 'UNKNOWN_ERROR';

/** Scan phase (per 05-RUST-CORE-ENGINE.md §5.14). */
export type ScanPhase =
  | 'enumerating'
  | 'aggregating'
  | 'hashing'
  | 'persisting'
  | 'complete'
  | 'aborted';

/** Scan status snapshot. */
export interface ScanStatus {
  scanId: ScanId;
  phase: ScanPhase;
  startedAt: number; // unix millis
  elapsedMs: number;
  filesScanned: number;
  bytesScanned: number; // logical total
  bytesAllocated: number; // allocated total (NTFS truth)
  currentPath: string | null;
  error: string | null; // populated if phase === 'aborted'
}

/** Coalesced scan progress event (~60Hz). */
export interface ScanProgressEvent {
  scanId: ScanId;
  phase: ScanPhase;
  filesScanned: number;
  bytesScanned: number;
  bytesAllocated: number;
  elapsedMs: number;
  currentPath?: string;
}

/**
 * Coalesced batch event. The renderer merges these into its in-memory model.
 * Per 05-RUST-CORE-ENGINE.md §5.14: the String field on a node is sent only
 * the first time a path component is seen; subsequent events reference the
 * StringId. The renderer maintains its own string pool mirroring Rust's.
 */
export interface ScanBatchEvent {
  scanId: ScanId;
  /** New string-pool entries to intern in the renderer. */
  newStrings: { id: StringId; value: string }[];
  /** New extension-pool entries (for the file-type donut). */
  newExtensions: { id: ExtensionId; ext: string; category: FileCategory }[];
  /** New / updated tree nodes (additive). */
  nodeAdditions: TreeNodeDelta[];
  /** Snapshot of running totals (per-directory cumulative size). */
  directoryTotals: {
    nodeId: NodeId;
    logicalBytes: bigint;
    allocatedBytes: bigint;
    fileCount: number;
  }[];
  /** Top-N largest files observed so far (renderer can swap this wholesale). */
  topN: { nodeId: NodeId; logicalBytes: bigint; allocatedBytes: bigint }[];
}

/** A single node delta (additive). */
export interface TreeNodeDelta {
  nodeId: NodeId;
  parentId: NodeId;
  nameId: StringId;
  isDirectory: boolean;
  logicalBytes: bigint;
  allocatedBytes: bigint;
  mtime: number; // unix millis
  ctime: number; // unix millis
  attrs: number; // Win32 file attributes
  reparseKind: ReparseKind;
  hardlinkCount: number;
  sparse: boolean;
  fileId: number; // NTFS file ref; 0 if unavailable
  extensionId: ExtensionId | null;
}

/** Mirrors Rust `ReparseKind` per 05-RUST-CORE-ENGINE.md §5.14. */
export type ReparseKind =
  | 'none'
  | 'junction'
  | 'symlink'
  | 'mountpoint'
  | 'hsm' // Hierarchical Storage Manager
  | 'sis' // Single Instance Storage
  | 'dedup' // Data Deduplication
  | 'appxlink'
  | 'loop' // detected a reparse loop and did not descend
  | 'other';

/** Emitted when a scan reaches the 'complete' phase. */
export interface ScanCompleteEvent {
  scanId: ScanId;
  startedAt: number;
  completedAt: number;
  durationMs: number;
  filesScanned: number;
  bytesScanned: number;
  bytesAllocated: number;
  accessDeniedCount: number;
  notFoundCount: number;
}

/** Emitted when a scan is aborted (cancel or fatal error). */
export interface ScanErrorEvent {
  scanId: ScanId;
  phase: ScanPhase;
  error: string;
  partialFilesScanned: number;
  partialBytesScanned: number;
}

// ============================================================================
// License — request/response + events
// Per 13-LICENSING-SERVER.md + 14-LICENSING-CLIENT.md
// ============================================================================

export type LicenseTier = 'free' | 'pro_yearly' | 'pro_lifetime';

export type ActivationError =
  | 'INVALID_KEY_FORMAT'
  | 'KEY_NOT_FOUND'
  | 'KEY_REVOKED'
  | 'KEY_EXPIRED'
  | 'DEVICE_LIMIT_REACHED'
  | 'DEVICE_ALREADY_BOUND_OTHER_KEY'
  | 'RATE_LIMITED'
  | 'NETWORK_ERROR'
  | 'SIGNATURE_VERIFICATION_FAILED';

export type ActivationResult =
  | { ok: true; entitlement: Entitlement }
  | { ok: false; error: ActivationError; message?: string };

/** The signed entitlement payload. The client verifies the signature. */
export interface Entitlement {
  v: 1;
  licenseId: string;
  productId: 'disk-analyzer';
  tier: Exclude<LicenseTier, 'free'>;
  customerId: string;
  customerEmail: string;
  features: string[];
  issuedAt: number; // unix seconds
  expiresAt: number | null; // null = lifetime
  graceUntil: number; // hard expiry + 30 days offline grace
  deviceFingerprint: string; // sha256 hex
  deviceId: string;
  maxDevices: number;
  channel: 'stable' | 'beta' | 'nightly';
  minAppVersion: string;
  maxAppVersion: string | null;
  nonce: string; // 16-byte hex
}

/** Wire format returned by the licensing server. */
export interface WireEntitlement {
  entitlementB64: string;
  signatureB64: string;
  alg: 'Ed25519';
  ver: 1;
}

export type LicenseStatus =
  | { kind: 'free' }
  | {
      kind: 'pro';
      tier: 'pro_yearly' | 'pro_lifetime';
      expiresAt: number | null;
      deviceId: string;
      lastRevalidatedAt: number;
      offline: boolean;
      features: string[];
    }
  | { kind: 'expired'; reason: 'revoked' | 'server_unreachable_grace_expired' | 'tier_expired' }
  | { kind: 'error'; message: string };

// ============================================================================
// Database queries — read-only, untrusted-by-renderer (per 02-SYSTEM-ARCHITECTURE.md §2.8)
// ============================================================================

export interface FileFilter {
  scanId: ScanId;
  parentId?: NodeId; // list children of this directory
  extensionId?: ExtensionId;
  category?: FileCategory;
  minSizeBytes?: number;
  maxSizeBytes?: number;
  minMtime?: number;
  maxMtime?: number;
  includePatterns?: GlobPattern[];
  excludePatterns?: GlobPattern[];
  limit?: number;
  offset?: number;
  sortBy?: 'name' | 'size-logical' | 'size-allocated' | 'mtime' | 'extension';
  sortDir?: 'asc' | 'desc';
}

export interface FileRecordDto {
  nodeId: NodeId;
  parentId: NodeId;
  path: string; // reconstructed on demand by Rust
  name: string;
  isDirectory: boolean;
  logicalBytes: bigint;
  allocatedBytes: bigint;
  mtime: number;
  ctime: number;
  attrs: number;
  reparseKind: ReparseKind;
  hardlinkCount: number;
  sparse: boolean;
  extensionId: ExtensionId | null;
  hashSha256: string | null;
}

export interface ScanSummary {
  scanId: ScanId;
  rootPath: string;
  startedAt: number;
  completedAt: number;
  durationMs: number;
  fileCount: number;
  dirCount: number;
  totalLogical: bigint;
  totalAllocated: bigint;
  status: 'running' | 'complete' | 'cancelled' | 'failed' | 'aborted';
}

export interface ScanHistoryEntry {
  scanId: ScanId;
  priorScanId: ScanId | null;
  rootPath: string;
  startedAt: number;
  completedAt: number;
  durationMs: number;
  fileCount: number;
  totalLogical: bigint;
  totalAllocated: bigint;
  diff: {
    added: number;
    removed: number;
    modified: number;
    deltaBytes: bigint;
  } | null;
}

// ============================================================================
// Cleanup staging — per 10-UI-COMPONENTS-AND-VIEWS.md §10.1.4 CleanupTray
// ============================================================================

export interface CleanupItem {
  id: string; // UUID
  addedAt: number;
  path: string;
  scanId: ScanId | null;
  nodeId: NodeId | null;
  logicalBytes: bigint;
  allocatedBytes: bigint;
  isDirectory: boolean;
  source: 'manual' | 'quickwins' | 'duplicates';
}

export interface CleanupCommitResult {
  moved: CleanupItem[];
  failedInUse: CleanupItem[];
  failedOther: { item: CleanupItem; error: string }[];
  totalFreedBytes: bigint;
}

// ============================================================================
// App shell — per 07-ELECTRON-APP-SHELL.md §7.3
// ============================================================================

export interface UpdateInfo {
  version: string;
  channel: 'stable' | 'beta' | 'nightly';
  releaseNotesSummary: string;
  releaseNotesUrl: string;
  packageSizeBytes: number;
  criticalSecurityUpdate: boolean;
}

export interface MoveToTrashResult {
  moved: string[];
  failed: { path: string; error: string }[];
}

// ============================================================================
// The DiskAnalyzer API — what the renderer sees via `window.diskanalyzer`
// ============================================================================
// This interface is implemented by the preload's `contextBridge.exposeInMainWorld`.
// The renderer imports this type and uses `window.diskanalyzer` directly.
// Per 07-ELECTRON-APP-SHELL.md §7.3, every method has a typed request and response.

export interface DiskAnalyzerAPI {
  // ── Scanning ─────────────────────────────────────────────
  scan: {
    start(req: ScanStartRequest): Promise<ScanStartResponse>;
    cancel(scanId: ScanId): Promise<boolean>;
    status(scanId: ScanId): Promise<ScanStatus | null>;
    onProgress(cb: (e: ScanProgressEvent) => void): () => void;
    onBatch(cb: (e: ScanBatchEvent) => void): () => void;
    onComplete(cb: (e: ScanCompleteEvent) => void): () => void;
    onError(cb: (e: ScanErrorEvent) => void): () => void;
  };

  // ── License ──────────────────────────────────────────────
  license: {
    activate(key: string): Promise<ActivationResult>;
    deactivate(): Promise<void>;
    status(): Promise<LicenseStatus>;
    onChange(cb: (s: LicenseStatus) => void): () => void;
  };

  // ── Database queries (read-only; renderer treats DB as untrusted per §2.8) ─
  db: {
    queryFiles(filter: FileFilter): Promise<FileRecordDto[]>;
    queryHistory(scanId: ScanId): Promise<ScanHistoryEntry | null>;
    listScans(limit: number): Promise<ScanSummary[]>;
  };

  // ── Drives / volumes ─────────────────────────────────────
  volumes: {
    list(): Promise<VolumeInfo[]>;
  };

  // ── App shell ────────────────────────────────────────────
  app: {
    openExternal(url: string): Promise<void>;
    showItemInFolder(path: string): Promise<void>;
    moveToTrash(paths: string[]): Promise<MoveToTrashResult>;
    checkForUpdates(): Promise<UpdateInfo | null>;
    quitAndInstall(): Promise<void>;
    getVersion(): string;
    getPlatform(): 'win32' | 'darwin' | 'linux';
  };

  // ── Cleanup staging ──────────────────────────────────────
  cleanup: {
    add(items: CleanupItem[]): Promise<void>;
    remove(ids: string[]): Promise<void>;
    list(): Promise<CleanupItem[]>;
    commit(): Promise<CleanupCommitResult>;
    onChange(cb: (items: CleanupItem[]) => void): () => void;
  };
}

// ============================================================================
// Preload bridge type — what `contextBridge.exposeInMainWorld('diskanalyzer', ...)`
// exposes to the renderer.
//
// The renderer declares:
//   declare global {
//     interface Window { diskanalyzer: DiskAnalyzerAPI }
//   }
// ============================================================================

// (DiskAnalyzerAPI above IS the bridge type; the preload just exposes an object
// that matches this interface.)
