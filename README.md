# Spatium — A modern, fast Windows disk analyzer

[![CI](https://github.com/tensorkernel/spatium/actions/workflows/ci.yml/badge.svg)](https://github.com/tensorkernel/spatium/actions/workflows/ci.yml)
[![License: Commercial](https://img.shields.io/badge/License-Commercial-blue.svg)](LICENSE)
[![Rust 1.78+](https://img.shields.io/badge/Rust-1.78%2B-orange.svg)](https://www.rust-lang.org/)
[![Electron 30+](https://img.shields.io/badge/Electron-30%2B-47848F.svg)](https://www.electronjs.org/)
[![React 18+](https://img.shields.io/badge/React-18%2B-61DAFB.svg)](https://react.dev/)
[![TypeScript 5+](https://img.shields.io/badge/TypeScript-5%2B-3178C6.svg)](https://www.typescriptlang.org/)

A modern, fast, paid Windows disk analyzer with yearly + lifetime licensing. Built with **Electron + React + TypeScript + Vite** for the UI, and a **Rust** core engine (windows-rs, rayon, tokio, rusqlite, ed25519-dalek) for scanning and analysis.

Internal codename: `disk-analyzer` (used in source paths + configs).
Public brand name: **Spatium** (per `docs/BRAND-NAMES.md`, approved by the user).

## Status

- ✅ Phase 0 (Foundation & Brand) — complete
- ✅ Phase 1 (Rust Core MVP) — complete
- ✅ Phase 2 (Parallel + NTFS + Persistence) — complete
- ✅ Phase 3 (Electron Shell + Three-Column Renderer) — complete
- ✅ Phase 4 (Visualizations + Licensing Server) — complete
- ✅ Phase 5 (Beta & Hardening: installer file, updater, telemetry) — substantially complete
- 🚧 Phase 6 (Public 1.0 Launch) — marketing site + Stripe webhook stub in place; signing deferred
- 🚧 Phase 7 (Post-Launch: MSIX, localization hooks, arm64) — hooks in place

## Quick Start

```bash
# Install toolchain (Node 20+, pnpm 9+, Rust 1.78+)
corepack enable && corepack prepare pnpm@9 --activate
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal

# Clone + install
git clone https://github.com/tensorkernel/spatium.git
cd spatium
pnpm install --frozen-lockfile

# Build the Rust native module (debug)
pnpm build:native:debug

# Start dev mode (licensing server on :8080 + Electron app on :5173)
pnpm dev
```

In dev mode:
- The licensing server starts at `http://127.0.0.1:8080` with the demo key `DAPR-DEMO-0000-0000` pre-seeded (10 Pro features, 1-year expiry).
- The Electron app loads the renderer from `http://localhost:5173`.
- The dev Ed25519 keypair is auto-generated on first run by `loadOrGenerateDevKeys()` in `apps/licensing-server/src/keys.ts` (NEVER committed).

## Stack

```
Electron 30+
├── React 18+ (concurrent)
├── TypeScript 5+ (strict)
├── Vite 5+ (via electron-vite)
├── shadcn/ui foundation + Radix UI primitives
├── Lucide icons
├── Tailwind CSS 3.4+
├── TanStack Table + Virtual + Query
├── Recharts + visx
└── Rust core engine (via napi-rs)
    ├── Scanner (rayon parallel + windows-rs)
    ├── Aggregator (arena tree + string pool)
    ├── NTFS module (allocated size, hard links, reparse, sparse, USN)
    ├── SQLite (rusqlite, WAL mode)
    ├── Crypto (ed25519-dalek, ring, sha2)
    └── Helper (separate signed EXE for elevated paths)
```

## Architecture

The complete architecture lives in `docs/architecture/` (mirrored from `/home/z/my-project/download/disk-analyzer-architecture/`).

Key docs:
- `00-MASTER-PROMPT.md` — single source of truth.
- `02-SYSTEM-ARCHITECTURE.md` — three-layer separation.
- `04-PHASES-OVERVIEW.md` — phase plan + Definition of Done.
- `15-SECURITY-ARCHITECTURE.md` — 10-layer threat model.

## Repo Layout

```
spatium/
├── .github/workflows/ci.yml     # CI: Windows + Ubuntu, lint + test + build
├── apps/
│   ├── desktop/                  # Electron + React + Vite renderer
│   ├── licensing-server/         # Fastify + Ed25519 + SQLite licensing backend
│   ├── stripe-webhook/           # Stripe webhook handler (stub; Phase 6)
│   ├── marketing-site/           # Product landing page (HTML)
│   └── msix/                     # MSIX manifest stub (Phase 7)
├── crates/
│   ├── core/                     # Rust engine (scan, aggregate, ntfs, db, dedup)
│   ├── napi/                     # napi-rs binding exposed to Node
│   └── helper/                   # privileged helper (Phase 5)
├── packages/
│   ├── ui/                       # React component library + design tokens
│   ├── ipc/                      # shared IPC contract (TS) — the source of truth
│   └── license-client/           # TS LicenseClient + Updater + Telemetry
├── scripts/
│   ├── installer/Product.wxs     # WiX Toolset 4 installer definition
│   ├── string-scan.js            # P3 no-trace enforcement rule
│   ├── dep-cycle-check.js       # §2.10 forbidden-direction rule
│   └── test-licensing-direct.mjs # end-to-end licensing flow test
├── docs/architecture/           # cross-linked markdown knowledge base
├── Cargo.toml                   # Rust workspace root
├── package.json                  # pnpm workspace root
├── pnpm-workspace.yaml
├── rust-toolchain.toml
├── CHANGELOG.md                  # Keep a Changelog format
├── SECURITY.md                   # vulnerability reporting policy
├── LICENSE                       # closed-source commercial
├── CONTRIBUTING.md
└── README.md                     # this file
```

## Hard Principles (per `00-MASTER-PROMPT.md` §0.5)

- **P1** User-first, not AI-slop.
- **P2** Information density over decoration.
- **P3** No-trace of inspiration sources (WinDirStat, DiskBuddy) — enforced by `scripts/string-scan.js` in CI.
- **P4** Three-layer separation (UI / service / Windows) — enforced by `scripts/dep-cycle-check.js` in CI.
- **P5** Server-side authority where it matters (Ed25519-signed entitlements).
- **P6** Phases, not timelines.
- **P7** Persist everything, every session.
- **P8** Multi-POV review at phase gates.
- **P9** One process, one responsibility.
- **P10** No custom cryptography (use ed25519-dalek, ring, sha2).

## Tests

- 36 Rust tests pass (5 string_pool unit + 10 proptest + 3 e2e scanner + 3 db migrate + 2 db write + 2 db history + 5 partial hash + 3 full hash + 3 dedup pipeline).
- 6 TypeScript workspaces typecheck clean.
- `pnpm string-scan` PASS (no forbidden references per P3).
- `pnpm depcheck:cycle` PASS (no forbidden-direction imports per §2.10).
- End-to-end licensing flow verified: curl activate → 200 OK with Ed25519-signed entitlement → client verifies → 10 Pro features unlocked.

## License

Closed-source commercial. See [LICENSE](LICENSE) for the full terms.

Third-party licenses (Inter, JetBrains Mono, Lucide, etc.) are documented in `LICENSE` and verified compatible with closed-source commercial distribution.

## Contact

- **Source**: https://github.com/tensorkernel/spatium
- **Support**: 4011umar@gmail.com
- **Marketing**: `apps/marketing-site/index.html`
- **Bug reports**: https://github.com/tensorkernel/spatium/issues
- **Security**: see [SECURITY.md](SECURITY.md) — DO NOT open public issues for security vulnerabilities.
