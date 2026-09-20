// Test script for the licensing flow.
// Starts the licensing server, activates with the demo key, verifies the entitlement.

import { spawn } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { computeDeviceFingerprint, verifyEntitlement } from '@disk-analyzer/license-client';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DEV_PUB_KEY_PATH = join(__dirname, '..', 'apps', 'licensing-server', 'src', 'keys', 'dev.public.der');

const DEMO_KEY = 'DAPR-DEMO-0000-0000';
const FINGERPRINT = computeDeviceFingerprint('spatium-dev-install-salt');

async function waitForServer(url, timeoutMs = 10000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const r = await fetch(url);
      if (r.ok) return;
    } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
  throw new Error(`Server at ${url} did not respond within ${timeoutMs}ms`);
}

async function main() {
  console.log('[test] Starting licensing server...');
  const server = spawn('pnpm', ['--filter', '@disk-analyzer/licensing-server', 'run', 'dev'], {
    cwd: join(__dirname, '..'),
    stdio: ['ignore', 'pipe', 'pipe'],
    env: { ...process.env, LICENSING_PORT: '8080' },
  });

  server.stdout.on('data', (d) => process.stdout.write(`[server] ${d}`));
  server.stderr.on('data', (d) => process.stderr.write(`[server] ${d}`));

  try {
    await waitForServer('http://127.0.0.1:8080/health');
    console.log('[test] Server is up.');

    // Activate with the demo key.
    console.log('[test] Activating demo key:', DEMO_KEY);
    const fingerprint = computeDeviceFingerprint('spatium-dev-install-salt');
    const res = await fetch('http://127.0.0.1:8080/v1/licenses/activate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        key: DEMO_KEY,
        device_fingerprint: fingerprint,
        device_name: 'test-device',
        app_version: '0.2.0',
        platform: process.platform,
        arch: process.arch,
      }),
    });
    if (!res.ok) {
      throw new Error(`activation failed: HTTP ${res.status} ${await res.text()}`);
    }
    const wire = await res.json();
    console.log('[test] Got wire entitlement; alg:', wire.alg, 'ver:', wire.ver);

    // Load the dev public key and verify.
    if (!existsSync(DEV_PUB_KEY_PATH)) {
      throw new Error(`Dev public key not found at ${DEV_PUB_KEY_PATH}`);
    }
    const publicKeyDer = readFileSync(DEV_PUB_KEY_PATH);
    console.log('[test] Loaded public key (%d bytes).', publicKeyDer.length);

    const verified = verifyEntitlement(
      wire,
      fingerprint,
      'disk-analyzer',
      '0.2.0',
      'stable',
      publicKeyDer,
      Math.floor(Date.now() / 1000),
    );
    if (typeof verified === 'string') {
      throw new Error(`signature verification failed: ${verified}`);
    }
    console.log('[test] Entitlement verified!');
    console.log('[test]   license_id:', verified.license_id);
    console.log('[test]   tier:', verified.tier);
    console.log('[test]   features:', verified.features.length, 'features');
    console.log('[test]   expires_at:', verified.expires_at, '(', new Date(verified.expires_at * 1000).toISOString(), ')');
    console.log('[test]   device_id:', verified.device_id);

    console.log('[test] PASS: full activate → sign → verify flow works.');
  } finally {
    server.kill('SIGTERM');
    // Give it a moment to clean up.
    await new Promise((r) => setTimeout(r, 500));
  }
}

main().catch((err) => {
  console.error('[test] FAIL:', err);
  process.exit(1);
});
