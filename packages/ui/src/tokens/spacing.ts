// Design tokens — spacing, radii, shadows, motion, density.
// Per 09-DESIGN-SYSTEM.md §9.4–9.9.

// 4px base grid per 09-DESIGN-SYSTEM.md §9.4.
export const spacing = {
  px: 1,
  half: 2,
  1: 4,
  15: 6,
  2: 8,
  3: 12,
  4: 16,
  5: 20,
  6: 24,
  8: 32,
  10: 40,
  12: 48,
  16: 64,
  20: 80,
  24: 96,
} as const;

export type DensityMode = 'compact' | 'comfortable' | 'spacious';

export const densityPresets = {
  compact: { rowHeight: 24, cellPadding: 4, bodyPadding: 8 },
  comfortable: { rowHeight: 32, cellPadding: 6, bodyPadding: 12 },
  spacious: { rowHeight: 40, cellPadding: 8, bodyPadding: 16 },
} as const;

// Radii per 09-DESIGN-SYSTEM.md §9.5.
export const radii = {
  sm: 4,
  md: 6,
  lg: 8,
  xl: 12,
  twoxl: 16,
  full: 9999,
} as const;

// Motion per 09-DESIGN-SYSTEM.md §9.7.
// Hard rule (P2): animate ONLY transform and opacity. No parallax, no
// bouncy springs on data, no width/height animations.
export const motion = {
  durationFast: 100, // ms — toast enter/exit
  durationBase: 150, // ms — default chrome transitions
  durationSlow: 250, // ms — drill-in/out
  durationSlower: 400, // ms — onboarding transitions

  easeOut: 'cubic-bezier(0.16, 1, 0.3, 1)',
  easeIn: 'cubic-bezier(0.7, 0, 0.84, 0)',
  easeInOut: 'cubic-bezier(0.65, 0, 0.35, 1)',
} as const;

// Icon sizes (Lucide). Per 09-DESIGN-SYSTEM.md §9.8.
export const iconSizes = {
  xs: 12, // sublabel
  sm: 14, // button
  md: 16, // default
  lg: 20, // header
  xl: 24, // large
} as const;

export const iconStrokeWidth = {
  default: 1.5,
  bold: 2.0,
} as const;
