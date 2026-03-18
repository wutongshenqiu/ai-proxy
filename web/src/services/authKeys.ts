import { apiClient } from './api';
import type {
  AuthKeyCreateRequest,
  AuthKeyCreateResponse,
  AuthKeyRevealResponse,
  AuthKeyUpdateRequest,
  AuthKeysResponse,
} from '../types/backend';

export const authKeysApi = {
  list: async () => (await apiClient.get<AuthKeysResponse>('/auth-keys')).data,

  create: async (body: AuthKeyCreateRequest) =>
    (await apiClient.post<AuthKeyCreateResponse>('/auth-keys', body)).data,

  update: async (id: number, body: AuthKeyUpdateRequest) =>
    (await apiClient.patch(`/auth-keys/${id}`, body)).data,

  reveal: async (id: number) =>
    (await apiClient.post<AuthKeyRevealResponse>(`/auth-keys/${id}/reveal`)).data,

  remove: async (id: number) => (await apiClient.delete(`/auth-keys/${id}`)).data,
};
