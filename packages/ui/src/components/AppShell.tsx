// DiskAnalyzer — three-column app shell.
// Per 09-DESIGN-SYSTEM.md §9.15 + 10-UI-COMPONENTS-AND-VIEWS.md §10.1.1.

import { type ReactNode, useEffect, useState } from 'react';

interface AppShellProps {
  leftRail: ReactNode;
  center: ReactNode;
  rightInspector: ReactNode;
  statusBar?: ReactNode;
}

export function AppShell({ leftRail, center, rightInspector, statusBar }: AppShellProps) {
  const [leftCollapsed, setLeftCollapsed] = useState(false);
  const [rightCollapsed, setRightCollapsed] = useState(false);

  // Keyboard shortcuts: Ctrl+B (toggle left), Ctrl+I (toggle right).
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'b') {
        e.preventDefault();
        setLeftCollapsed((c) => !c);
      } else if (e.ctrlKey && e.key === 'i') {
        e.preventDefault();
        setRightCollapsed((c) => !c);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-canvas text-text-primary">
      {!leftCollapsed && (
        <aside className="w-[280px] shrink-0 border-r border-border-subtle bg-surface flex flex-col">
          {leftRail}
        </aside>
      )}
      <main className="flex-1 flex flex-col min-w-0">
        {center}
        {statusBar && (
          <footer className="h-8 shrink-0 border-t border-border-subtle bg-surface px-3 flex items-center justify-between text-xs text-text-secondary">
            {statusBar}
          </footer>
        )}
      </main>
      {!rightCollapsed && (
        <aside className="w-[300px] shrink-0 border-l border-border-subtle bg-surface flex flex-col">
          {rightInspector}
        </aside>
      )}
    </div>
  );
}
