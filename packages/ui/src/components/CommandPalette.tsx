// Per 10-UI-COMPONENTS-AND-VIEWS.md §10.3 — Command palette (Ctrl+K).
// Uses shadcn/ui's Command pattern (cmdk under the hood).

import { useEffect, useState } from 'react';

export interface CommandItem {
  id: string;
  label: string;
  shortcut?: string;
  group?: string;
  action?: () => void;
}

interface CommandPaletteProps {
  items: CommandItem[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function CommandPalette({ items, open, onOpenChange }: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [activeIndex, setActiveIndex] = useState(0);

  useEffect(() => {
    if (!open) {
      setQuery('');
      setActiveIndex(0);
    }
  }, [open]);

  // Escape to close, Enter to activate, arrows to navigate.
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        onOpenChange(false);
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        setActiveIndex((i) => Math.min(i + 1, filtered.length - 1));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setActiveIndex((i) => Math.max(i - 1, 0));
      } else if (e.key === 'Enter') {
        e.preventDefault();
        const item = filtered[activeIndex];
        if (item?.action) {
          item.action();
        }
        onOpenChange(false);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  });

  if (!open) return null;

  const q = query.toLowerCase();
  const filtered = items.filter((it) => {
    if (!q) return true;
    return it.label.toLowerCase().includes(q) || (it.group?.toLowerCase().includes(q) ?? false);
  });

  // Group items
  const groups: Record<string, CommandItem[]> = {};
  for (const it of filtered) {
    const g = it.group ?? 'Commands';
    (groups[g] ??= []).push(it);
  }

  let flatIndex = 0;
  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/50"
      onClick={() => onOpenChange(false)}
    >
      <div
        className="w-[640px] max-w-[90vw] rounded-lg border border-border-default bg-surface-elevated shadow-lg overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <input
          autoFocus
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Type a command…"
          className="w-full px-4 py-3 bg-transparent text-sm text-text-primary placeholder:text-text-tertiary border-b border-border-subtle outline-none"
        />
        <div className="max-h-[60vh] overflow-y-auto py-2">
          {Object.entries(groups).map(([group, items]) => (
            <div key={group} className="mb-2">
              <div className="px-4 py-1 text-[10px] font-medium uppercase tracking-wider text-text-tertiary">
                {group}
              </div>
              {items.map((it) => {
                const idx = flatIndex++;
                const active = idx === activeIndex;
                return (
                  <button
                    key={it.id}
                    type="button"
                    onMouseEnter={() => setActiveIndex(idx)}
                    onClick={() => {
                      it.action?.();
                      onOpenChange(false);
                    }}
                    className={
                      'w-full flex items-center justify-between px-4 py-2 text-left text-sm transition-colors ' +
                      (active
                        ? 'bg-accent-primary/15 text-accent-primary'
                        : 'text-text-primary hover:bg-surface-3')
                    }
                  >
                    <span>{it.label}</span>
                    {it.shortcut && (
                      <span className="text-xs text-text-tertiary font-mono">{it.shortcut}</span>
                    )}
                  </button>
                );
              })}
            </div>
          ))}
          {filtered.length === 0 && (
            <div className="px-4 py-6 text-center text-sm text-text-tertiary">
              No commands match "{query}"
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
