import { useEffect, useState, type FormEvent } from 'react';
import { Navigate } from 'react-router-dom';
import { Activity, Eye, EyeOff, ShieldCheck } from 'lucide-react';
import { useI18n } from '../i18n';
import { useAuthStore } from '../stores/authStore';

export function LoginPage() {
  const { t } = useI18n();
  const [username, setUsername] = useState('admin');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState('');
  const initialized = useAuthStore((state) => state.initialized);
  const isLoading = useAuthStore((state) => state.isLoading);
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  const initialize = useAuthStore((state) => state.initialize);
  const login = useAuthStore((state) => state.login);

  useEffect(() => {
    if (!initialized) {
      void initialize();
    }
  }, [initialize, initialized]);

  if (initialized && isAuthenticated) {
    return <Navigate to="/command-center" replace />;
  }

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setError('');
    if (!username.trim() || !password.trim()) {
      setError(t('login.error.required'));
      return;
    }

    try {
      await login(username.trim(), password);
    } catch {
      setError(t('login.error.invalid'));
    }
  };

  return (
    <div className="auth-screen">
      <div className="auth-card">
        <div className="auth-brand">
          <div className="brand__mark">P</div>
          <div>
            <strong>{t('common.appName')}</strong>
            <p>{t('common.controlPlane')}</p>
          </div>
        </div>

        <div className="auth-hero">
          <p className="workspace-eyebrow">{t('login.eyebrow')}</p>
          <h1>{t('login.title')}</h1>
          <p className="auth-copy">{t('login.summary')}</p>
          <div className="auth-meta">
            <span><ShieldCheck size={16} /> {t('login.meta.localhost')}</span>
            <span><Activity size={16} /> {t('login.meta.runtime')}</span>
          </div>
        </div>

        <form className="auth-form" onSubmit={handleSubmit}>
          {error ? <div className="auth-error">{error}</div> : null}

          <label className="auth-field">
            <span>{t('login.username')}</span>
            <input
              name="username"
              type="text"
              value={username}
              onChange={(event) => setUsername(event.target.value)}
              autoComplete="username"
              autoFocus
            />
          </label>

          <label className="auth-field">
            <span>{t('login.password')}</span>
            <div className="auth-password">
              <input
                name="password"
                type={showPassword ? 'text' : 'password'}
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                autoComplete="current-password"
              />
              <button
                type="button"
                className="button button--ghost auth-password__toggle"
                onClick={() => setShowPassword((current) => !current)}
              >
                {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
              </button>
            </div>
          </label>

          <button type="submit" className="button button--primary auth-submit" disabled={isLoading}>
            {isLoading ? t('login.submitting') : t('login.submit')}
          </button>
        </form>
      </div>
    </div>
  );
}
