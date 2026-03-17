import { apiClient } from './api';
import type { RequestLog } from '../types/backend';

export const logsApi = {
  getRequest: async (requestId: string) => {
    const response = await apiClient.get<{ data: RequestLog[] }>('/logs', {
      params: {
        request_id: requestId,
        page: 1,
        page_size: 1,
      },
    });
    return response.data.data[0] ?? null;
  },
};
