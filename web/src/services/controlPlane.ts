import { apiClient } from './api';
import type {
  ChangeStudioResponse,
  CommandCenterResponse,
  ProviderAtlasResponse,
  RouteStudioResponse,
  TrafficLabResponse,
  WorkspaceQuery,
} from '../types/controlPlane';

function workspaceParams(query: WorkspaceQuery, extras?: Record<string, number | string>) {
  return {
    range: query.range,
    source_mode: query.sourceMode,
    ...extras,
  };
}

export const controlPlaneApi = {
  commandCenter: async (query: WorkspaceQuery, signal?: AbortSignal) =>
    (await apiClient.get<CommandCenterResponse>('/control-plane/command-center', {
      params: workspaceParams(query),
      signal,
    })).data,

  trafficLab: async (query: WorkspaceQuery, signal?: AbortSignal) =>
    (await apiClient.get<TrafficLabResponse>('/control-plane/traffic-lab', {
      params: workspaceParams(query, { limit: 12 }),
      signal,
    })).data,

  providerAtlas: async (query: WorkspaceQuery, signal?: AbortSignal) =>
    (await apiClient.get<ProviderAtlasResponse>('/control-plane/provider-atlas', {
      params: workspaceParams(query),
      signal,
    })).data,

  routeStudio: async (query: WorkspaceQuery, signal?: AbortSignal) =>
    (await apiClient.get<RouteStudioResponse>('/control-plane/route-studio', {
      params: workspaceParams(query),
      signal,
    })).data,

  changeStudio: async (query: WorkspaceQuery, signal?: AbortSignal) =>
    (await apiClient.get<ChangeStudioResponse>('/control-plane/change-studio', {
      params: workspaceParams(query),
      signal,
    })).data,
};
