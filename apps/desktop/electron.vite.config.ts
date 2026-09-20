import { resolve } from 'node:path';
import react from '@vitejs/plugin-react';
import { defineConfig, externalizeDepsPlugin } from 'electron-vite';
import tailwindcss from 'tailwindcss';
import autoprefixer from 'autoprefixer';

// Per 07-ELECTRON-APP-SHELL.md §7.2 and §7.16.
const rootDir = resolve(__dirname, '../..');

// Workspace packages that MUST be bundled (not externalized) because they
// only have .ts source files (no pre-built .js).
const WORKSPACE_PACKAGES = [
  '@disk-analyzer/ipc',
  '@disk-analyzer/license-client',
  '@disk-analyzer/ui',
];

// Tailwind config (inline so Vite/PostCSS finds it regardless of working directory)
const tailwindConfig = {
  content: [
    resolve(__dirname, 'renderer/index.html'),
    resolve(__dirname, 'renderer/src/**/*.{ts,tsx}'),
    resolve(rootDir, 'packages/ui/src/**/*.{ts,tsx}'),
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        canvas: 'var(--bg-canvas)',
        surface: { DEFAULT: 'var(--bg-surface)', 2: 'var(--bg-surface-2)', 3: 'var(--bg-surface-3)', elevated: 'var(--bg-elevated)' },
        text: { DEFAULT: 'var(--text-primary)', primary: 'var(--text-primary)', secondary: 'var(--text-secondary)', tertiary: 'var(--text-tertiary)', inverse: 'var(--text-inverse)' },
        border: { DEFAULT: 'var(--border-default)', subtle: 'var(--border-subtle)', strong: 'var(--border-strong)' },
        accent: { DEFAULT: 'var(--accent-primary)', primary: 'var(--accent-primary)', 'primary-hover': 'var(--accent-primary-hover)', 'primary-fg': 'var(--accent-primary-fg)', warning: 'var(--accent-warning)', danger: 'var(--accent-danger)', success: 'var(--accent-success)', info: 'var(--accent-info)' },
        cat: { 1: 'var(--cat-1)', 2: 'var(--cat-2)', 3: 'var(--cat-3)', 4: 'var(--cat-4)', 5: 'var(--cat-5)', 6: 'var(--cat-6)', 7: 'var(--cat-7)', 8: 'var(--cat-8)' },
      },
      fontFamily: { sans: ['Inter Variable', 'Inter', 'system-ui', 'sans-serif'], mono: ['JetBrains Mono Variable', 'JetBrains Mono', 'ui-monospace', 'monospace'] },
      borderRadius: { sm: 'var(--radius-sm)', md: 'var(--radius-md)', lg: 'var(--radius-lg)', xl: 'var(--radius-xl)', '2xl': 'var(--radius-2xl)' },
      boxShadow: { sm: 'var(--shadow-sm)', md: 'var(--shadow-md)', lg: 'var(--shadow-lg)' },
    },
  },
  plugins: [],
};

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin({ exclude: WORKSPACE_PACKAGES })],
    build: {
      rollupOptions: {
        input: { index: resolve(__dirname, 'src/main.ts') },
        output: { dir: resolve(__dirname, 'out/main') },
      },
    },
    resolve: {
      alias: {
        '@disk-analyzer/ipc': resolve(rootDir, 'packages/ipc/src/index.ts'),
        '@disk-analyzer/napi': resolve(rootDir, 'crates/napi/index.js'),
        '@disk-analyzer/license-client': resolve(rootDir, 'packages/license-client/src/index.ts'),
      },
    },
  },
  preload: {
    plugins: [externalizeDepsPlugin({ exclude: WORKSPACE_PACKAGES })],
    build: {
      rollupOptions: {
        input: { index: resolve(__dirname, 'preload/index.ts') },
        output: {
          dir: resolve(__dirname, 'out/preload'),
          format: 'cjs',
          entryFileNames: '[name].js',
        },
      },
    },
    resolve: {
      alias: {
        '@disk-analyzer/ipc': resolve(rootDir, 'packages/ipc/src/index.ts'),
      },
    },
  },
  renderer: {
    root: 'renderer',
    plugins: [react()],
    resolve: {
      alias: {
        '@disk-analyzer/ipc': resolve(rootDir, 'packages/ipc/src/index.ts'),
        '@disk-analyzer/ui': resolve(rootDir, 'packages/ui/src/index.ts'),
      },
    },
    // Explicitly configure PostCSS so Tailwind works regardless of config file discovery
    css: {
      postcss: {
        plugins: [
          tailwindcss(tailwindConfig),
          autoprefixer(),
        ],
      },
    },
    build: {
      rollupOptions: {
        input: { index: resolve(__dirname, 'renderer/index.html') },
        output: { dir: resolve(__dirname, 'out/renderer') },
      },
    },
    server: {
      port: 5173,
      fs: {
        allow: [rootDir],
      },
    },
  },
});
