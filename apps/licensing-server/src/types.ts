// Shared types for the licensing server.

export type LicenseTier = 'free' | 'pro_yearly' | 'pro_lifetime';

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

export interface ActivateRequest {
  key: string;
  device_fingerprint: string;
  device_name?: string;
  app_version: string;
  platform: 'win32' | 'darwin' | 'linux';
  arch: 'x64' | 'arm64';
}

export interface RevalidateRequest {
  current_entitlement_b64?: string;
  current_entitlement_signature_b64?: string;
}

export interface DeviceRecord {
  id: string;
  fingerprint: string;
  name: string;
  platform: string;
  arch: string;
  first_seen_at: number;
  last_seen_at: number;
}

export interface LicenseRecord {
  key: string;
  tier: Exclude<LicenseTier, 'free'>;
  customer_id: string;
  customer_email: string;
  issued_at: number;
  expires_at: number | null;
  max_devices: number;
  status: 'active' | 'revoked' | 'expired' | 'subscription_cancelled';
  devices: DeviceRecord[];
  features: string[];
  channel: 'stable' | 'beta' | 'nightly';
  min_app_version: string;
  max_app_version: string | null;
}
