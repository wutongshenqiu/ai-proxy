import { apiClient } from './api';
import type { RouteExplanation, RoutingConfig, RoutingUpdateRequest } from '../types/backend';

export interface RouteIntrospectionRequest {
  model: string;
  endpoint?: string;
  source_format?: string;
  tenant_id?: string | null;
  api_key_id?: string | null;
  region?: string | null;
  stream?: boolean;
  headers?: Record<string, string>;
  routing_override?: RoutingConfig;
}

export const routingApi = {
  get: async () =>
    (await apiClient.get<RoutingConfig>('/routing')).data,

  update: async (body: RoutingUpdateRequest) =>
    (await apiClient.patch('/routing', body)).data,

  preview: async (body: RouteIntrospectionRequest) =>
    (await apiClient.post<RouteExplanation>('/routing/preview', body)).data,

  explain: async (body: RouteIntrospectionRequest) =>
    (await apiClient.post<RouteExplanation>('/routing/explain', body)).data,
};
