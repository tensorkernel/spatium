// DiskAnalyzer — telemetry pipeline.
// Per 15-SECURITY-ARCHITECTURE.md §15.9 + 02-SYSTEM-ARCHITECTURE.md §2.9.
//
// Hard rule: telemetry NEVER includes file paths, file names, file sizes, or file contents.
// Only: app/OS version, architecture, scan-duration aggregate, scan total file count,
// scan total bytes scanned, channel, license tier (anonymized), crash stacks (redacted).

export interface TelemetryEvent {
  event: string; // 'app.launch' | 'scan.start' | 'scan.complete' | 'scan.error' | 'crash' | ...
  app_version: string;
  os_version: string;
  arch: string;
  channel: 'stable' | 'beta' | 'nightly';
  timestamp: number;
  [key: string]: unknown;
}

/// Path-like string detector for redaction. Matches:
/// - Windows paths: `C:\foo\bar`, `C:/foo/bar`
/// - Unix paths: `/home/user`, `/tmp/...`
/// - URLs: `http://...` / `https://...`
/// - File names with extensions: `name.ext` (simplified)
const PATH_RE =
  /([A-Za-z]:[\\/][^\s"'<>|*?]+)|(\/(?:home|Users|tmp|var|etc|root|mnt)[^\s"'<>|*?]*)|(\bhttps?:\/\/[^\s"'<>|*?]+)|(\b[a-zA-Z0-9_\-]+\.(?:txt|exe|dll|node|json|rs|ts|tsx|js|jsx|md|zip|tar|gz|7z|wxs)\b)/g;

/// Redact any path-like string from the event. Per §15.9.
/// Returns a NEW object; the input is not mutated.
export function redactPaths<T extends Record<string, unknown>>(event: T): T {
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(event)) {
    out[k] = redactValue(v);
  }
  return out as T;
}

function redactValue(v: unknown): unknown {
  if (typeof v === 'string') {
    return redactString(v);
  }
  if (Array.isArray(v)) {
    return v.map(redactValue);
  }
  if (v !== null && typeof v === 'object') {
    return redactPaths(v as Record<string, unknown>);
  }
  return v;
}

function redactString(s: string): string {
  return s.replace(PATH_RE, '[REDACTED]');
}

/// The TelemetryService. Per §15.9.
export class TelemetryService {
  private optIn = false;
  private events: TelemetryEvent[] = [];
  private endpoint: string;

  constructor(opts: { endpoint: string; optIn?: boolean }) {
    this.endpoint = opts.endpoint;
    this.optIn = opts.optIn ?? false;
  }

  setOptIn(optIn: boolean): void {
    this.optIn = optIn;
  }

  isOptedIn(): boolean {
    return this.optIn;
  }

  /// Emit an event. Redacted before queueing.
  emit(event: TelemetryEvent): void {
    if (!this.optIn) return;
    const redacted = redactPaths(event);
    this.events.push(redacted);
    if (this.events.length >= 50) {
      void this.flush();
    }
  }

  /// Flush queued events to the endpoint.
  async flush(): Promise<void> {
    if (!this.optIn || this.events.length === 0) return;
    const toSend = this.events;
    this.events = [];
    try {
      await fetch(this.endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ events: toSend }),
      });
    } catch (err) {
      console.error('[telemetry] flush failed:', err);
    }
  }
}

// ─────────────────────────────────────────────────────────────────────
// Self-tests (basic; run via vitest in Phase 4 wiring)
// ─────────────────────────────────────────────────────────────────────

// Run via vitest:
//   import { redactPaths } from './telemetry.js';
//   test('redactPaths redacts Windows paths', () => {
//     const e = redactPaths({ event: 'scan.error', message: 'Cannot open C:\\Users\\jane\\file.txt' });
//     expect(e.message).toBe('Cannot open [REDACTED]');
//   });
//   test('redactPaths redacts Unix paths', () => {
//     const e = redactPaths({ event: 'scan.error', message: 'failed to read /home/user/.ssh/key' });
//     expect(e.message).toBe('failed to read [REDACTED]');
//   });
//   test('redactPaths does not redact simple strings', () => {
//     const e = redactPaths({ event: 'app.launch', duration_ms: 1500 });
//     expect(e.duration_ms).toBe(1500);
//     expect(e.event).toBe('app.launch');
//   });
//   test('redactPaths redacts URLs', () => {
//     const e = redactPaths({ event: 'update.check', url: 'https://updates.spatium.app/manifest.json' });
//     expect(e.url).toBe('[REDACTED]');
//   });
