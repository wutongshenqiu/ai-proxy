import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import { AppShell } from './components/AppShell';
import { ProtectedRoute } from './components/ProtectedRoute';
import { AuthProfileCallbackPage } from './pages/AuthProfileCallbackPage';
import { ChangeStudioPage } from './pages/ChangeStudioPage';
import { CommandCenterPage } from './pages/CommandCenterPage';
import { LoginPage } from './pages/LoginPage';
import { ProviderAtlasPage } from './pages/ProviderAtlasPage';
import { RouteStudioPage } from './pages/RouteStudioPage';
import { TrafficLabPage } from './pages/TrafficLabPage';

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route
          element={(
            <ProtectedRoute>
              <AppShell />
            </ProtectedRoute>
          )}
        >
          <Route index element={<Navigate to="/command-center" replace />} />
          <Route path="/provider-atlas/callback" element={<AuthProfileCallbackPage />} />
          <Route path="/command-center" element={<CommandCenterPage />} />
          <Route path="/traffic-lab" element={<TrafficLabPage />} />
          <Route path="/provider-atlas" element={<ProviderAtlasPage />} />
          <Route path="/route-studio" element={<RouteStudioPage />} />
          <Route path="/change-studio" element={<ChangeStudioPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
