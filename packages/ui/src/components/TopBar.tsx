// Per 10-UI-COMPONENTS-AND-VIEWS.md §10.1.1 — top bar with view switcher + breadcrumb.

import type { ReactNode } from 'react';

export type ViewId =
  | 'treemap'
  | 'sunburst'
  | 'folderGrid'
  | 'topN'
  | 'donut'
  | 'ageMap'
  | 'flame'
  | 'mindmap'
  | 'bubbles'
  | 'duplicates'
  | 'history'
  | 'quickWins';

interface ViewTab {
  id: ViewId;
  label: string;
  shortcut?: string;
}

const DEFAULT_TABS: ViewTab[] = [
  { id: 'treemap', label: 'Treemap', shortcut: 'Ctrl+1' },
  { id: 'sunburst', label: 'Sunburst', shortcut: 'Ctrl+2' },
  { id: 'folderGrid', label: 'Folder Grid', shortcut: 'Ctrl+3' },
  { id: 'topN', label: 'Top-N', shortcut: 'Ctrl+4' },
  { id: 'donut', label: 'File Types', shortcut: 'Ctrl+5' },
  { id: 'ageMap', label: 'Age Map', shortcut: 'Ctrl+6' },
  { id: 'flame', label: 'Flame (exp)', shortcut: 'Ctrl+7' },
  { id: 'mindmap', label: 'Mindmap (exp)', shortcut: 'Ctrl+8' },
  { id: 'bubbles', label: 'Bubbles (exp)', shortcut: 'Ctrl+9' },
];

interface TopBarProps {
  currentView: ViewId;
  onViewChange: (v: ViewId) => void;
  breadcrumb?: ReactNode;
  toolbar?: ReactNode;
}

export function TopBar({ currentView, onViewChange, breadcrumb, toolbar }: TopBarProps) {
  return (
    <div className="flex flex-col gap-2 border-b border-border-subtle bg-surface px-3 py-2">
      <div className="flex items-center gap-2">
        <nav className="flex items-center gap-1 overflow-x-auto">
          {DEFAULT_TABS.map((tab) => {
            const active = tab.id === currentView;
            return (
              <button
                key={tab.id}
                type="button"
                onClick={() => onViewChange(tab.id)}
                className={
                  'px-3 py-1 text-xs font-medium rounded-md transition-colors ' +
                  (active
                    ? 'bg-accent-primary text-accent-primary-fg'
                    : 'text-text-secondary hover:bg-surface-3 hover:text-text-primary')
                }
                title={tab.shortcut ? `${tab.label} (${tab.shortcut})` : tab.label}
              >
                {tab.label}
              </button>
            );
          })}
        </nav>
        <div className="ml-auto text-xs text-text-tertiary">{breadcrumb}</div>
      </div>
      {toolbar && <div className="flex items-center gap-2 text-xs">{toolbar}</div>}
    </div>
  );
}
