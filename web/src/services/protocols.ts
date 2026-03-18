import { apiClient } from './api';
import type { ProtocolMatrixResponse } from '../types/backend';

export const protocolsApi = {
  matrix: async () =>
    (await apiClient.get<ProtocolMatrixResponse>('/protocols/matrix')).data,
};
