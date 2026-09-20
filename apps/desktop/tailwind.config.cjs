/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './renderer/index.html',
    './renderer/src/**/*.{ts,tsx}',
    '../../packages/ui/src/**/*.{ts,tsx}',
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
  plugins: [require('tailwindcss-animate')],
};
