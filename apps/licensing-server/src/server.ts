// DiskAnalyzer licensing server.
// Per 13-LICENSING-SERVER.md.
//
// Runs on http://localhost:8080 in dev (per 00-MASTER-PROMPT.md §0.3 and
// user direction). Demo key: DAPR-DEMO-0000-0000 (per §13.5).

import { type KeyObject, sign as ed25519Sign, randomBytes } from 'node:crypto';
import { existsSync, mkdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import cors from '@fastify/cors';
import Database from 'better-sqlite3';
import Fastify from 'fastify';

import { type EntitlementPayload, type WireEntitlement, canonicalize } from './entitlement.js';
import { loadOrGenerateDevKeys } from './keys.js';
import type { ActivateRequest, DeviceRecord, LicenseRecord, RevalidateRequest } from './types.js';

const PORT = Number(process.env.LICENSING_PORT ?? 8080);
const HOST = process.env.LICENSING_HOST ?? '127.0.0.1';

// The demo key (per §13.5). Always available in dev.
const DEMO_KEY = 'DAPR-DEMO-0000-0000';

// Database path — under user's local app data.
function dbPath(): string {
  const home = homedir();
  const dir = join(home, '.spatium', 'licensing-server');
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
  return join(dir, 'dev.sqlite3');
}

// ─────────────────────────────────────────────────────────────────────
// Database
// ─────────────────────────────────────────────────────────────────────

const db = new Database(dbPath());
db.pragma('journal_mode = WAL');
db.pragma('foreign_keys = ON');
db.exec(`
  CREATE TABLE IF NOT EXISTS licenses (
    key TEXT PRIMARY KEY,
    tier TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    customer_email TEXT NOT NULL,
    issued_at INTEGER NOT NULL,
    expires_at INTEGER,
    max_devices INTEGER NOT NULL DEFAULT 3,
    status TEXT NOT NULL DEFAULT 'active',
    features_json TEXT NOT NULL DEFAULT '[]',
    channel TEXT NOT NULL DEFAULT 'stable',
    min_app_version TEXT NOT NULL DEFAULT '0.1.0',
    max_app_version TEXT
  );
  CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY,
    license_key TEXT NOT NULL REFERENCES licenses(key),
    fingerprint TEXT NOT NULL,
    name TEXT NOT NULL,
    platform TEXT NOT NULL,
    arch TEXT NOT NULL,
    first_seen_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    UNIQUE(license_key, fingerprint)
  );
  CREATE TABLE IF NOT EXISTS activations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_key TEXT NOT NULL,
    device_id TEXT,
    ip_hash TEXT,
    success INTEGER NOT NULL,
    error_code TEXT,
    at INTEGER NOT NULL
  );
  CREATE INDEX IF NOT EXISTS idx_activations_key ON activations(license_key, at);
`);

// Insert the demo key if it's not present.
const demoKeyRow = db.prepare('SELECT key FROM licenses WHERE key = ?').get(DEMO_KEY);
if (!demoKeyRow) {
  const now = Math.floor(Date.now() / 1000);
  db.prepare(`
    INSERT INTO licenses (key, tier, customer_id, customer_email, issued_at, expires_at, max_devices, status, features_json, channel, min_app_version, max_app_version)
    VALUES (?, 'pro_yearly', 'demo-customer', 'demo@spatium.local', ?, ?, 3, 'active', ?, 'stable', '0.1.0', NULL)
  `).run(
    DEMO_KEY,
    now,
    now + 365 * 86400,
    JSON.stringify([
      'scan.unlimited',
      'duplicate.analysis',
      'historical.scans',
      'age.map',
      'cleanup.unlimited',
      'quickwins.full',
      'scheduled.scans',
      'export.advanced',
      'ntfs.advanced',
      'cli',
    ]),
  );
  console.log(`[licensing-server] Inserted demo key: ${DEMO_KEY}`);
}

// ─────────────────────────────────────────────────────────────────────
// Key pair
// ─────────────────────────────────────────────────────────────────────

const devKeys = loadOrGenerateDevKeys();
console.log(
  `[licensing-server] Dev public key (DER, ${devKeys.publicKeyDer.length} bytes) loaded.`,
);
console.log(
  `[licensing-server] Save the corresponding public key to packages/license-client/src/keys/dev.public.der`,
);
console.log(`[licensing-server] (See scripts/copy-dev-pubkey.js for a one-shot copy.)`);

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

function signEntitlement(payload: EntitlementPayload): WireEntitlement {
  const canonical = canonicalize(payload);
  // Per Node crypto docs: Ed25519 doesn't use a hash; pass `null` as the algorithm.
  // The `createSign('Ed25519')` API fails with "Invalid digest"; use `crypto.sign()` directly.
  const signature = ed25519Sign(
    null,
    Buffer.from(canonical, 'utf8'),
    devKeys.privateKey as KeyObject,
  );
  return {
    entitlement_b64: Buffer.from(canonical, 'utf8').toString('base64'),
    signature_b64: signature.toString('base64'),
    alg: 'Ed25519',
    ver: 1,
  };
}

function buildEntitlement(license: LicenseRecord, device: DeviceRecord): EntitlementPayload {
  const now = Math.floor(Date.now() / 1000);
  const graceUntil = license.expires_at ? license.expires_at + 30 * 86400 : now + 365 * 86400;
  return {
    v: 1,
    license_id: license.key,
    product_id: 'disk-analyzer',
    tier: license.tier,
    customer_id: license.customer_id,
    customer_email: license.customer_email,
    features: license.features,
    issued_at: now,
    expires_at: license.expires_at,
    grace_until: graceUntil,
    device_fingerprint: device.fingerprint,
    device_id: device.id,
    max_devices: license.max_devices,
    channel: license.channel,
    min_app_version: license.min_app_version,
    max_app_version: license.max_app_version,
    nonce: randomBytes(16).toString('hex'),
  };
}

function loadLicense(key: string): LicenseRecord | null {
  const row = db.prepare('SELECT * FROM licenses WHERE key = ?').get(key) as
    | Record<string, unknown>
    | undefined;
  if (!row) return null;
  const devices = (
    db.prepare('SELECT * FROM devices WHERE license_key = ?').all(key) as Array<
      Record<string, unknown>
    >
  ).map((r) => ({
    id: r.id as string,
    fingerprint: r.fingerprint as string,
    name: r.name as string,
    platform: r.platform as string,
    arch: r.arch as string,
    first_seen_at: r.first_seen_at as number,
    last_seen_at: r.last_seen_at as number,
  }));
  return {
    key: row.key as string,
    tier: row.tier as 'pro_yearly' | 'pro_lifetime',
    customer_id: row.customer_id as string,
    customer_email: row.customer_email as string,
    issued_at: row.issued_at as number,
    expires_at: row.expires_at as number | null,
    max_devices: row.max_devices as number,
    status: row.status as LicenseRecord['status'],
    features: JSON.parse(row.features_json as string) as string[],
    channel: row.channel as 'stable' | 'beta' | 'nightly',
    min_app_version: row.min_app_version as string,
    max_app_version: (row.max_app_version as string | null) ?? null,
    devices,
  };
}

function upsertDevice(licenseKey: string, req: ActivateRequest): DeviceRecord {
  const now = Math.floor(Date.now() / 1000);
  const deviceId = `${req.device_fingerprint.slice(0, 8)}-${req.device_name ?? 'device'}`;
  const existing = db
    .prepare('SELECT * FROM devices WHERE license_key = ? AND fingerprint = ?')
    .get(licenseKey, req.device_fingerprint) as Record<string, unknown> | undefined;
  if (existing) {
    db.prepare('UPDATE devices SET last_seen_at = ? WHERE id = ?').run(now, existing.id as string);
    return {
      id: existing.id as string,
      fingerprint: existing.fingerprint as string,
      name: existing.name as string,
      platform: existing.platform as string,
      arch: existing.arch as string,
      first_seen_at: existing.first_seen_at as number,
      last_seen_at: now,
    };
  }
  db.prepare(`
    INSERT INTO devices (id, license_key, fingerprint, name, platform, arch, first_seen_at, last_seen_at)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
  `).run(
    deviceId,
    licenseKey,
    req.device_fingerprint,
    req.device_name ?? 'device',
    req.platform,
    req.arch,
    now,
    now,
  );
  return {
    id: deviceId,
    fingerprint: req.device_fingerprint,
    name: req.device_name ?? 'device',
    platform: req.platform,
    arch: req.arch,
    first_seen_at: now,
    last_seen_at: now,
  };
}

// ─────────────────────────────────────────────────────────────────────
// Server
// ─────────────────────────────────────────────────────────────────────

const server = Fastify({ logger: { level: 'info' } });

await server.register(cors, { origin: true });

server.get('/health', async () => ({ status: 'ok', version: '0.1.0', time: Date.now() }));

server.post<{ Body: ActivateRequest }>('/v1/licenses/activate', async (req, reply) => {
  const body = req.body;
  if (!body?.key || !body?.device_fingerprint) {
    return reply.code(400).send({ error: 'INVALID_KEY_FORMAT' });
  }
  const license = loadLicense(body.key);
  if (!license) {
    db.prepare(
      'INSERT INTO activations (license_key, ip_hash, success, error_code, at) VALUES (?, ?, 0, ?, ?)',
    ).run(body.key, hashIp(req.ip), 'KEY_NOT_FOUND', Date.now());
    return reply.code(404).send({ error: 'KEY_NOT_FOUND' });
  }
  if (license.status === 'revoked') {
    return reply.code(410).send({ error: 'KEY_REVOKED' });
  }
  if (license.expires_at !== null && license.expires_at < Math.floor(Date.now() / 1000)) {
    return reply.code(410).send({ error: 'KEY_EXPIRED' });
  }
  if (license.devices.length >= license.max_devices) {
    const alreadyBound = license.devices.some((d) => d.fingerprint === body.device_fingerprint);
    if (!alreadyBound) {
      return reply.code(409).send({ error: 'DEVICE_LIMIT_REACHED' });
    }
  }

  const device = upsertDevice(license.key, body);
  const entitlement = buildEntitlement(license, device);
  const wire = signEntitlement(entitlement);

  db.prepare(
    'INSERT INTO activations (license_key, device_id, ip_hash, success, at) VALUES (?, ?, ?, 1, ?)',
  ).run(license.key, device.id, hashIp(req.ip), Date.now());

  return reply.send(wire);
});

server.post<{ Body: RevalidateRequest; Headers: { authorization?: string } }>(
  '/v1/licenses/revalidate',
  async (req, reply) => {
    const auth = req.headers.authorization;
    if (!auth?.startsWith('Bearer ')) {
      return reply.code(401).send({ error: 'UNAUTHORIZED' });
    }
    const licenseKey = auth.slice('Bearer '.length);
    const license = loadLicense(licenseKey);
    if (!license) {
      return reply.code(404).send({ error: 'KEY_NOT_FOUND' });
    }
    if (license.status === 'revoked') {
      return reply.code(410).send({ error: 'ENTITLEMENT_REVOKED' });
    }
    // Issue a fresh entitlement. (Device is already registered; we just re-sign.)
    const device = license.devices[0];
    if (!device) {
      return reply.code(409).send({ error: 'DEVICE_NOT_REGISTERED' });
    }
    const entitlement = buildEntitlement(license, device);
    const wire = signEntitlement(entitlement);
    return reply.send(wire);
  },
);

server.post<{ Headers: { authorization?: string } }>(
  '/v1/licenses/deactivate',
  async (req, reply) => {
    const auth = req.headers.authorization;
    if (!auth?.startsWith('Bearer ')) {
      return reply.code(401).send({ error: 'UNAUTHORIZED' });
    }
    const licenseKey = auth.slice('Bearer '.length);
    // For simplicity, deactivate the first device registered to this license.
    // (Real impl would take a device_id in the body.)
    db.prepare(
      'DELETE FROM devices WHERE license_key = ? AND id = (SELECT id FROM devices WHERE license_key = ? LIMIT 1)',
    ).run(licenseKey, licenseKey);
    return reply.code(204).send();
  },
);

// ─────────────────────────────────────────────────────────────────────
// Admin endpoints (per §13.4.5)
// ─────────────────────────────────────────────────────────────────────

const ADMIN_TOKEN = process.env.LICENSING_ADMIN_TOKEN ?? 'dev-admin-token';

server.get<{ Params: { key: string } }>('/v1/licenses/status/:key', async (req, reply) => {
  const auth = req.headers.authorization;
  if (auth !== `Bearer ${ADMIN_TOKEN}`) {
    return reply.code(401).send({ error: 'UNAUTHORIZED' });
  }
  const license = loadLicense(req.params.key);
  if (!license) return reply.code(404).send({ error: 'KEY_NOT_FOUND' });
  return reply.send(license);
});

server.post<{ Body: { license_id: string; reason: string }; Headers: { authorization?: string } }>(
  '/v1/licenses/revoke',
  async (req, reply) => {
    const auth = req.headers.authorization;
    if (auth !== `Bearer ${ADMIN_TOKEN}`) {
      return reply.code(401).send({ error: 'UNAUTHORIZED' });
    }
    db.prepare('UPDATE licenses SET status = ? WHERE key = ?').run('revoked', req.body.license_id);
    return reply.code(204).send();
  },
);

function hashIp(ip: string | undefined): string {
  if (!ip) return 'unknown';
  // Simple hash for storage. (Not for security; just to avoid storing raw IPs.)
  let h = 0;
  for (let i = 0; i < ip.length; i += 1) {
    h = (h * 31 + ip.charCodeAt(i)) >>> 0;
  }
  return h.toString(16);
}

// ─────────────────────────────────────────────────────────────────────
// Start
// ─────────────────────────────────────────────────────────────────────

try {
  await server.listen({ port: PORT, host: HOST });
  console.log(`[licensing-server] Listening on http://${HOST}:${PORT}`);
  console.log(`[licensing-server] Demo key: ${DEMO_KEY}`);
  console.log(`[licensing-server] Admin token: ${ADMIN_TOKEN}`);
  console.log(`[licensing-server] Health: http://${HOST}:${PORT}/health`);
} catch (err) {
  console.error(err);
  process.exit(1);
}
