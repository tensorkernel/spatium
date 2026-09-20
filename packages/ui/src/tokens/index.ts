// Design tokens — combined barrel.
// Per 09-DESIGN-SYSTEM.md.

export * from './colors.js';
export * from './typography.js';
export * from './spacing.js';

import { type ThemeName, darkColors, lightColors } from './colors.js';
import {
  type DensityMode,
  densityPresets,
  iconSizes,
  iconStrokeWidth,
  motion,
  radii,
  spacing,
} from './spacing.js';
import {
  fontFamilies,
  fontWeights,
  interFontFeatures,
  numericClass,
  subheaderClass,
  typeScale,
} from './typography.js';

export const tokens = {
  fonts: fontFamilies,
  type: typeScale,
  weights: fontWeights,
  subheaderClass,
  numericClass,
  interFontFeatures,
  spacing,
  density: densityPresets,
  radii,
  motion,
  icons: iconSizes,
  iconStroke: iconStrokeWidth,
  colors: { dark: darkColors, light: lightColors },
} as const;

export type Tokens = typeof tokens;
