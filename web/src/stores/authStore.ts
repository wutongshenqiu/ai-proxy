import { create } from 'zustand';
import { authApi, setTokenSetter } from '../services/api';
import { destroyWebSocketManager } from '../services/websocket';

interface AuthState {
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  login: (username: string, password: string) => Promise<void>;
  logout: () => void;
  refreshToken: () => Promise<void>;
  initialize: () => void;
}

// Single point of truth for persisting and updating token state.
// All paths (login, refresh, interceptor) converge here.
function applyToken(token: string | null) {
  if (token) {
    localStorage.setItem('auth_token', token);
    useAuthStore.setState({ token, isAuthenticated: true });
  } else {
    localStorage.removeItem('auth_token');
    destroyWebSocketManager();
    useAuthStore.setState({ token: null, isAuthenticated: false });
  }
}

// Read token synchronously so ProtectedRoute sees it on first render
const savedToken = localStorage.getItem('auth_token');

export const useAuthStore = create<AuthState>((set) => ({
  token: savedToken,
  isAuthenticated: !!savedToken,
  isLoading: false,
  error: null,

  initialize: () => {
    const token = localStorage.getItem('auth_token');
    if (token) {
      set({ token, isAuthenticated: true });
    }
  },

  login: async (username: string, password: string) => {
    set({ isLoading: true, error: null });
    try {
      const response = await authApi.login(username, password);
      applyToken(response.data.token);
      set({ isLoading: false });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Login failed';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  logout: () => {
    applyToken(null);
  },

  refreshToken: async () => {
    try {
      const response = await authApi.refresh();
      applyToken(response.data.token);
    } catch {
      applyToken(null);
    }
  },
}));

// Register the unified setter so the Axios interceptor can update auth state
// on token refresh (avoiding circular imports).
setTokenSetter(applyToken);
