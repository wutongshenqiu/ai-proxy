import type { ShellInspectorState, SourceMode, TimeRangeMode } from './shell';

export type StatusTone = 'neutral' | 'success' | 'warning' | 'danger' | 'info';

export interface WorkspaceQuery {
  range: TimeRangeMode;
  sourceMode: SourceMode;
}

export interface FactRow {
  label: string;
  value: string;
}

export interface KpiMetric {
  label: string;
  value: string;
  delta: string;
}

export interface SignalItem {
  id: string;
  title: string;
  detail: string;
  severity: string;
  severity_tone: StatusTone;
  target_workspace: string;
}

export interface CommandCenterResponse {
  kpis: KpiMetric[];
  signals: SignalItem[];
  pressure_map: FactRow[];
  watch_windows: FactRow[];
  inspector: ShellInspectorState;
}

export interface TrafficSessionItem {
  request_id: string;
  model: string;
  decision: string;
  result: string;
  result_tone: StatusTone;
  latency_ms: number;
}

export interface TimelineStep {
  label: string;
  tone: StatusTone;
  title: string;
  detail: string;
}

export interface TrafficLabResponse {
  selected_request_id: string | null;
  sessions: TrafficSessionItem[];
  compare_facts: FactRow[];
  trace: TimelineStep[];
  inspector: ShellInspectorState;
}

export interface ProviderAtlasRow {
  provider: string;
  format: string;
  auth: string;
  status: string;
  status_tone: StatusTone;
  rotation: string;
  region: string;
  wire_api: string;
  model_count: number;
}

export interface ProviderAtlasResponse {
  providers: ProviderAtlasRow[];
  coverage: FactRow[];
  inspector: ShellInspectorState;
}

export interface RouteScenarioRow {
  scenario: string;
  winner: string;
  delta: string;
  decision: string;
  decision_tone: StatusTone;
  endpoint: string;
  source_format: string;
  stream: boolean;
  model: string;
  tenant_id: string | null;
  api_key_id: string | null;
  region: string | null;
}

export interface RouteStudioResponse {
  summary_facts: FactRow[];
  explain_facts: FactRow[];
  scenarios: RouteScenarioRow[];
  inspector: ShellInspectorState;
}

export interface RegistryRow {
  family: string;
  record: string;
  state: string;
  state_tone: StatusTone;
  dependents: string;
}

export interface ChangeStudioResponse {
  registry: RegistryRow[];
  publish_facts: FactRow[];
  inspector: ShellInspectorState;
}
