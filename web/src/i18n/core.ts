import { createContext } from 'react';
import type { LocalizedPrimitive, LocalizedText } from '../types/i18n';
import type { LocaleMode } from '../types/shell';

export type TranslationValues = Record<string, LocalizedPrimitive> | undefined;

export interface I18nContextValue {
  locale: LocaleMode;
  t: (key: string, values?: TranslationValues) => string;
  tx: (text: LocalizedText) => string;
  formatNumber: (value: number, options?: Intl.NumberFormatOptions) => string;
  formatCurrency: (value: number, currency?: string) => string;
  formatDateTime: (value: string | number | Date, options?: Intl.DateTimeFormatOptions) => string;
  formatDurationMs: (value: number) => string;
}

export function interpolate(template: string, values?: TranslationValues) {
  if (!values) {
    return template;
  }

  return template.replace(/\{(\w+)\}/g, (_, name: string) => {
    const value = values[name];
    return value === undefined || value === null ? '' : String(value);
  });
}

export const I18nContext = createContext<I18nContextValue | null>(null);
