// Design tokens — colors.
// Per 09-DESIGN-SYSTEM.md §9.2.
//
// These mirror the CSS custom properties in `packages/ui/src/styles/tokens.css`.
// Components should prefer Tailwind classes (e.g. `bg-surface`) over the raw
// hex values; these constants are for places that need the literal value
// (e.g. Canvas 2D stroke styles, visx color scales, d3 color scales).

export interface ColorToken {
  readonly hex: string;
}

/** Dark theme (default). Per 09-DESIGN-SYSTEM.md §9.2.1. */
export const darkColors = {
  bgCanvas: { hex: '#0F1115' },
  bgSurface: { hex: '#161A21' },
  bgSurface2: { hex: '#1C2129' },
  bgSurface3: { hex: '#232934' },
  bgElevated: { hex: '#1A1F27' },

  textPrimary: { hex: '#E6E9EF' },
  textSecondary: { hex: '#9BA3AF' },
  textTertiary: { hex: '#6B7280' },
  textInverse: { hex: '#0F1115' },

  borderSubtle: { hex: '#1F2530' },
  borderDefault: { hex: '#2A313D' },
  borderStrong: { hex: '#3D4452' },

  accentPrimary: { hex: '#3B82F6' },
  accentPrimaryHover: { hex: '#2563EB' },
  accentPrimaryFg: { hex: '#FFFFFF' },
  accentWarning: { hex: '#F59E0B' },
  accentDanger: { hex: '#EF4444' },
  accentSuccess: { hex: '#10B981' },
  accentInfo: { hex: '#06B6D4' },

  // Treemap categorical hues (8) — warm muted pastels per the design-system research.
  cat1Documents: { hex: '#F4C2B0' }, // peach
  cat2Code: { hex: '#C5D8A8' }, // sage
  cat3System: { hex: '#BDD9E8' }, // sky
  cat4Media: { hex: '#D4C4E0' }, // lavender
  cat5Caches: { hex: '#F9E79F' }, // pale yellow
  cat6Downloads: { hex: '#F5D5CB' }, // coral
  cat7UserData: { hex: '#DFECD9' }, // moss
  cat8Apps: { hex: '#E8C9DB' }, // mauve

  // Shadows
  shadowSm: { hex: 'rgba(0,0,0,0.40)' },
  shadowMd: { hex: 'rgba(0,0,0,0.45)' },
  shadowLg: { hex: 'rgba(0,0,0,0.55)' },

  // Mica fallback (Windows 10)
  micaTint: { hex: 'rgba(15,17,21,0.7)' },
} as const;

/** Light theme. Per 09-DESIGN-SYSTEM.md §9.2.2. */
export const lightColors = {
  bgCanvas: { hex: '#F6F6F8' },
  bgSurface: { hex: '#FFFFFF' },
  bgSurface2: { hex: '#F0F1F4' },
  bgSurface3: { hex: '#E7E9ED' },
  bgElevated: { hex: '#FFFFFF' },

  textPrimary: { hex: '#1D1D1F' },
  textSecondary: { hex: '#86868B' },
  textTertiary: { hex: '#B0B0B5' },
  textInverse: { hex: '#FFFFFF' },

  borderSubtle: { hex: '#ECEEF1' },
  borderDefault: { hex: '#D8DCE0' },
  borderStrong: { hex: '#B8BDC4' },

  accentPrimary: { hex: '#3B82F6' },
  accentPrimaryHover: { hex: '#2563EB' },
  accentPrimaryFg: { hex: '#FFFFFF' },
  accentWarning: { hex: '#D97706' },
  accentDanger: { hex: '#DC2626' },
  accentSuccess: { hex: '#059669' },
  accentInfo: { hex: '#0891B2' },

  // Categorical hues (slightly more saturated for light bg)
  cat1Documents: { hex: '#E76F51' },
  cat2Code: { hex: '#6A994E' },
  cat3System: { hex: '#4A86C5' },
  cat4Media: { hex: '#9B6FB0' },
  cat5Caches: { hex: '#D4A017' },
  cat6Downloads: { hex: '#E07A5F' },
  cat7UserData: { hex: '#588157' },
  cat8Apps: { hex: '#B567A1' },

  shadowSm: { hex: 'rgba(0,0,0,0.06)' },
  shadowMd: { hex: 'rgba(0,0,0,0.08)' },
  shadowLg: { hex: 'rgba(0,0,0,0.10)' },

  micaTint: { hex: 'rgba(255,255,255,0.85)' },
} as const;

export type ThemeName = 'dark' | 'light';

export interface ColorPalette {
  readonly bgCanvas: { readonly hex: string };
  readonly bgSurface: { readonly hex: string };
  readonly bgSurface2: { readonly hex: string };
  readonly bgSurface3: { readonly hex: string };
  readonly bgElevated: { readonly hex: string };
  readonly textPrimary: { readonly hex: string };
  readonly textSecondary: { readonly hex: string };
  readonly textTertiary: { readonly hex: string };
  readonly textInverse: { readonly hex: string };
  readonly borderSubtle: { readonly hex: string };
  readonly borderDefault: { readonly hex: string };
  readonly borderStrong: { readonly hex: string };
  readonly accentPrimary: { readonly hex: string };
  readonly accentPrimaryHover: { readonly hex: string };
  readonly accentPrimaryFg: { readonly hex: string };
  readonly accentWarning: { readonly hex: string };
  readonly accentDanger: { readonly hex: string };
  readonly accentSuccess: { readonly hex: string };
  readonly accentInfo: { readonly hex: string };
  readonly cat1Documents: { readonly hex: string };
  readonly cat2Code: { readonly hex: string };
  readonly cat3System: { readonly hex: string };
  readonly cat4Media: { readonly hex: string };
  readonly cat5Caches: { readonly hex: string };
  readonly cat6Downloads: { readonly hex: string };
  readonly cat7UserData: { readonly hex: string };
  readonly cat8Apps: { readonly hex: string };
  readonly shadowSm: { readonly hex: string };
  readonly shadowMd: { readonly hex: string };
  readonly shadowLg: { readonly hex: string };
  readonly micaTint: { readonly hex: string };
}

export const colorsByTheme: Record<ThemeName, ColorPalette> = {
  dark: darkColors,
  light: lightColors,
};

/**
 * Resolve a categorical hue by file-type category.
 * Per 09-DESIGN-SYSTEM.md §9.2.3 — the mapping of extensions to categories
 * lives in the Rust core (crates/core/src/aggregate/extensions.rs).
 */
export function categoryColor(
  theme: ThemeName,
  category: import('@disk-analyzer/ipc').FileCategory,
): string {
  const c = colorsByTheme[theme];
  switch (category) {
    case 'documents':
      return c.cat1Documents.hex;
    case 'code':
      return c.cat2Code.hex;
    case 'system':
      return c.cat3System.hex;
    case 'media':
      return c.cat4Media.hex;
    case 'caches':
      return c.cat5Caches.hex;
    case 'downloads':
      return c.cat6Downloads.hex;
    case 'userdata':
      return c.cat7UserData.hex;
    case 'apps':
      return c.cat8Apps.hex;
    case 'other':
      return c.textTertiary.hex;
    default:
      return c.textTertiary.hex;
  }
}
