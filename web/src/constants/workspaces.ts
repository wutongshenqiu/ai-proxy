import type { ShellInspectorState, WorkspaceId } from '../types/shell';
import { text } from '../i18n';

export const WORKSPACES: Array<{
  id: WorkspaceId;
  label: ReturnType<typeof text>;
  summary: ReturnType<typeof text>;
}> = [
  {
    id: 'command-center',
    label: text('shell.workspace.command-center.label'),
    summary: text('shell.workspace.command-center.summary'),
  },
  {
    id: 'traffic-lab',
    label: text('shell.workspace.traffic-lab.label'),
    summary: text('shell.workspace.traffic-lab.summary'),
  },
  {
    id: 'provider-atlas',
    label: text('shell.workspace.provider-atlas.label'),
    summary: text('shell.workspace.provider-atlas.summary'),
  },
  {
    id: 'route-studio',
    label: text('shell.workspace.route-studio.label'),
    summary: text('shell.workspace.route-studio.summary'),
  },
  {
    id: 'change-studio',
    label: text('shell.workspace.change-studio.label'),
    summary: text('shell.workspace.change-studio.summary'),
  },
];

export const DEFAULT_INSPECTORS: Record<WorkspaceId, ShellInspectorState> = {
  'command-center': {
    eyebrow: text('shell.defaultInspector.commandCenter.eyebrow'),
    title: text('shell.defaultInspector.commandCenter.title'),
    summary: text('shell.defaultInspector.commandCenter.summary'),
    sections: [],
    actions: [{ id: 'refresh-workspace', label: text('shell.action.refreshWorkspace'), effect: 'reload' }],
  },
  'traffic-lab': {
    eyebrow: text('shell.defaultInspector.trafficLab.eyebrow'),
    title: text('shell.defaultInspector.trafficLab.title'),
    summary: text('shell.defaultInspector.trafficLab.summary'),
    sections: [],
    actions: [{ id: 'refresh-workspace', label: text('shell.action.refreshWorkspace'), effect: 'reload' }],
  },
  'provider-atlas': {
    eyebrow: text('shell.defaultInspector.providerAtlas.eyebrow'),
    title: text('shell.defaultInspector.providerAtlas.title'),
    summary: text('shell.defaultInspector.providerAtlas.summary'),
    sections: [],
    actions: [{ id: 'refresh-workspace', label: text('shell.action.refreshWorkspace'), effect: 'reload' }],
  },
  'route-studio': {
    eyebrow: text('shell.defaultInspector.routeStudio.eyebrow'),
    title: text('shell.defaultInspector.routeStudio.title'),
    summary: text('shell.defaultInspector.routeStudio.summary'),
    sections: [],
    actions: [{ id: 'refresh-workspace', label: text('shell.action.refreshWorkspace'), effect: 'reload' }],
  },
  'change-studio': {
    eyebrow: text('shell.defaultInspector.changeStudio.eyebrow'),
    title: text('shell.defaultInspector.changeStudio.title'),
    summary: text('shell.defaultInspector.changeStudio.summary'),
    sections: [],
    actions: [{ id: 'refresh-workspace', label: text('shell.action.refreshWorkspace'), effect: 'reload' }],
  },
};
