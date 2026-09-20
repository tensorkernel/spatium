// Per 10-UI-COMPONENTS-AND-VIEWS.md §10.1.1 — sidebar (left rail).
// Holds drive cards, storage donut, quick wins, file-type legend.

import type { ReactNode } from 'react';

interface SidebarProps {
  driveCards?: ReactNode;
  storageDonut?: ReactNode;
  quickWins?: ReactNode;
  legend?: ReactNode;
}

export function Sidebar({ driveCards, storageDonut, quickWins, legend }: SidebarProps) {
  return (
    <div className="flex flex-col gap-4 p-3 overflow-y-auto">
      {driveCards && <section>{driveCards}</section>}
      {storageDonut && (
        <section>
          <h2 className="text-xs font-medium uppercase tracking-[0.04em] text-text-secondary mb-2">
            Disk Storage
          </h2>
          {storageDonut}
        </section>
      )}
      {quickWins && (
        <section>
          <h2 className="text-xs font-medium uppercase tracking-[0.04em] text-text-secondary mb-2">
            Quick Wins
          </h2>
          {quickWins}
        </section>
      )}
      {legend && (
        <section>
          <h2 className="text-xs font-medium uppercase tracking-[0.04em] text-text-secondary mb-2">
            File Types
          </h2>
          {legend}
        </section>
      )}
    </div>
  );
}
