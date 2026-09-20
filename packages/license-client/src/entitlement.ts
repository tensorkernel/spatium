// DiskAnalyzer — entitlement verification + device fingerprint.
// Per 14-LICENSING-CLIENT.md §14.3 and §14.4.

import { createHash, createPublicKey, verify } from 'node:crypto';
import { cpus, hostname, platform, release, totalmem, userInfo } from 'node:os';

export type LicenseTier = 'free' | 'pro_yearly' | 'pro_lifetime';

export interface Entitlement {
  v: 1;
  license_id: string;
  product_id: 'disk-analyzer';
  tier: Exclude<LicenseTier, 'free'>;
  customer_id: string;
  customer_email: string;
  features: string[];
  issued_at: number; // unix seconds
  expires_at: number | null;
  grace_until: number;
  device_fingerprint: string;
  device_id: string;
  max_devices: number;
  channel: 'stable' | 'beta' | 'nightly';
  min_app_version: string;
  max_app_version: string | null;
  nonce: string;
}

export interface WireEntitlement {
  entitlement_b64: string;
  signature_b64: string;
  alg: 'Ed25519';
  ver: 1;
}

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

/// Compute the device fingerprint from multiple OS signals.
///
/// Per 14-LICENSING-CLIENT.md §14.4. Six signals:
///   1. hostname
///   2. platform + release
///   3. user info (username + uid)
///   4. process.cwd() (proxy for "where the app lives")
///   5. a random per-installation salt (persisted to disk on first run)
///   6. CPU count + total memory
///
/// The hash is SHA-256 of the joined signals.
export function computeDeviceFingerprint(installSalt: string): string {
  const signals: string[] = [
    hostname(),
    `${platform()}-${release()}`,
    `${userInfo().username}-${userInfo().uid ?? 0}`,
    process.cwd(),
    installSalt,
    `${cpus().length}cpus-${Math.round(totalmem() / 1024 ** 3)}gb`,
  ];
  const joined = signals.join('|');
  return createHash('sha256').update(joined).digest('hex');
}

/// Verify a wire entitlement's signature.
///
/// Per 14-LICENSING-CLIENT.md §14.3 — 7 rejection rules:
///   1. Never accept an unsigned entitlement.
///   2. Verify before persisting.
///   3. Verify on every load.
///   4. Reject entitlements from the future.
///   5. Reject entitlements for the wrong product.
///   6. Reject entitlements for the wrong device.
///   7. Reject entitlements for the wrong channel.
export function verifyEntitlement(
  wire: WireEntitlement,
  expectedDeviceFingerprint: string,
  expectedProductId: 'disk-analyzer',
  currentAppVersion: string,
  currentChannel: 'stable' | 'beta' | 'nightly',
  publicKeyDer: Buffer,
  nowSeconds: number,
): Entitlement | ActivationError {
  const payloadJson = Buffer.from(wire.entitlement_b64, 'base64').toString('utf8');
  const signature = Buffer.from(wire.signature_b64, 'base64');

  // Parse and re-canonicalize for verification.
  let parsed: Record<string, unknown>;
  try {
    parsed = JSON.parse(payloadJson) as Record<string, unknown>;
  } catch {
    return 'SIGNATURE_VERIFICATION_FAILED';
  }
  const canonical = canonicalize(parsed);

  // Verify signature with Ed25519.
  const publicKey = createPublicKey({ key: publicKeyDer, format: 'der', type: 'spki' });
  const ok = verify(null, Buffer.from(canonical, 'utf8'), publicKey, signature);
  if (!ok) {
    return 'SIGNATURE_VERIFICATION_FAILED';
  }

  const ent = parsed as unknown as Entitlement;
  if (ent.v !== 1) {
    return 'SIGNATURE_VERIFICATION_FAILED';
  }
  if (ent.product_id !== expectedProductId) {
    return 'SIGNATURE_VERIFICATION_FAILED';
  }
  if (ent.device_fingerprint !== expectedDeviceFingerprint) {
    return 'SIGNATURE_VERIFICATION_FAILED';
  }
  if (ent.channel !== currentChannel) {
    return 'SIGNATURE_VERIFICATION_FAILED';
  }
  // App version range check.
  if (!semverSatisfies(currentAppVersion, ent.min_app_version, ent.max_app_version)) {
    return 'SIGNATURE_VERIFICATION_FAILED';
  }
  // Reject entitlements from the future (issued > now + 60s).
  if (ent.issued_at > nowSeconds + 60) {
    return 'SIGNATURE_VERIFICATION_FAILED';
  }
  return ent;
}

/// Canonical JSON: sorted keys, no whitespace. Per 13-LICENSING-SERVER.md §13.2.
function canonicalize(payload: Record<string, unknown>): string {
  const sorted: Record<string, unknown> = {};
  for (const key of Object.keys(payload).sort()) {
    sorted[key] = payload[key];
  }
  return JSON.stringify(sorted);
}

/// Naive semver "≥ min and ≤ max (if max is set)" check.
function semverSatisfies(current: string, min: string, max: string | null): boolean {
  try {
    const c = current.split('.').map(Number);
    const m = min.split('.').map(Number);
    const x = max ? max.split('.').map(Number) : null;
    if (c.length < 3 || m.length < 3) return false;
    const [cMajor, cMinor, cPatch] = c;
    const [mMajor, mMinor, mPatch] = m;
    if (cMajor! < mMajor!) return false;
    if (cMajor === mMajor && cMinor! < mMinor!) return false;
    if (cMajor === mMajor && cMinor === mMinor && cPatch! < mPatch!) return false;
    if (x) {
      const [xMajor, xMinor, xPatch] = x;
      if (cMajor! > xMajor!) return false;
      if (cMajor === xMajor && cMinor! > xMinor!) return false;
      if (cMajor === xMajor && cMinor === xMinor && cPatch! > xPatch!) return false;
    }
    return true;
  } catch {
    return false;
  }
}
