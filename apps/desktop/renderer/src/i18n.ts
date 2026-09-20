// Spatium i18n — minimal hook (Phase 7 hooks per §09.13 of 09-DESIGN-SYSTEM.md).
//
// At launch, English only. The hook is here so Phase 7 localization is a
// drop-in: just add a new locale JSON under `locales/<lang>/common.json`
// and switch `currentLocale` below.

// Per TS 5+, importing JSON modules requires `resolveJsonModule: true` (set in tsconfig.base.json).
import en from './locales/en/common.json' with { type: 'json' };

type Locale = 'en';
type Messages = typeof en;

const messagesByLocale: Record<Locale, Messages> = {
  en,
};

const currentLocale: Locale = 'en';

/// Translate a message key with optional params (e.g., `t('statusBar.fileCount', { count: 123 })`).
export function t(key: string, params?: Record<string, string | number>): string {
  const messages = messagesByLocale[currentLocale];
  let value = (messages as Record<string, string>)[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      value = value.replace(`{{${k}}}`, String(v));
    }
  }
  return value;
}

/// React hook for i18n (Phase 7 — for now just returns `t`).
export function useI18n() {
  return { t, locale: currentLocale };
}
