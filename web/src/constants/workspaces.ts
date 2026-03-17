import type { ShellInspectorState, WorkspaceId } from '../types/shell';

export const WORKSPACES: Array<{ id: WorkspaceId; label: string; summary: string }> = [
  { id: 'command-center', label: 'Command Center', summary: 'Signals, watch windows, and runtime posture' },
  { id: 'traffic-lab', label: 'Traffic Lab', summary: 'Request sessions, evidence, and explain flows' },
  { id: 'provider-atlas', label: 'Provider Atlas', summary: 'Provider identity, auth posture, and capability truth' },
  { id: 'route-studio', label: 'Route Studio', summary: 'Routing rules, explain truth, and scenario impact' },
  { id: 'change-studio', label: 'Change Studio', summary: 'Config registry, transaction truth, and rollout posture' },
];

export const DEFAULT_INSPECTORS: Record<WorkspaceId, ShellInspectorState> = {
  'command-center': {
    eyebrow: 'SIGNAL / LOADING',
    title: 'Runtime posture',
    summary: 'Loading current control-plane posture.',
    sections: [],
    actions: ['Refresh workspace'],
  },
  'traffic-lab': {
    eyebrow: 'SESSION / LOADING',
    title: 'Request sessions',
    summary: 'Loading current traffic evidence.',
    sections: [],
    actions: ['Refresh workspace'],
  },
  'provider-atlas': {
    eyebrow: 'PROVIDER / LOADING',
    title: 'Provider posture',
    summary: 'Loading provider identity and auth posture.',
    sections: [],
    actions: ['Refresh workspace'],
  },
  'route-studio': {
    eyebrow: 'ROUTE / LOADING',
    title: 'Routing control',
    summary: 'Loading routing explain truth.',
    sections: [],
    actions: ['Refresh workspace'],
  },
  'change-studio': {
    eyebrow: 'CHANGE / LOADING',
    title: 'Config transactions',
    summary: 'Loading configuration transaction posture.',
    sections: [],
    actions: ['Refresh workspace'],
  },
};
