import { render, screen } from '@testing-library/react';
import App from './App';
import { useAuthStore } from './stores/authStore';

describe('control-plane shell', () => {
  beforeEach(() => {
    useAuthStore.setState({
      username: 'admin',
      isAuthenticated: true,
      isLoading: false,
      initialized: true,
      error: null,
    });
    window.history.replaceState({}, '', '/command-center');
  });

  it('renders the command center entry and workspace navigation', () => {
    render(<App />);
    expect(screen.getByText('Prism')).toBeInTheDocument();
    expect(screen.getAllByText('Command Center').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Traffic Lab').length).toBeGreaterThan(0);
    expect(screen.getByText('Control plane')).toBeInTheDocument();
  });
});
