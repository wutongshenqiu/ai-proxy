import { create } from 'zustand';
import { getApiErrorMessage } from '../services/errors';
import { authApi, setSessionSetter } from '../services/api';

interface AuthState {
  username: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  initialized: boolean;
  error: string | null;
  initialize: () => Promise<void>;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
}

function applySession(authenticated: boolean, username?: string | null) {
  useAuthStore.setState((state) => ({
    username: authenticated ? (username ?? state.username) : null,
    isAuthenticated: authenticated,
    initialized: true,
    isLoading: false,
    error: authenticated ? null : state.error,
  }));
}

export const useAuthStore = create<AuthState>((set) => ({
  username: null,
  isAuthenticated: false,
  isLoading: true,
  initialized: false,
  error: null,

  initialize: async () => {
    set({ isLoading: true });
    try {
      const response = await authApi.session();
      applySession(response.data.authenticated, response.data.username);
    } catch {
      applySession(false, null);
    }
  },

  login: async (username: string, password: string) => {
    set({ isLoading: true, error: null });
    try {
      const response = await authApi.login(username, password);
      applySession(response.data.authenticated, response.data.username);
    } catch (error) {
      const message = getApiErrorMessage(error, 'Login failed');
      set({
        username: null,
        isAuthenticated: false,
        isLoading: false,
        initialized: true,
        error: message,
      });
      throw error;
    }
  },

  logout: async () => {
    try {
      await authApi.logout();
    } finally {
      applySession(false, null);
    }
  },
}));

setSessionSetter((authenticated) => {
  applySession(authenticated);
});
