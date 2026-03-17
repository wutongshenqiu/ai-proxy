import { apiClient } from './api';
import type {
  AuthProfileConnectRequest,
  AuthProfileCreateRequest,
  CodexDevicePollResponse,
  CodexDeviceStartRequest,
  CodexDeviceStartResponse,
  CodexOauthStartRequest,
  CodexOauthStartResponse,
  AuthProfilesListResponse,
  AuthProfileMutationResponse,
  AuthProfilesRuntimeResponse,
} from '../types/backend';

export const authProfilesApi = {
  list: async () =>
    (await apiClient.get<AuthProfilesListResponse>('/auth-profiles')).data,

  runtime: async () =>
    (await apiClient.get<AuthProfilesRuntimeResponse>('/auth-profiles/runtime')).data,

  create: async (body: AuthProfileCreateRequest) =>
    (await apiClient.post<AuthProfileMutationResponse>('/auth-profiles', body)).data,

  replace: async (provider: string, profileId: string, body: Omit<AuthProfileCreateRequest, 'provider' | 'id'>) =>
    (await apiClient.put<AuthProfileMutationResponse>(
      `/auth-profiles/${encodeURIComponent(provider)}/${encodeURIComponent(profileId)}`,
      body,
    )).data,

  startCodexOauth: async (body: CodexOauthStartRequest) =>
    (await apiClient.post<CodexOauthStartResponse>('/auth-profiles/codex/oauth/start', body)).data,

  completeCodexOauth: async (state: string, code: string) =>
    (await apiClient.post<AuthProfileMutationResponse>('/auth-profiles/codex/oauth/complete', { state, code })).data,

  startCodexDevice: async (body: CodexDeviceStartRequest) =>
    (await apiClient.post<CodexDeviceStartResponse>('/auth-profiles/codex/device/start', body)).data,

  pollCodexDevice: async (state: string) =>
    (await apiClient.post<CodexDevicePollResponse>('/auth-profiles/codex/device/poll', { state })).data,

  connect: async (provider: string, profileId: string, body: AuthProfileConnectRequest) =>
    (await apiClient.post<AuthProfileMutationResponse>(
      `/auth-profiles/${encodeURIComponent(provider)}/${encodeURIComponent(profileId)}/connect`,
      body,
    )).data,

  importLocal: async (provider: string, profileId: string, path?: string) =>
    (await apiClient.post<AuthProfileMutationResponse>(
      `/auth-profiles/${encodeURIComponent(provider)}/${encodeURIComponent(profileId)}/import-local`,
      path ? { path } : {},
    )).data,

  refresh: async (provider: string, profileId: string) =>
    (await apiClient.post<AuthProfileMutationResponse>(
      `/auth-profiles/${encodeURIComponent(provider)}/${encodeURIComponent(profileId)}/refresh`,
      {},
    )).data,

  remove: async (provider: string, profileId: string) =>
    apiClient.delete(`/auth-profiles/${encodeURIComponent(provider)}/${encodeURIComponent(profileId)}`),
};
