// DiskAnalyzer — full Phase 3+ React app shell.
// Per 08-RENDERER-REACT-APP.md §8.2 + 10-UI-COMPONENTS-AND-VIEWS.md.
//
// Uses self-contained CSS classes (defined in globals.css) instead of relying
// on Tailwind utility classes being generated correctly.

import { useCallback, useEffect, useMemo, useState } from 'react';
import type { ScanOptions } from '@disk-analyzer/ipc';
import {
  AppShell,
  Sidebar,
  InspectorPanel,
  TopBar,
  type ViewId,
  StatusBar,
  DriveCard,
  CommandPalette,
  type CommandItem,
  Treemap,
  type TreeNode,
} from '@disk-analyzer/ui';
import './styles/globals.css';

interface StubScanEvent {
  name: string;
  logical: bigint;
  reparse_kind: string;
  parent_id: number;
}

export function App() {
  const [scanId, setScanId] = useState<string | null>(null);
  const [scanPhase, setScanPhase] = useState<string>('idle');
  const [filesScanned, setFilesScanned] = useState(0);
  const [bytesScanned, setBytesScanned] = useState(0n);
  const [elapsedMs, setElapsedMs] = useState(0);
  const [currentView, setCurrentView] = useState<ViewId>('treemap');
  const [selectedNodeId, setSelectedNodeId] = useState<number | null>(null);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [licenseTier, setLicenseTier] = useState<'free' | 'pro_yearly' | 'pro_lifetime'>('free');

  const [tree] = useState<TreeNode>(() => makeStubTree());

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'k') {
        e.preventDefault();
        setCommandPaletteOpen((o) => !o);
      } else if (e.ctrlKey && e.key >= '1' && e.key <= '9') {
        const idx = Number(e.key) - 1;
        const views: ViewId[] = ['treemap', 'sunburst', 'folderGrid', 'topN', 'donut', 'ageMap', 'flame', 'mindmap', 'bubbles'];
        if (views[idx]) {
          e.preventDefault();
          setCurrentView(views[idx]!);
        }
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  useEffect(() => {
    const off = window.diskanalyzer.scan.onBatch((e: unknown) => {
      const stub = e as Partial<StubScanEvent>;
      if (stub && typeof stub === 'object') {
        setFilesScanned((n) => n + 1);
        if (typeof stub.logical === 'bigint') {
          setBytesScanned((b) => b + stub.logical!);
        }
      }
    });
    return () => off();
  }, []);

  const handleStart = useCallback(async () => {
    setScanPhase('starting');
    setFilesScanned(0);
    setBytesScanned(0n);
    setElapsedMs(0);
    const opts: ScanOptions = {
      followReparsePoints: 'only-junctions-and-symlinks',
      countHardLinks: 'none',
      hashDuplicates: false,
      respectGitignore: false,
      includePatterns: [],
      excludePatterns: [],
      minSizeBytes: null,
      maxDepth: 32,
      incrementalUsn: false,
      priorScanId: null,
    };
    const result = await window.diskanalyzer.scan.start({ root: process.cwd(), options: opts });
    if (result.ok) {
      setScanId(result.scanId);
      setScanPhase('scanning');
      const poll = setInterval(async () => {
        const status = await window.diskanalyzer.scan.status(result.scanId);
        if (status === null) {
          clearInterval(poll);
          setScanPhase('complete');
        } else {
          setElapsedMs(status.elapsedMs);
          setFilesScanned(Number(status.filesScanned));
          setBytesScanned(BigInt(status.bytesScanned.toString()));
          setScanPhase(status.phase);
        }
      }, 250);
    } else {
      setScanPhase(`error: ${result.error}`);
    }
  }, []);

  const handleCancel = useCallback(async () => {
    if (scanId) {
      await window.diskanalyzer.scan.cancel(scanId);
      setScanPhase('cancelled');
    }
  }, [scanId]);

  const commandItems: CommandItem[] = useMemo(() => [
    { id: 'scan-new', label: 'New scan…', shortcut: 'Ctrl+N', group: 'Scan', action: () => handleStart() },
    { id: 'scan-cancel', label: 'Cancel current scan', group: 'Scan', action: () => handleCancel() },
    { id: 'view-treemap', label: 'View: Treemap', shortcut: 'Ctrl+1', group: 'View', action: () => setCurrentView('treemap') },
    { id: 'view-sunburst', label: 'View: Sunburst', shortcut: 'Ctrl+2', group: 'View', action: () => setCurrentView('sunburst') },
    { id: 'view-folderGrid', label: 'View: Folder Grid', shortcut: 'Ctrl+3', group: 'View', action: () => setCurrentView('folderGrid') },
    { id: 'view-topN', label: 'View: Top-N', shortcut: 'Ctrl+4', group: 'View', action: () => setCurrentView('topN') },
    { id: 'view-donut', label: 'View: File Types', shortcut: 'Ctrl+5', group: 'View', action: () => setCurrentView('donut') },
    { id: 'view-ageMap', label: 'View: Age Map (Pro)', shortcut: 'Ctrl+6', group: 'View', action: () => setCurrentView('ageMap') },
    { id: 'view-flame', label: 'View: Flame (exp)', shortcut: 'Ctrl+7', group: 'View', action: () => setCurrentView('flame') },
    { id: 'view-mindmap', label: 'View: Mindmap (exp)', shortcut: 'Ctrl+8', group: 'View', action: () => setCurrentView('mindmap') },
    { id: 'view-bubbles', label: 'View: Bubbles (exp)', shortcut: 'Ctrl+9', group: 'View', action: () => setCurrentView('bubbles') },
    { id: 'license-activate', label: 'Activate license…', group: 'Settings', action: () => alert('License dialog: enter DAPR-DEMO-0000-0000') },
    { id: 'settings', label: 'Settings…', shortcut: 'Ctrl+,', group: 'Settings', action: () => alert('Settings: Phase 6 wiring') },
    { id: 'check-updates', label: 'Check for updates', group: 'Settings', action: async () => {
      const u = await window.diskanalyzer.app.checkForUpdates();
      if (u) alert(`Update available: v${u.version}`);
      else alert('No updates available.');
    } },
  ], [handleStart, handleCancel]);

  return (
    <>
      <div className="app-shell">
        {/* LEFT RAIL */}
        <aside className="left-rail">
          <Sidebar
            driveCards={
              <DriveCard
                letter="C:"
                label="Spatium (dev)"
                totalBytes={931_000_000_000n}
                freeBytes={465_000_000_000n}
                isSSD
                isSystem
                selected={scanPhase === 'scanning' || scanPhase === 'complete'}
                onSelect={handleStart}
              />
            }
            storageDonut={
              <div style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
                Used: {formatBytes(bytesScanned)} of {formatBytes(931_000_000_000n)}
              </div>
            }
            quickWins={
              <div style={{ fontSize: '12px', color: 'var(--text-tertiary)' }}>
                {scanPhase === 'scanning' ? 'Scanning…' : 'Run a scan to see suggestions.'}
              </div>
            }
          />
        </aside>

        {/* CENTER */}
        <main className="center">
          <TopBar
            currentView={currentView}
            onViewChange={setCurrentView}
            breadcrumb={<span>Home &gt; Spatium</span>}
            toolbar={
              <div className="toolbar" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <button
                  type="button"
                  onClick={handleStart}
                  disabled={scanPhase === 'scanning'}
                  className="btn-primary"
                >
                  Scan
                </button>
                <button
                  type="button"
                  onClick={handleCancel}
                  disabled={scanPhase !== 'scanning'}
                  className="btn-secondary"
                >
                  Cancel
                </button>
                <span style={{ color: 'var(--text-tertiary)', fontSize: '12px' }}>
                  {scanPhase} {scanId ? `· ${scanId}` : ''}
                </span>
              </div>
            }
          />
          <div style={{ flex: 1, background: 'var(--bg-canvas)', overflow: 'hidden', position: 'relative' }}>
            <VisualizationContainer view={currentView} tree={tree} selectedId={selectedNodeId ?? undefined} onSelect={setSelectedNodeId} />
          </div>
          <StatusBar
            scanTimeMs={elapsedMs}
            fileCount={filesScanned}
            totalBytes={bytesScanned}
            licenseTier={licenseTier}
            telemetryEnabled={false}
          />
        </main>

        {/* RIGHT INSPECTOR */}
        <aside className="right-inspector">
          <InspectorPanel
            selection={
              selectedNodeId !== null ? (
                <div style={{ fontSize: '12px' }}>
                  <div style={{ fontWeight: 600, color: 'var(--text-primary)', marginBottom: '4px' }}>
                    Selected node #{selectedNodeId}
                  </div>
                  <div style={{ color: 'var(--text-secondary)' }}>Path: (TBD)</div>
                  <div style={{ color: 'var(--text-secondary)' }}>Size: (TBD)</div>
                  <div style={{ color: 'var(--text-secondary)' }}>Modified: (TBD)</div>
                </div>
              ) : undefined
            }
            actions={
              <>
                <button type="button" className="btn-secondary">Reveal</button>
                <button type="button" className="btn-secondary">Add to Cleanup</button>
                <button type="button" className="btn-secondary">Copy Path</button>
              </>
            }
          />
        </aside>
      </div>
      <CommandPalette items={commandItems} open={commandPaletteOpen} onOpenChange={setCommandPaletteOpen} />
    </>
  );
}

function VisualizationContainer({
  view, tree, selectedId, onSelect,
}: {
  view: ViewId;
  tree: TreeNode;
  selectedId?: number;
  onSelect?: (id: number) => void;
}) {
  switch (view) {
    case 'treemap':
      return <Treemap root={tree} selectedId={selectedId} onSelect={onSelect} width={800} height={500} />;
    case 'sunburst':
      return <StubView title="Sunburst" hint="Phase 4: implement radial treemap" />;
    case 'folderGrid':
      return <StubView title="Folder Grid" hint="Phase 4: card-based layout" />;
    case 'topN':
      return <StubView title="Top-N Largest Files" hint="Phase 4: bar list" />;
    case 'donut':
      return <StubView title="File Type Donut" hint="Phase 4: donut chart" />;
    case 'ageMap':
      return <StubView title="Age Map (Pro)" hint="Phase 4: heat by mtime" />;
    case 'flame':
      return <StubView title="Flamegraph (Experimental)" hint="Phase 7" />;
    case 'mindmap':
      return <StubView title="Mindmap (Experimental)" hint="Phase 7" />;
    case 'bubbles':
      return <StubView title="Bubbles (Experimental)" hint="Phase 7" />;
    default:
      return <StubView title={view} hint="" />;
  }
}

function StubView({ title, hint }: { title: string; hint: string }) {
  return (
    <div style={{ position: 'absolute', inset: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', textAlign: 'center' }}>
      <div>
        <div style={{ fontSize: '20px', fontWeight: 600, color: 'var(--text-primary)', marginBottom: '8px' }}>{title}</div>
        <div style={{ fontSize: '14px', color: 'var(--text-tertiary)' }}>{hint}</div>
      </div>
    </div>
  );
}

function makeStubTree(): TreeNode {
  return {
    id: 0, name: 'Root', size: 1_000_000_000, isDirectory: true, category: 'other',
    children: [
      { id: 1, name: 'Documents', size: 200_000_000, isDirectory: true, category: 'documents', children: [
        { id: 11, name: 'Resume.pdf', size: 50_000_000, isDirectory: false, category: 'documents' },
        { id: 12, name: 'Report.docx', size: 30_000_000, isDirectory: false, category: 'documents' },
        { id: 13, name: 'Slides.pptx', size: 80_000_000, isDirectory: false, category: 'documents' },
      ]},
      { id: 2, name: 'Code', size: 150_000_000, isDirectory: true, category: 'code', children: [
        { id: 21, name: 'main.rs', size: 20_000_000, isDirectory: false, category: 'code' },
        { id: 22, name: 'lib.rs', size: 40_000_000, isDirectory: false, category: 'code' },
      ]},
      { id: 3, name: 'Media', size: 400_000_000, isDirectory: true, category: 'media', children: [
        { id: 31, name: 'movie.mp4', size: 250_000_000, isDirectory: false, category: 'media' },
        { id: 32, name: 'song.flac', size: 80_000_000, isDirectory: false, category: 'media' },
        { id: 33, name: 'photo.jpg', size: 70_000_000, isDirectory: false, category: 'media' },
      ]},
      { id: 4, name: 'Caches', size: 100_000_000, isDirectory: true, category: 'caches' },
      { id: 5, name: 'Downloads', size: 150_000_000, isDirectory: true, category: 'downloads' },
    ],
  };
}

function formatBytes(b: bigint): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  let size = Number(b);
  let i = 0;
  while (size >= 1024 && i < units.length - 1) { size /= 1024; i++; }
  return `${size.toFixed(2)} ${units[i]}`;
}
