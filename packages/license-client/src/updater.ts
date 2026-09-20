// DiskAnalyzer — auto-updater.
// Per 07-ELECTRON-APP-SHELL.md §7.5 + 15-SECURITY-ARCHITECTURE.md §15.6.
//
// Phase 5 stub: fetches the signed manifest from the update channel, verifies
// the Ed25519 signature, checks the version policy. The actual download + apply
// + rollback is delegated to the bootstrapper (Phase 5 production).

import { createHash, createPublicKey, verify } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

export interface UpdateManifest {
  v: 1;
  channel: 'stable' | 'beta' | 'nightly';
  version: string;
  min_app_version: string;
  target_arch: string;
  package_url: string;
  package_sha256: string;
  package_size_bytes: number;
  release_notes_url: string;
  release_notes_summary: string;
  critical_security_update: boolean;
  signature: string; // Ed25519 over canonical JSON of all fields above
}

export interface UpdateInfo {
  version: string;
  channel: 'stable' | 'beta' | 'nightly';
  releaseNotesSummary: string;
  releaseNotesUrl: string;
  packageSizeBytes: number;
  criticalSecurityUpdate: boolean;
}

/// The updater. Per 15-SECURITY-ARCHITECTURE.md §15.6.
export class Updater {
  constructor(
    private endpoint: string, // 'https://updates.spatium.app/stable/manifest.json' or 'http://localhost:8081/manifest.json'
    private publicKeyDer: Buffer, // prod or dev, depending on build
    private currentVersion: string,
    private currentChannel: 'stable' | 'beta' | 'nightly',
    private currentArch: string, // 'x64' or 'arm64'
    private httpClient: typeof fetch = fetch,
  ) {}

  /// Check for an available update. Per §15.6.1.
  async check(): Promise<UpdateInfo | null> {
    try {
      const res = await this.httpClient(`${this.endpoint}/manifest.json`);
      if (!res.ok) return null;
      const manifest = (await res.json()) as UpdateManifest;

      // Verify signature.
      if (!this.verifyManifest(manifest)) {
        console.error('[updater] manifest signature verification failed');
        return null;
      }

      // Version policy.
      if (!this.versionPolicyAllows(manifest)) {
        return null;
      }

      return {
        version: manifest.version,
        channel: manifest.channel,
        releaseNotesSummary: manifest.release_notes_summary,
        releaseNotesUrl: manifest.release_notes_url,
        packageSizeBytes: manifest.package_size_bytes,
        criticalSecurityUpdate: manifest.critical_security_update,
      };
    } catch (err) {
      console.error('[updater] check failed:', err);
      return null;
    }
  }

  /// Verify the manifest signature.
  /// Per §15.6.1 — Ed25519 over the canonical JSON of all fields except `signature`.
  private verifyManifest(manifest: UpdateManifest): boolean {
    const { signature, ...rest } = manifest;
    const canonical = canonicalize(rest);
    const signatureBytes = Buffer.from(signature, 'base64');
    const publicKey = createPublicKey({ key: this.publicKeyDer, format: 'der', type: 'spki' });
    return verify(null, Buffer.from(canonical, 'utf8'), publicKey, signatureBytes);
  }

  /// Version policy per §15.6.3.
  /// - Channel downgrade forbidden (stable client cannot accept beta/nightly manifest).
  /// - Version downgrade forbidden (manifest.version > currentVersion).
  /// - min_app_version must be ≤ currentVersion.
  /// - target_arch must match.
  private versionPolicyAllows(manifest: UpdateManifest): boolean {
    if (manifest.channel !== this.currentChannel) return false;
    if (!semverGreater(manifest.version, this.currentVersion)) return false;
    if (!semverAtLeast(this.currentVersion, manifest.min_app_version)) return false;
    if (manifest.target_arch !== this.currentArch) return false;
    return true;
  }

  /// Stage the update package (download + verify hash). Returns the staged path.
  /// Per §15.6.1. Phase 5 production: also verify Authenticode signature.
  async downloadAndStage(manifest: UpdateManifest): Promise<string> {
    const tmpDir = join(homedir(), '.spatium', 'updates');
    if (!existsSync(tmpDir)) {
      mkdirSync(tmpDir, { recursive: true });
    }
    const pkgPath = join(tmpDir, `spatium-${manifest.version}.zip`);

    // Download.
    const res = await this.httpClient(manifest.package_url);
    if (!res.ok) {
      throw new Error(`download failed: HTTP ${res.status}`);
    }
    const buf = Buffer.from(await res.arrayBuffer());

    // Verify SHA-256.
    const actualHash = createHash('sha256').update(buf).digest('hex');
    if (actualHash !== manifest.package_sha256) {
      throw new Error(
        `package hash mismatch: expected ${manifest.package_sha256}, got ${actualHash}`,
      );
    }

    // Stage.
    writeFileSync(pkgPath, buf);
    console.log(`[updater] Staged update at ${pkgPath}`);
    return pkgPath;
  }
}

function canonicalize(obj: Record<string, unknown>): string {
  const sorted: Record<string, unknown> = {};
  for (const key of Object.keys(obj).sort()) {
    sorted[key] = obj[key];
  }
  return JSON.stringify(sorted);
}

function semverGreater(a: string, b: string): boolean {
  const av = a.split('.').map(Number);
  const bv = b.split('.').map(Number);
  if (av.length < 3 || bv.length < 3) return false;
  for (let i = 0; i < 3; i++) {
    if (av[i]! > bv[i]!) return true;
    if (av[i]! < bv[i]!) return false;
  }
  return false;
}

function semverAtLeast(a: string, b: string): boolean {
  const av = a.split('.').map(Number);
  const bv = b.split('.').map(Number);
  if (av.length < 3 || bv.length < 3) return false;
  for (let i = 0; i < 3; i++) {
    if (av[i]! > bv[i]!) return true;
    if (av[i]! < bv[i]!) return false;
  }
  return true;
}
