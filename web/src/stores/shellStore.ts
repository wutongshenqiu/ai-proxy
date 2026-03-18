import { create } from 'zustand';
import type {
  EnvironmentMode,
  LocaleMode,
  ShellInspectorState,
  SourceMode,
  TimeRangeMode,
  WorkspaceId,
} from '../types/shell';

interface ShellState {
  environment: EnvironmentMode;
  timeRange: TimeRangeMode;
  sourceMode: SourceMode;
  live: boolean;
  locale: LocaleMode;
  inspectors: Partial<Record<WorkspaceId, ShellInspectorState>>;
  setEnvironment: (environment: EnvironmentMode) => void;
  setTimeRange: (timeRange: TimeRangeMode) => void;
  setSourceMode: (sourceMode: SourceMode) => void;
  setLocale: (locale: LocaleMode) => void;
  setInspector: (workspace: WorkspaceId, inspector: ShellInspectorState) => void;
  toggleLive: () => void;
  toggleLocale: () => void;
}

const LOCALE_STORAGE_KEY = 'prism-control-plane:locale';
const SUPPORTED_LOCALES: LocaleMode[] = ['en-US', 'zh-CN', 'en-XA'];

function isLocaleMode(value: string): value is LocaleMode {
  return SUPPORTED_LOCALES.includes(value as LocaleMode);
}

function getStorage(): Storage | null {
  if (typeof window === 'undefined') {
    return null;
  }

  const storage = window.localStorage;
  if (
    !storage ||
    typeof storage.getItem !== 'function' ||
    typeof storage.setItem !== 'function'
  ) {
    return null;
  }

  return storage;
}

function detectLocale(): LocaleMode {
  const storage = getStorage();
  if (storage) {
    const stored = storage.getItem(LOCALE_STORAGE_KEY);
    if (stored && isLocaleMode(stored)) {
      return stored;
    }
  }

  if (typeof window === 'undefined') {
    return 'en-US';
  }

  const language = window.navigator.language.toLowerCase();
  if (language === 'en-xa') {
    return 'en-XA';
  }
  return language.startsWith('zh') ? 'zh-CN' : 'en-US';
}

function persistLocale(locale: LocaleMode) {
  const storage = getStorage();
  if (storage) {
    storage.setItem(LOCALE_STORAGE_KEY, locale);
  }
}

export const useShellStore = create<ShellState>((set) => ({
  environment: 'production',
  timeRange: '1h',
  sourceMode: 'hybrid',
  live: true,
  locale: detectLocale(),
  inspectors: {},
  setEnvironment: (environment) => set({ environment }),
  setTimeRange: (timeRange) => set({ timeRange }),
  setSourceMode: (sourceMode) => set({ sourceMode }),
  setLocale: (locale) => {
    persistLocale(locale);
    set({ locale });
  },
  setInspector: (workspace, inspector) =>
    set((state) => ({
      inspectors: {
        ...state.inspectors,
        [workspace]: inspector,
      },
    })),
  toggleLive: () => set((state) => ({ live: !state.live })),
  toggleLocale: () =>
    set((state) => {
      const locale = state.locale === 'zh-CN' ? 'en-US' : 'zh-CN';
      persistLocale(locale);
      return { locale };
    }),
}));
