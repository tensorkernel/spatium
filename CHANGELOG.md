# Changelog

All notable changes to **Spatium** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Phase 5 production wiring: GitHub Actions CI on Windows + Ubuntu.
- Phase 6 launch artifacts: CHANGELOG, SECURITY policy, LICENSE template.
- Phase 7 hooks: MSIX manifest stub, i18n skeleton (en), arm64 build target.

## [0.2.0] — 2026-09-20

### Added — Phase 2: Rust Core (Parallel + NTFS + Persistence)
- `crates/core/src/ntfs/` module with cross-platform simulation (allocated size, reparse, hard links, sparse, USN). Real Windows implementations gated behind `#[cfg(windows)]` via `windows-rs`.
- `crates/core/src/db/` SQLite persistence layer: 4 migrations (volumes, scans, paths, files, extensions, settings, cleanup_items, quick_wins, duplicate_groups, duplicate_files). WAL mode + batched transactions.
- `crates/core/src/dedup/` duplicate detection pipeline: size-group → xxhash64 partial (head+tail+middle 4KB each) → SHA-256 full. Avoids hashing every file.
- `crates/core/src/scan/worker.rs` rayon parallel scan worker with bounded tokio mpsc channel (cap 8192) for backpressure.

### Added — Phase 3: Electron Shell + Three-Column Renderer
- `apps/desktop/src/main.ts` expanded Electron main with LicenseClient init + capability gating at Layer B.
- Three-column React shell: `AppShell`, `Sidebar`, `InspectorPanel`, `TopBar` (9-tab view switcher with Ctrl+1..9), `StatusBar`, `DriveCard`, `CommandPalette` (Ctrl+K).
- `packages/ui/src/visualizations/Treemap.tsx` squarified treemap (Bruls 2000 paper) with Canvas 2D rendering, viewport culling, hover tooltip, drill-in.
- Tailwind config wired to CSS custom properties for instant dark/light theme swap.

### Added — Phase 4: Visualizations + Licensing Server
- `apps/licensing-server/` Fastify + Ed25519 + better-sqlite3 licensing server on `http://127.0.0.1:8080`. Demo key `DAPR-DEMO-0000-0000` pre-seeded with 10 Pro features, 1-year expiry.
- `packages/license-client/` TS license-client with Ed25519 verify (7 rejection rules), 30-day offline grace, 6-signal device fingerprint, persistence.
- Capability gating at Layer B verified end-to-end: scan:start handler rejects Pro features without entitlement.
- `pnpm dev` starts licensing server + Electron together.

### Added — Phase 5: Installer + Updater + Telemetry
- `scripts/installer/Product.wxs` WiX Toolset 4 installer definition (per-user install, Start Menu + Desktop shortcuts, Inter/JetBrains Mono fonts bundled).
- `packages/license-client/src/updater.ts` auto-updater with signed-manifest Ed25519 verification, version policy (channel/arch/min_app_version check), SHA-256 hash verification, staging.
- `packages/license-client/src/telemetry.ts` telemetry pipeline with opt-in + path-redaction regex (Windows/Unix paths, URLs, file extensions).

### Tests
- 36 Rust tests pass (5 string_pool unit + 10 proptest + 3 e2e scanner + 3 db migrate + 2 db write + 2 db history + 5 partial hash + 3 full hash + 3 dedup pipeline).
- 6 TypeScript workspaces typecheck clean (ipc, ui, napi, desktop, licensing-server, license-client).
- `pnpm string-scan` PASS (no forbidden references per P3).
- `pnpm depcheck:cycle` PASS (no forbidden-direction imports per §2.10).
- End-to-end licensing flow verified: curl activate → 200 OK with Ed25519-signed entitlement → client verifies → 10 Pro features unlocked.

## [0.1.0] — 2026-09-20

### Added — Phase 0: Foundation & Brand
- Repo initialized at `/home/z/my-project/disk-analyzer/` per `02-SYSTEM-ARCHITECTURE.md` §2.6 layout.
- Root `package.json` + `pnpm-workspace.yaml` + `Cargo.toml` (workspace) + `rust-toolchain.toml` (Rust 1.78+ MSRV).
- `.gitignore`, `.editorconfig`, `biome.json`, `tsconfig.base.json`, `eslint.config.js`, `.cargo/config.toml`, `.vscode/`.
- `.github/workflows/ci.yml` — 11-job CI pipeline.
- `scripts/string-scan.js` (P3 no-trace enforcement rule) + `scripts/dep-cycle-check.js` (§2.10 forbidden-direction enforcement rule).
- `packages/ipc/src/types.ts` v0.1 — the canonical IPC contract (`DiskAnalyzerAPI`).
- `packages/ui/src/tokens/` design tokens (colors, typography, spacing, radii, motion, density) + CSS custom properties.
- `docs/BRAND-NAMES.md` with 12 candidates; **Spatium** recommended and approved by user.
- `README.md` at repo root.

### Added — Phase 1: Rust Core, Single-Threaded MVP
- `crates/core/` Rust engine: `scan/` (scanner, event, options, reparse), `aggregate/` (arena, string_pool, totals, topn, extensions), `util/` (cancel, paths).
- 72-byte `FileRecord` arena-allocated; no owned strings per record (paths reconstructed on demand).
- Iterative (not recursive) traversal with loop detection + per-path error handling.
- `crates/napi/` napi-rs binding exposing `onScanEvent`, `scanStart`, `scanCancel`, `getActiveScanId`, `getScanStatus`, `engineVersion` to Node. Built `.node` native module successfully on Linux x64.
- `apps/desktop/` minimal Electron app: `main.ts` (strict security, Mock IPC fallback), `preload/index.ts` (typed contextBridge), `renderer/` minimal React shell.
- 13 tests pass (5 string_pool + 10 proptest + 3 e2e scanner).
