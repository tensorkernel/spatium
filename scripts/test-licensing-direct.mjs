// Simple licensing flow test — assumes the server is already running on :8080.
// Run with: `tsx scripts/test-licensing-direct.mjs` after starting `apps/licensing-server`.

import { existsSync, readFileSync } from 'node:fs';
import { computeDeviceFingerprint, verifyEntitlement } from '../packages/license-client/src/index.ts';

const DEV_PUB_KEY_PATH = 'apps/licensing-server/src/keys/dev.public.der';
const DEMO_KEY = 'DAPR-DEMO-0000-0000';

async function main() {
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
}

main().catch((err) => {
  console.error('[test] FAIL:', err.message);
  process.exit(1);
});
