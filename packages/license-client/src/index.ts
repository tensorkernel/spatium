// DiskAnalyzer — license client + updater + telemetry.
// Per 14-LICENSING-CLIENT.md + 07-ELECTRON-APP-SHELL.md §7.5 + 15-SECURITY-ARCHITECTURE.md §15.9.

export { LicenseClient } from './client.js';
export {
  type Entitlement,
  type WireEntitlement,
  type LicenseStatus,
  type ActivationResult,
  type ActivationError,
  verifyEntitlement,
  computeDeviceFingerprint,
} from './entitlement.js';
export { Updater, type UpdateManifest, type UpdateInfo } from './updater.js';
export { TelemetryService, type TelemetryEvent, redactPaths } from './telemetry.js';
