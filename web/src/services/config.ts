import { apiClient } from './api';
import type {
  ConfigApplyResponse,
  ConfigValidateResponse,
  RawConfigResponse,
} from '../types/backend';

export const configApi = {
  raw: async () => (await apiClient.get<RawConfigResponse>('/config/raw')).data,

  validate: async (yaml: string) =>
    (await apiClient.post<ConfigValidateResponse>('/config/validate', { yaml })).data,

  apply: async (yaml: string, configVersion?: string) =>
    (await apiClient.put<ConfigApplyResponse>('/config/apply', {
      yaml,
      config_version: configVersion,
    })).data,

  reload: async () => (await apiClient.post<{ message: string }>('/config/reload')).data,
};
