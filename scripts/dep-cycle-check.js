// Dependency-cycle linter per 02-SYSTEM-ARCHITECTURE.md §2.10 (forbidden directions).
//
// Forbidden:
//   - renderer -> apps/desktop/src (renderer must not import Electron main code)
//   - renderer -> any Rust crate
//   - crates/core -> apps/* (Rust must not depend on Node/Electron)
//   - packages/license-client -> crates/*
//   - crates/helper -> apps/desktop/src (helper is independent)
//
// Implementation (Phase 0): we parse each TS file's import statements and check
// the resolved path against the forbidden direction. This is a rough first cut;
// Phase 3 will switch to a typed import-only ESLint rule + tsconfig path-alias
// enforcement so that the IDE flags violations before CI does.

import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, relative, resolve, sep } from 'node:path';

const ROOT = process.cwd();
const SKIP_DIRS = new Set([
  'node_modules',
  'dist',
  'target',
  'out',
  'release',
  '.git',
  '.vite',
  '.cache',
  '.pnpm-store',
  'coverage',
  'docs',
]);

let violations = 0;

function* walk(dir) {
  for (const name of readdirSync(dir)) {
    if (SKIP_DIRS.has(name)) continue;
    const full = join(dir, name);
    const st = statSync(full);
    if (st.isDirectory()) yield* walk(full);
    else if (st.isFile()) yield full;
  }
}

function classifyPath(p) {
  const rel = relative(ROOT, p).replace(/\\/g, '/');
  if (rel.startsWith('apps/desktop/renderer/')) return 'renderer';
  if (rel.startsWith('apps/desktop/src/')) return 'electron-main';
  if (rel.startsWith('apps/desktop/preload/')) return 'preload';
  if (rel.startsWith('apps/licensing-server/')) return 'licensing-server';
  if (rel.startsWith('crates/')) return 'rust-crate';
  if (rel.startsWith('packages/ui/')) return 'ui-pkg';
  if (rel.startsWith('packages/ipc/')) return 'ipc-pkg';
  if (rel.startsWith('packages/license-client/')) return 'license-client-pkg';
  if (rel === 'scripts/' || rel.startsWith('scripts/')) return 'scripts';
  return 'other';
}

function checkImport(importer, importSpec) {
  const importerClass = classifyPath(importer);
  if (importerClass === 'other' || importerClass === 'scripts') return;

  // Map external-package imports to a class.
  let targetClass = null;
  if (importSpec.startsWith('@disk-analyzer/ui')) targetClass = 'ui-pkg';
  else if (importSpec.startsWith('@disk-analyzer/ipc')) targetClass = 'ipc-pkg';
  else if (importSpec.startsWith('@disk-analyzer/license-client'))
    targetClass = 'license-client-pkg';
  else if (importSpec.startsWith('@disk-analyzer/napi') || importSpec.includes('crates/napi'))
    targetClass = 'rust-crate';

  // Relative imports
  if (importSpec.startsWith('.')) {
    const resolved = resolve(dirname(importer), importSpec);
    targetClass = classifyPath(resolved);
  }

  if (!targetClass) return;

  // Check forbidden directions per 02-SYSTEM-ARCHITECTURE.md §2.10.
  const isForbidden =
    (importerClass === 'renderer' && targetClass === 'electron-main') ||
    (importerClass === 'renderer' && targetClass === 'rust-crate') ||
    (importerClass === 'renderer' && targetClass === 'preload') ||
    (importerClass === 'electron-main' && targetClass === 'renderer') ||
    (importerClass === 'rust-crate' && targetClass === 'electron-main') ||
    (importerClass === 'rust-crate' && targetClass === 'renderer') ||
    (importerClass === 'license-client-pkg' && targetClass === 'rust-crate') ||
    (importerClass === 'rust-crate' && targetClass === 'licensing-server');

  if (isForbidden) {
    violations += 1;
    console.error(
      `FAIL: ${relative(ROOT, importer).replace(/\\/g, '/')}: forbidden import "${importSpec}" (importer=${importerClass}, target=${targetClass})`,
    );
  }
}

const IMPORT_RE = /(?:import\s+[^'"]+?\s+from\s+|require\(\s*)['"]([^'"]+)['"]/g;

let scanned = 0;
for (const file of walk(ROOT)) {
  if (
    !file.endsWith('.ts') &&
    !file.endsWith('.tsx') &&
    !file.endsWith('.js') &&
    !file.endsWith('.jsx')
  )
    continue;
  const text = readFileSync(file, 'utf8');
  scanned += 1;
  for (const m of text.matchAll(IMPORT_RE)) {
    checkImport(file, m[1]);
  }
}

console.log(`dependency-cycle-lint: scanned ${scanned} TS/JS files.`);
if (violations > 0) {
  console.error(`${violations} forbidden-direction violation(s) found.`);
  console.error('See 02-SYSTEM-ARCHITECTURE.md §2.10 for the allowed dependency direction graph.');
  process.exit(1);
} else {
  console.log('dependency-cycle-lint: PASS (no forbidden-direction imports found).');
}
