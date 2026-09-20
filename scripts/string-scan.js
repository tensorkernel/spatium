// String-scan rule per §20.2.1 SD-11 and §00-MASTER-PROMPT.md P3 (no-trace rule).
// Rejects PRs that introduce user-visible references to inspiration sources.
//
// Forbidden in user-visible artifacts (code under apps/, packages/, crates/).
// Permitted in internal docs (docs/, *.md at repo root) for audit trail.

import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';

const ROOT = process.cwd();
const FORBIDDEN = [
  'windirstat',
  'windir',
  'kdirstat',
  'qdir',
  'treemap.cpp',
  'diskbuddy',
  // portmanteaus that obviously derive
  'diskbuddy_clone',
  'dirbuddy',
];
const FORBIDDEN_RE = new RegExp(FORBIDDEN.map((s) => s.replace(/\./g, '\\.')).join('|'), 'i');

// Folders we don't scan (no source-of-truth files there).
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
]);

// Folders where docs live (mentions are allowed for audit).
const DOCS_DIRS = new Set(['docs']);

const ALLOWED_SUFFIXES = new Set([
  '.ts',
  '.tsx',
  '.js',
  '.jsx',
  '.rs',
  '.json',
  '.yml',
  '.yaml',
  '.toml',
  '.jsonc',
  '.wxs',
  '.wxl',
]);

let violations = 0;

function* walk(dir) {
  for (const name of readdirSync(dir)) {
    if (SKIP_DIRS.has(name)) continue;
    const full = join(dir, name);
    const st = statSync(full);
    if (st.isDirectory()) {
      yield* walk(full);
    } else if (st.isFile()) {
      yield full;
    }
  }
}

function isUserVisible(filePath) {
  const rel = relative(ROOT, filePath);
  // Internal docs (any .md anywhere, plus /docs/ content) are exempt.
  if (rel.endsWith('.md')) return false;
  const parts = rel.split(sep);
  if (parts.some((p) => DOCS_DIRS.has(p))) return false;
  // The architecture doc set is mirrored under /docs/ — exempt.
  if (parts[0] === 'docs') return false;
  // The repo README at root is exempt (it links to the doc set).
  if (rel === 'README.md') return false;
  return true;
}

for (const file of walk(ROOT)) {
  if (!isUserVisible(file)) continue;
  const rel = relative(ROOT, file).replace(/\\/g, '/');
  // Skip self (this file obviously contains the forbidden words list).
  if (rel === 'scripts/string-scan.js') continue;
  const ext = file.slice(file.lastIndexOf('.'));
  if (!ext || !ALLOWED_SUFFIXES.has(ext.toLowerCase())) continue;

  const text = readFileSync(file, 'utf8');
  // Split into lines; report each offending line.
  const lines = text.split(/\r?\n/);
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const m = line.match(FORBIDDEN_RE);
    if (m) {
      // Filter false positives: a Cargo.lock / pnpm-lock may reference a crate name like `qdir-rs` if we ever adopted one. For now we don't; ignore lockfiles.
      if (
        rel.endsWith('Cargo.lock') ||
        rel.endsWith('pnpm-lock.yaml') ||
        rel.endsWith('package-lock.json')
      )
        continue;
      violations += 1;
      console.error(`FAIL: ${rel}:${i + 1}: forbidden reference "${m[0]}"`);
      console.error(`  > ${line.trim().slice(0, 200)}`);
    }
  }
}

if (violations > 0) {
  console.error(`\n${violations} forbidden-reference violation(s) found.`);
  console.error(
    'Per P3 (00-MASTER-PROMPT.md), no user-visible artifact may reference inspiration sources.',
  );
  console.error('See 22-GLOSSARY-AND-REFERENCES.md §22.10 for the forbidden list.');
  process.exit(1);
} else {
  console.log('string-scan: PASS (no forbidden references in user-visible artifacts).');
}
