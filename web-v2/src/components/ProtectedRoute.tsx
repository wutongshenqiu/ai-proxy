import { useEffect } from 'react';
import { Navigate } from 'react-router-dom';
import { useAuthStore } from '../stores/authStore';

interface ProtectedRouteProps {
  children: React.ReactNode;
}

export function ProtectedRoute({ children }: ProtectedRouteProps) {
  const initialized = useAuthStore((state) => state.initialized);
  const isLoading = useAuthStore((state) => state.isLoading);
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  const initialize = useAuthStore((state) => state.initialize);

  useEffect(() => {
    if (!initialized) {
      void initialize();
    }
  }, [initialize, initialized]);

  if (!initialized || isLoading) {
    return (
      <div className="auth-screen">
        <div className="auth-card auth-card--compact">
          <p className="workspace-eyebrow">PRISM / SESSION</p>
          <h1>Restoring dashboard session</h1>
          <p className="auth-copy">Checking the cookie-backed dashboard session before opening the control plane.</p>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}
