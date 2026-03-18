import { useMemo, type PropsWithChildren } from 'react';
import { enMessages, messageCatalogs } from './messages';
import { I18nContext, interpolate, type I18nContextValue } from './core';
import { useShellStore } from '../stores/shellStore';

export function I18nProvider({ children }: PropsWithChildren) {
  const locale = useShellStore((state) => state.locale);

  const value = useMemo<I18nContextValue>(() => {
    const catalog = messageCatalogs[locale];
    const number = new Intl.NumberFormat(locale);

    const t = (key: string, values?: Record<string, string | number | boolean | null | undefined>) => {
      const template =
        (catalog as Record<string, string>)[key] ??
        (enMessages as Record<string, string>)[key] ??
        key;
      return interpolate(template, values);
    };

    return {
      locale,
      t,
      tx: (text) => t(text.key, text.values),
      formatNumber: (input, options) => new Intl.NumberFormat(locale, options).format(input),
      formatCurrency: (input, currency = 'USD') =>
        new Intl.NumberFormat(locale, {
          style: 'currency',
          currency,
          maximumFractionDigits: 2,
        }).format(input),
      formatDateTime: (input, options) =>
        new Intl.DateTimeFormat(locale, {
          dateStyle: 'medium',
          timeStyle: 'short',
          ...options,
        }).format(new Date(input)),
      formatDurationMs: (input) => {
        if (input < 1_000) {
          return t('format.duration.ms', { value: number.format(input) });
        }
        if (input < 60_000) {
          return t('format.duration.seconds', {
            value: new Intl.NumberFormat(locale, { maximumFractionDigits: 1 }).format(input / 1_000),
          });
        }
        if (input < 3_600_000) {
          return t('format.duration.minutes', {
            value: new Intl.NumberFormat(locale, { maximumFractionDigits: 1 }).format(input / 60_000),
          });
        }
        return t('format.duration.hours', {
          value: new Intl.NumberFormat(locale, { maximumFractionDigits: 1 }).format(input / 3_600_000),
        });
      },
    };
  }, [locale]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}
