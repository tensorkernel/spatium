// Per 10-UI-COMPONENTS-AND-VIEWS.md §10.1.1 — bottom status bar.

interface StatusBarProps {
  scanTimeMs?: number;
  fileCount?: number;
  totalBytes?: bigint;
  licenseTier?: 'free' | 'pro_yearly' | 'pro_lifetime';
  telemetryEnabled?: boolean;
}

export function StatusBar({
  scanTimeMs,
  fileCount,
  totalBytes,
  licenseTier,
  telemetryEnabled,
}: StatusBarProps) {
  return (
    <>
      <div className="flex items-center gap-3">
        {scanTimeMs !== undefined && (
          <span className="numeric">Scan: {(scanTimeMs / 1000).toFixed(2)}s</span>
        )}
        {fileCount !== undefined && (
          <span className="numeric">{fileCount.toLocaleString()} files</span>
        )}
        {totalBytes !== undefined && <span className="numeric">{formatBytes(totalBytes)}</span>}
      </div>
      <div className="flex items-center gap-3">
        {licenseTier && (
          <span className="rounded-full bg-surface-3 px-2 py-0.5 text-[10px] font-medium uppercase tracking-wider">
            {licenseTier.replace('_', ' ')}
          </span>
        )}
        {telemetryEnabled !== undefined && (
          <span className="text-text-tertiary">Telemetry: {telemetryEnabled ? 'on' : 'off'}</span>
        )}
        <span className="text-text-tertiary">Spatium v0.2.0</span>
      </div>
    </>
  );
}

function formatBytes(b: bigint): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  let size = Number(b);
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  return `${size.toFixed(2)} ${units[unitIndex]}`;
}
