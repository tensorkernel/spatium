// ESLint flat config (ESLint 9+).
// Per 18-CI-CD-RELEASE-PIPELINE.md §4.2 deliverables.
// Biome handles formatting + most linting; ESLint adds TS-specific rules.
// (We can drop ESLint once Biome reaches feature parity for the rules we care about; for now both ship.)

import js from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: [
      '**/node_modules/**',
      '**/dist/**',
      '**/target/**',
      '**/out/**',
      '**/release/**',
      '**/*.node',
      '**/.vite/**',
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.strict,
  {
    rules: {
      // Per 20-QUALITY-GATES-AND-POVS.md §20.2.1 SD-4: no `any` without a // reason: comment.
      '@typescript-eslint/no-explicit-any': 'error',
      // Per SD-7: prefer @ts-expect-error with a reason.
      '@typescript-eslint/ban-ts-comment': [
        'error',
        {
          'ts-expect-error': 'allow-with-description',
          'ts-ignore': true,
          'ts-nocheck': true,
          'ts-check': false,
          minimumDescriptionLength: 10,
        },
      ],
      // Per SD-9: TODO requires owner + tracking issue
      'no-warning-comments': [
        'warn',
        {
          terms: ['TODO', 'FIXME', 'XXX'],
          location: 'anywhere',
        },
      ],
      '@typescript-eslint/no-unused-vars': [
        'warn',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
    },
  },
  {
    // Test files: allow console, allow non-null assertions in test fixtures
    files: ['**/*.test.ts', '**/*.test.tsx', '**/*.spec.ts', '**/tests/**'],
    rules: {
      '@typescript-eslint/no-non-null-assertion': 'off',
      'no-console': 'off',
    },
  },
);
