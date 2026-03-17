export type WorkspaceId =
  | 'command-center'
  | 'traffic-lab'
  | 'provider-atlas'
  | 'route-studio'
  | 'change-studio';

export type SourceMode = 'runtime' | 'hybrid' | 'external';
export type LocaleMode = 'en' | 'zh';
export type EnvironmentMode = 'production' | 'staging';
export type TimeRangeMode = '15m' | '1h' | '6h' | '24h';

export interface ShellInspectorSection {
  title: string;
  rows: Array<{ label: string; value: string }>;
}

export interface ShellInspectorState {
  eyebrow: string;
  title: string;
  summary: string;
  sections: ShellInspectorSection[];
  actions: string[];
}
