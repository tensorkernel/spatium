# Contributing to Spatium

First off: thank you for considering contributing! Even though Spatium is closed-source commercial software, we accept bug reports, feature requests, and (for selected partners) external PRs.

## Code of Conduct

- Be respectful. We're all here to make a great disk analyzer.
- Don't mention inspiration sources (per `00-MASTER-PROMPT.md` §0.5 P3) in any user-visible artifact.
- Don't commit secrets. Per `15-SECURITY-ARCHITECTURE.md` §15.10, no private key, API secret, or production credential lives in the repo. The dev licensing keys are auto-generated on first run by `loadOrGenerateDevKeys()` (per `13-LICENSING-SERVER.md` §13.5) and are gitignored.

## Getting Started

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

## Architecture

Read the architecture doc set in `docs/architecture/` before touching code. Key files:

- `00-MASTER-PROMPT.md` — single source of truth, principles, scope.
- `02-SYSTEM-ARCHITECTURE.md` — three-layer separation (UI / service / Windows).
- `04-PHASES-OVERVIEW.md` — phase plan + Definition of Done.
- `19-AGENT-WORKFLOW-AND-SESSIONS.md` — session protocol + worklog.
- `20-QUALITY-GATES-AND-POVS.md` — POV review checklists.

## Session Protocol (for AI agents)

Per `19-AGENT-WORKFLOW-AND-SESSIONS.md`, every session starts by:

1. Read `00-MASTER-PROMPT.md`.
2. Read `23-WORKLOG.md` to see prior sessions' work.
3. Read `04-PHASES-OVERVIEW.md` for the active phase.
4. Read the specific docs for the work.
5. Acknowledge the session start in the worklog.

## Code Quality

CI runs on every push and PR:

- `pnpm lint` — Biome format + lint.
- `pnpm typecheck` — strict TS in 6 workspaces.
- `cargo fmt --check` + `cargo clippy -- -D warnings` — Rust lint.
- `cargo test --workspace --features proptest` — 36 tests.
- `pnpm string-scan` — P3 no-trace rule.
- `pnpm depcheck:cycle` — §2.10 forbidden-direction rule.

PRs that fail any check are not merged. The five senior POV roles (per `00-MASTER-PROMPT.md` §0.9) review at phase gates.

## License

By contributing, you agree that your contributions will be licensed under the same closed-source commercial terms as the rest of Spatium (see `LICENSE`).
