import { useContext } from 'react';
import type { LocalizedText } from '../types/i18n';
import type { MessageKey } from './messages';
import { I18nContext, type TranslationValues } from './core';

export function useI18n() {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error('useI18n must be used within I18nProvider');
  }
  return context;
}

export function text(key: MessageKey, values?: TranslationValues): LocalizedText {
  return { key, values };
}
