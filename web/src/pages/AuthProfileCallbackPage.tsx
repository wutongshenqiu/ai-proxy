import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { LoaderCircle, ShieldCheck, ShieldX } from 'lucide-react';
import { authProfilesApi } from '../services/authProfiles';
import { getApiErrorMessage } from '../services/errors';

const completionRequests = new Map<string, Promise<unknown>>();

export function AuthProfileCallbackPage() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const oauthState = searchParams.get('state');
  const code = searchParams.get('code');
  const callbackErrorMessage = searchParams.get('error')
    ? `OAuth provider returned an error: ${searchParams.get('error')}`
    : !oauthState || !code
      ? 'Missing OAuth state or code.'
      : null;
  const [status, setStatus] = useState<'loading' | 'success' | 'error'>('loading');
  const [message, setMessage] = useState('Completing OAuth login…');

  useEffect(() => {
    if (callbackErrorMessage || !oauthState || !code) {
      return;
    }

    const operationKey = `${oauthState}:${code}`;
    let completion = completionRequests.get(operationKey);
    if (!completion) {
      completion = authProfilesApi.completeCodexOauth(oauthState, code);
      completionRequests.set(operationKey, completion);
    }

    let cancelled = false;
    void completion
      .then(() => {
        if (cancelled) {
          return;
        }
        setStatus('success');
        setMessage('OAuth login completed. Redirecting back to Provider Atlas…');
        window.setTimeout(() => {
          navigate('/provider-atlas', { replace: true });
        }, 1200);
      })
      .catch((loadError: unknown) => {
        if (cancelled) {
          return;
        }
        setStatus('error');
        setMessage(getApiErrorMessage(loadError, 'Failed to complete OAuth login.'));
      })
      .finally(() => {
        completionRequests.delete(operationKey);
      });

    return () => {
      cancelled = true;
    };
  }, [callbackErrorMessage, code, navigate, oauthState]);

  const effectiveStatus = callbackErrorMessage ? 'error' : status;
  const effectiveMessage = callbackErrorMessage ?? message;

  return (
    <div className="auth-screen">
      <div className="auth-card">
        <div className="auth-hero">
          <p className="workspace-eyebrow">PRISM / AUTH CALLBACK</p>
          <h1>Managed auth callback</h1>
          <p className="auth-copy">{effectiveMessage}</p>
        </div>
        <div className="auth-meta" style={{ justifyContent: 'center' }}>
          {effectiveStatus === 'loading' ? <LoaderCircle size={20} className="spinning" /> : null}
          {effectiveStatus === 'success' ? <ShieldCheck size={20} /> : null}
          {effectiveStatus === 'error' ? <ShieldX size={20} /> : null}
        </div>
        {effectiveStatus === 'error' ? (
          <button type="button" className="button button--primary auth-submit" onClick={() => navigate('/provider-atlas')}>
            Back to Provider Atlas
          </button>
        ) : null}
      </div>
    </div>
  );
}
