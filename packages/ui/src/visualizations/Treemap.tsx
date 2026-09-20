// Treemap visualization — squarified algorithm.
// Per 11-VISUALIZATIONS.md §11.3.
//
// Re-implemented from the squarified treemap paper (Bruls, Huijsen, van Wijk, 2000).
// Per Per architecture §6.7: "the renderer computes the treemap layout" (not Rust).

import { useEffect, useMemo, useRef, useState } from 'react';

export interface TreeNode {
  id: number;
  name: string;
  size: number; // bytes
  isDirectory: boolean;
  category?: string;
  children?: TreeNode[];
}

interface TreemapProps {
  root: TreeNode;
  selectedId?: number;
  onSelect?: (id: number) => void;
  onDrillInto?: (id: number) => void;
  width?: number;
  height?: number;
  colorBy?: 'type' | 'age' | 'depth' | 'size';
}

interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

interface LaidOutNode {
  node: TreeNode;
  rect: Rect;
  depth: number;
}

const CATEGORY_COLORS: Record<string, string> = {
  documents: 'var(--cat-1)',
  code: 'var(--cat-2)',
  system: 'var(--cat-3)',
  media: 'var(--cat-4)',
  caches: 'var(--cat-5)',
  downloads: 'var(--cat-6)',
  userdata: 'var(--cat-7)',
  apps: 'var(--cat-8)',
  other: 'var(--text-tertiary)',
};

const MIN_CELL_SIZE = 4; // px — per §11.3 anti-pattern

export function Treemap({
  root,
  selectedId,
  onSelect,
  onDrillInto,
  width = 800,
  height = 600,
  colorBy = 'type',
}: TreemapProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [hovered, setHovered] = useState<LaidOutNode | null>(null);
  const [tooltip, setTooltip] = useState<{ x: number; y: number; text: string } | null>(null);

  // Layout: compute squarified treemap.
  const layout = useMemo(() => {
    const result: LaidOutNode[] = [];
    const rootRect: Rect = { x: 0, y: 0, w: width, h: height };
    squarify(root, rootRect, 0, result);
    return result;
  }, [root, width, height]);

  // Render to canvas.
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set DPI for crispness.
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, width, height);

    for (const item of layout) {
      const { node, rect } = item;
      if (rect.w < MIN_CELL_SIZE || rect.h < MIN_CELL_SIZE) continue;

      // Fill color
      const color = colorForNode(node, colorBy);
      ctx.fillStyle = color;
      ctx.fillRect(rect.x, rect.y, rect.w, rect.h);

      // Selection border
      if (selectedId === node.id) {
        ctx.strokeStyle = 'var(--accent-primary)';
        ctx.lineWidth = 2;
        ctx.strokeRect(rect.x + 1, rect.y + 1, rect.w - 2, rect.h - 2);
      } else {
        // Subtle border for separation
        ctx.strokeStyle = 'rgba(0,0,0,0.2)';
        ctx.lineWidth = 0.5;
        ctx.strokeRect(rect.x, rect.y, rect.w, rect.h);
      }

      // Label (only if cell is large enough)
      if (rect.w >= 32 && rect.h >= 16) {
        ctx.fillStyle = 'rgba(255,255,255,0.95)';
        ctx.font = '10px Inter, sans-serif';
        ctx.textBaseline = 'top';
        const name = truncate(node.name, rect.w - 8, ctx);
        if (name) {
          ctx.fillText(name, rect.x + 4, rect.y + 4);
        }
      }
    }
  }, [layout, selectedId, colorBy, width, height]);

  // Mouse handlers.
  const handleMove = (e: React.MouseEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const item = layout.find(
      (it) =>
        x >= it.rect.x &&
        x < it.rect.x + it.rect.w &&
        y >= it.rect.y &&
        y < it.rect.y + it.rect.h &&
        it.rect.w >= MIN_CELL_SIZE &&
        it.rect.h >= MIN_CELL_SIZE,
    );
    if (item !== hovered) {
      setHovered(item ?? null);
      if (item) {
        setTooltip({
          x: e.clientX,
          y: e.clientY,
          text: `${item.node.name}\n${formatBytes(item.node.size)}`,
        });
      } else {
        setTooltip(null);
      }
    } else if (item) {
      setTooltip({
        x: e.clientX,
        y: e.clientY,
        text: `${item.node.name}\n${formatBytes(item.node.size)}`,
      });
    }
  };

  const handleClick = (e: React.MouseEvent) => {
    if (!hovered) return;
    if (e.detail === 2 && onDrillInto) {
      onDrillInto(hovered.node.id);
    } else {
      onSelect?.(hovered.node.id);
    }
  };

  return (
    <div className="relative">
      <canvas
        ref={canvasRef}
        onMouseMove={handleMove}
        onMouseLeave={() => {
          setHovered(null);
          setTooltip(null);
        }}
        onClick={handleClick}
        className="cursor-pointer"
      />
      {tooltip && (
        <div
          className="fixed z-50 pointer-events-none px-2 py-1 rounded-md bg-surface-elevated border border-border-default shadow-md text-xs whitespace-pre-line"
          style={{ left: tooltip.x + 12, top: tooltip.y + 12 }}
        >
          {tooltip.text}
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────
// Squarified algorithm — Bruls, Huijsen, van Wijk (2000)
// ─────────────────────────────────────────────────────────────────────

function squarify(node: TreeNode, rect: Rect, depth: number, out: LaidOutNode[]) {
  out.push({ node, rect, depth });

  if (!node.children || node.children.length === 0) return;

  const totalChildSize = node.children.reduce((sum, c) => sum + c.size, 0);
  if (totalChildSize === 0) return;

  // Sort children descending by size (algorithm requires this).
  const sorted = [...node.children].sort((a, b) => b.size - a.size);

  // Scale children's areas to fit `rect`.
  const rectArea = rect.w * rect.h;
  const scale = rectArea / totalChildSize;

  // Lay out using the squarified algorithm.
  const scaled = sorted.map((c) => ({ node: c, area: c.size * scale }));
  layoutRow(scaled, rect, depth + 1, out);
}

function layoutRow(
  items: { node: TreeNode; area: number }[],
  rect: Rect,
  depth: number,
  out: LaidOutNode[],
) {
  let remaining = items;
  let workingRect = rect;

  while (remaining.length > 0) {
    const shortestSide = Math.min(workingRect.w, workingRect.h);
    const totalArea = remaining.reduce((sum, it) => sum + it.area, 0);

    // Take a prefix of items whose worst aspect ratio improves with each addition.
    let row: { node: TreeNode; area: number }[] = [];
    let worstWithout: number = Number.POSITIVE_INFINITY;
    for (const item of remaining) {
      const candidate = [...row, item];
      const worst = worstAspectRatio(candidate, totalArea);
      if (row.length === 0 || worst <= worstWithout) {
        row = candidate;
        worstWithout = worst;
      } else {
        break;
      }
    }

    // Lay out the row.
    const rowArea = row.reduce((sum, it) => sum + it.area, 0);
    const isHorizontal = workingRect.w >= workingRect.h;
    const rowThickness = rowArea / (isHorizontal ? workingRect.h : workingRect.w);
    const remainingTotalArea = totalArea - rowArea;
    const newRect = isHorizontal
      ? {
          x: workingRect.x + rowThickness,
          y: workingRect.y,
          w: workingRect.w - rowThickness,
          h: workingRect.h,
        }
      : {
          x: workingRect.x,
          y: workingRect.y + rowThickness,
          w: workingRect.w,
          h: workingRect.h - rowThickness,
        };

    // Place each item in the row.
    let offset = 0;
    for (const it of row) {
      const length = it.area / rowThickness;
      const itemRect = isHorizontal
        ? { x: workingRect.x, y: workingRect.y + offset, w: rowThickness, h: length }
        : { x: workingRect.x + offset, y: workingRect.y, w: length, h: rowThickness };
      squarify(it.node, itemRect, depth, out);
      offset += length;
    }

    workingRect = newRect;
    remaining = remaining.slice(row.length);
    if (remaining.length === 0) break;
    // Recompute totalArea for remaining items (we removed `row` from the rect; the scale is now different).
    const newRectArea = workingRect.w * workingRect.h;
    if (newRectArea <= 0) break;
    const newScale = newRectArea / remainingTotalArea;
    remaining = remaining.map((it) => ({
      node: it.node,
      area: it.area * (newScale / (totalArea / newRectArea + 1e-9)),
    }));
  }
}

function worstAspectRatio(row: { area: number }[], totalArea: number): number {
  if (row.length === 0) return Number.POSITIVE_INFINITY;
  const sum = row.reduce((s, it) => s + it.area, 0);
  let max = Number.NEGATIVE_INFINITY;
  let min = Number.POSITIVE_INFINITY;
  for (const it of row) {
    if (it.area > max) max = it.area;
    if (it.area < min) min = it.area;
  }
  const w = sum / totalArea;
  const a = (w * w * max) / (sum * sum);
  const b = (sum * sum) / (w * w * min);
  return Math.max(a, b);
}

function colorForNode(node: TreeNode, colorBy: string): string {
  if (colorBy === 'type' && node.category) {
    return CATEGORY_COLORS[node.category] ?? CATEGORY_COLORS.other ?? 'var(--bg-surface-2)';
  }
  // Default: surface color
  return 'var(--bg-surface-2)';
}

function truncate(name: string, maxWidth: number, ctx: CanvasRenderingContext2D): string {
  if (ctx.measureText(name).width <= maxWidth) return name;
  let truncated = name;
  while (truncated.length > 1 && ctx.measureText(truncated + '…').width > maxWidth) {
    truncated = truncated.slice(0, -1);
  }
  return truncated.length > 0 ? truncated + '…' : '';
}

function formatBytes(b: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  let size = b;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  return `${size.toFixed(2)} ${units[unitIndex]}`;
}
