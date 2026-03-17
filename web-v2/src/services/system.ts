import { apiClient } from './api';
import type { SystemHealthResponse, SystemLogsResponse } from '../types/backend';

export interface SystemLogsQuery {
  page?: number;
  page_size?: number;
  level?: string;
  search?: string;
}

export const systemApi = {
  health: async () => (await apiClient.get<SystemHealthResponse>('/system/health')).data,

  logs: async (params: SystemLogsQuery = {}) =>
    (await apiClient.get<SystemLogsResponse>('/system/logs', { params })).data,
};
