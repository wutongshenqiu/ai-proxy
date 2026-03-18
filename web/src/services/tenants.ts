import { apiClient } from './api';
import type { TenantMetricsResponse, TenantsResponse } from '../types/backend';

export const tenantsApi = {
  list: async () => (await apiClient.get<TenantsResponse>('/tenants')).data,

  metrics: async (tenantId: string) =>
    (await apiClient.get<TenantMetricsResponse>(`/tenants/${encodeURIComponent(tenantId)}/metrics`)).data,
};
