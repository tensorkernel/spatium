// Design tokens — typography.
// Per 09-DESIGN-SYSTEM.md §9.3.
//
// Inter Variable (300-900 weights) is the primary typeface. JetBrains Mono
// Variable is the monospace companion for path / hash / size columns.
// Both ship inside the Electron app at apps/desktop/resources/fonts/.

export const fontFamilies = {
  sans: 'Inter Variable, Inter, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
  mono: 'JetBrains Mono Variable, JetBrains Mono, ui-monospace, "SF Mono", Menlo, monospace',
} as const;

// Type scale. Per 09-DESIGN-SYSTEM.md §9.3.2.
export const typeScale = {
  text2xs: { size: 10, lineHeight: 14, tailwind: 'text-[10px] leading-[14px]' },
  textXs: { size: 11, lineHeight: 16, tailwind: 'text-xs' },
  textSm: { size: 13, lineHeight: 20, tailwind: 'text-sm' },
  textBase: { size: 14, lineHeight: 22, tailwind: 'text-base' },
  textLg: { size: 16, lineHeight: 24, tailwind: 'text-lg' },
  textXl: { size: 20, lineHeight: 28, tailwind: 'text-xl' },
  text2xl: { size: 24, lineHeight: 32, tailwind: 'text-2xl' },
  text3xl: { size: 32, lineHeight: 40, tailwind: 'text-3xl' },
} as const;

export const fontWeights = {
  thin: 100,
  light: 300,
  regular: 400,
  medium: 500,
  semibold: 600,
  bold: 700,
  extrabold: 800,
  black: 900,
} as const;

// Subheader treatment: medium, xs, uppercase, tracking 0.04em.
export const subheaderClass =
  'text-xs font-medium uppercase tracking-[0.04em] text-secondary' as const;

// Tabular numerals: applied via the `numeric` class anywhere a number appears in a column.
// Per 09-DESIGN-SYSTEM.md §9.3.5.
export const numericClass = 'numeric' as const;

// Inter opentype features for tabular numerals (cv01, cv03, cv04 — alt 1, 4, 6/9 styles).
export const interFontFeatures = "'tnum' 1, 'cv01' 1, 'cv03' 1, 'cv04' 1" as const;
