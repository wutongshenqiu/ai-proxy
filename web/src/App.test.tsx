import { render, screen } from '@testing-library/react';
import App from './App';
import { I18nProvider } from './i18n';
import { useAuthStore } from './stores/authStore';
import { useShellStore } from './stores/shellStore';

describe('control-plane shell', () => {
  beforeEach(() => {
    useAuthStore.setState({
      username: 'admin',
      isAuthenticated: true,
      isLoading: false,
      initialized: true,
      error: null,
    });
    useShellStore.setState({ locale: 'en-US' });
    window.history.replaceState({}, '', '/command-center');
  });

  it('renders the command center entry and workspace navigation', () => {
    render(
      <I18nProvider>
        <App />
      </I18nProvider>,
    );
    expect(screen.getByText('Prism')).toBeInTheDocument();
    expect(screen.getAllByText('Command Center').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Traffic Lab').length).toBeGreaterThan(0);
    expect(screen.getByText('Control plane')).toBeInTheDocument();
  });

  it('renders localized shell copy when locale is switched to Chinese', () => {
    useShellStore.setState({ locale: 'zh-CN' });

    render(
      <I18nProvider>
        <App />
      </I18nProvider>,
    );

    expect(screen.getAllByText('指挥中心').length).toBeGreaterThan(0);
    expect(screen.getAllByText('流量实验室').length).toBeGreaterThan(0);
    expect(screen.getByText('控制平面')).toBeInTheDocument();
  });
});
