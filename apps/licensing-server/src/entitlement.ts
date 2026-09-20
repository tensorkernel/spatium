// DiskAnalyzer licensing server — entitlement model.
//
// Per 13-LICENSING-SERVER.md §13.2.
//
// The entitlement is signed, NOT encrypted. Authenticity/integrity is what matters;
// secrecy of the payload doesn't. The client verifies the signature with an
// embedded public key; a modified client cannot forge a new entitlement
// without the private signing key (which lives only here).

import type { LicenseTier } from './types.js';

/// Server-side entitlement payload. Per 13-LICENSING-SERVER.md §13.2.
export interface EntitlementPayload {
  v: 1;
  license_id: string;
  product_id: 'disk-analyzer';
  tier: Exclude<LicenseTier, 'free'>;
  customer_id: string;
  customer_email: string;
  features: string[];
  issued_at: number;
  expires_at: number | null; // null = lifetime
  grace_until: number;
  device_fingerprint: string;
  device_id: string;
  max_devices: number;
  channel: 'stable' | 'beta' | 'nightly';
  min_app_version: string;
  max_app_version: string | null;
  nonce: string;
}

/// Wire format returned to the client. Per 13-LICENSING-SERVER.md §13.2.
export interface WireEntitlement {
  entitlement_b64: string;
  signature_b64: string;
  alg: 'Ed25519';
  ver: 1;
}

/// Canonical JSON for signature stability. Sorted keys, no whitespace.
/// Per 13-LICENSING-SERVER.md §13.2 (canonicalization).
export function canonicalize(payload: EntitlementPayload): string {
  const sorted: Record<string, unknown> = {};
  for (const key of Object.keys(payload).sort()) {
    sorted[key] = (payload as unknown as Record<string, unknown>)[key];
  }
  return JSON.stringify(sorted);
}
