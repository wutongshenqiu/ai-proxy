import { apiClient } from './api';
import type {
  PresentationPreviewResponse,
  ProviderCreateRequest,
  ProviderCapabilitiesResponse,
  ProviderDetail,
  ProviderHealthResult,
} from '../types/backend';

export interface ProviderUpdateRequest {
  base_url?: string | null;
  disabled?: boolean;
  region?: string | null;
  weight?: number;
}

export const providersApi = {
  list: async () =>
    (await apiClient.get<{ providers: ProviderDetail[] }>('/providers')).data,

  create: async (body: ProviderCreateRequest) =>
    (await apiClient.post('/providers', body)).data,

  get: async (name: string) =>
    (await apiClient.get<ProviderDetail>(`/providers/${encodeURIComponent(name)}`)).data,

  update: async (name: string, body: ProviderUpdateRequest) =>
    (await apiClient.patch(`/providers/${encodeURIComponent(name)}`, body)).data,

  remove: async (name: string) =>
    (await apiClient.delete(`/providers/${encodeURIComponent(name)}`)).data,

  healthCheck: async (name: string) =>
    (await apiClient.post<ProviderHealthResult>(`/providers/${encodeURIComponent(name)}/health`))
      .data,

  fetchModels: async (body: { format: string; upstream?: string; api_key: string; base_url?: string | null }) =>
    (await apiClient.post<{ models: string[] }>('/providers/fetch-models', body)).data,

  presentationPreview: async (
    name: string,
    body: { model?: string; user_agent?: string; sample_body?: unknown },
  ) =>
    (await apiClient.post<PresentationPreviewResponse>(
      `/providers/${encodeURIComponent(name)}/presentation-preview`,
      body,
    )).data,

  capabilities: async () =>
    (await apiClient.get<ProviderCapabilitiesResponse>('/providers/capabilities')).data,
};
