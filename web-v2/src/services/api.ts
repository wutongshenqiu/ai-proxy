import axios from 'axios';
import type { LoginResponse, SessionResponse } from '../types/backend';

export const apiClient = axios.create({
  baseURL: '/api/dashboard',
  timeout: 15_000,
  withCredentials: true,
  headers: {
    'Content-Type': 'application/json',
  },
});

let sessionSetter: ((authenticated: boolean) => void) | null = null;

export function setSessionSetter(setter: (authenticated: boolean) => void) {
  sessionSetter = setter;
}

function applySession(authenticated: boolean) {
  sessionSetter?.(authenticated);
}

apiClient.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config ?? {};
    const url = String(originalRequest.url ?? '');
    const isAuthRequest =
      url.includes('/auth/login') ||
      url.includes('/auth/refresh') ||
      url.includes('/auth/session') ||
      url.includes('/auth/logout');

    if (error.response?.status === 401 && !originalRequest._retry && !isAuthRequest) {
      originalRequest._retry = true;
      try {
        await axios.post<LoginResponse>('/api/dashboard/auth/refresh', null, {
          withCredentials: true,
        });
        applySession(true);
        return apiClient(originalRequest);
      } catch {
        applySession(false);
      }
    }

    return Promise.reject(error);
  },
);

export const authApi = {
  login: (username: string, password: string) =>
    apiClient.post<LoginResponse>('/auth/login', { username, password }),
  refresh: () => apiClient.post<LoginResponse>('/auth/refresh'),
  session: () => apiClient.get<SessionResponse>('/auth/session'),
  logout: () => apiClient.post<SessionResponse>('/auth/logout'),
};
