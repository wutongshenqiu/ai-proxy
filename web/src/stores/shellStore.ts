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
  setInspector: (workspace: WorkspaceId, inspector: ShellInspectorState) => void;
  toggleLive: () => void;
  toggleLocale: () => void;
}

export const useShellStore = create<ShellState>((set) => ({
  environment: 'production',
  timeRange: '1h',
  sourceMode: 'hybrid',
  live: true,
  locale: 'en',
  inspectors: {},
  setEnvironment: (environment) => set({ environment }),
  setTimeRange: (timeRange) => set({ timeRange }),
  setSourceMode: (sourceMode) => set({ sourceMode }),
  setInspector: (workspace, inspector) =>
    set((state) => ({
      inspectors: {
        ...state.inspectors,
        [workspace]: inspector,
      },
    })),
  toggleLive: () => set((state) => ({ live: !state.live })),
  toggleLocale: () => set((state) => ({ locale: state.locale === 'en' ? 'zh' : 'en' })),
}));
