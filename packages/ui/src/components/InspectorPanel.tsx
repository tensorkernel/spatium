// Per 10-UI-COMPONENTS-AND-VIEWS.md §10.1.1 — right inspector panel.

import type { ReactNode } from 'react';

interface InspectorPanelProps {
  selection?: ReactNode;
  actions?: ReactNode;
}

export function InspectorPanel({ selection, actions }: InspectorPanelProps) {
  return (
    <div className="flex flex-col gap-4 p-3 overflow-y-auto">
      <h2 className="text-xs font-medium uppercase tracking-[0.04em] text-text-secondary">
        Inspector
      </h2>
      {selection ?? <div className="text-sm text-text-tertiary">No selection</div>}
      {actions && <section className="flex flex-col gap-2">{actions}</section>}
    </div>
  );
}
