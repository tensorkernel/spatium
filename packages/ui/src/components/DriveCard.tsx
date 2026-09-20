// Per 10-UI-COMPONENTS-AND-VIEWS.md §10.1.2 — DriveCard.

interface DriveCardProps {
  letter: string; // "C:"
  label: string; // "Windows"
  totalBytes: bigint;
  freeBytes: bigint;
  isSSD: boolean;
  isSystem?: boolean;
  selected?: boolean;
  onSelect?: () => void;
}

export function DriveCard({
  letter,
  label,
  totalBytes,
  freeBytes,
  isSSD,
  isSystem,
  selected,
  onSelect,
}: DriveCardProps) {
  const usedBytes = totalBytes - freeBytes;
  const usedPct = totalBytes > 0n ? Number((usedBytes * 100n) / totalBytes) : 0;

  return (
    <button
      type="button"
      onClick={onSelect}
      className={
        'w-full text-left p-3 rounded-lg border transition-colors ' +
        (selected
          ? 'border-accent-primary bg-accent-primary/10'
          : 'border-border-subtle bg-surface-2 hover:bg-surface-3 hover:border-border-default')
      }
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <span className="font-mono text-sm font-semibold">{letter}</span>
          <span className="text-xs text-text-secondary truncate max-w-[120px]">{label}</span>
        </div>
        <div className="flex items-center gap-1">
          {isSystem && (
            <span className="text-[9px] uppercase tracking-wider text-accent-warning px-1 py-0.5 rounded bg-accent-warning/10">
              System
            </span>
          )}
          <span className="text-[9px] uppercase tracking-wider text-text-tertiary px-1 py-0.5 rounded bg-surface-3">
            {isSSD ? 'SSD' : 'HDD'}
          </span>
        </div>
      </div>
      <div className="h-1.5 rounded-full bg-surface-3 overflow-hidden mb-1">
        <div
          className={
            'h-full rounded-full ' +
            (usedPct > 90
              ? 'bg-accent-danger'
              : usedPct > 75
                ? 'bg-accent-warning'
                : 'bg-accent-primary')
          }
          style={{ width: `${usedPct}%` }}
        />
      </div>
      <div className="flex items-center justify-between text-xs text-text-secondary numeric">
        <span>{formatBytes(usedBytes)} used</span>
        <span>{formatBytes(freeBytes)} free</span>
      </div>
    </button>
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
  return `${size.toFixed(1)} ${units[unitIndex]}`;
}
