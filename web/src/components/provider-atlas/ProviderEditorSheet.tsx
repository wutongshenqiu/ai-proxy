import { WorkbenchSheet } from '../WorkbenchSheet';
import { profileKey } from '../../lib/authProfileDraft';
import type {
  AuthProfilesRuntimeResponse,
  PresentationPreviewResponse,
  ProviderCapabilityEntry,
  ProviderDetail,
  ProviderHealthResult,
} from '../../types/backend';
import type { ProviderEditorFormState } from './types';

interface ProviderEditorSheetProps {
  open: boolean;
  loadingDetail: boolean;
  actionStatus: string | null;
  actionError: string | null;
  detail: ProviderDetail | null;
  runtimeInfo: AuthProfilesRuntimeResponse | null;
  health: ProviderHealthResult | null;
  preview: PresentationPreviewResponse | null;
  previewing: boolean;
  saving: boolean;
  selectedCapabilities: ProviderCapabilityEntry | null;
  formState: ProviderEditorFormState;
  refreshingProfileId: string | null;
  onClose: () => void;
  onRunHealthCheck: () => void;
  onRunPresentationPreview: () => void;
  onSaveProvider: () => void;
  onFormStateChange: (patch: Partial<ProviderEditorFormState>) => void;
  onRefreshAuthProfile: (provider: string, profileId: string) => void;
}

export function ProviderEditorSheet({
  open,
  loadingDetail,
  actionStatus,
  actionError,
  detail,
  runtimeInfo,
  health,
  preview,
  previewing,
  saving,
  selectedCapabilities,
  formState,
  refreshingProfileId,
  onClose,
  onRunHealthCheck,
  onRunPresentationPreview,
  onSaveProvider,
  onFormStateChange,
  onRefreshAuthProfile,
}: ProviderEditorSheetProps) {
  return (
    <WorkbenchSheet
      open={open}
      onClose={onClose}
      title="Provider editor"
      subtitle="Edit runtime-facing provider fields, run a real upstream health probe, and preview presentation mutations."
      actions={(
        <>
          <button type="button" className="button button--ghost" onClick={onRunHealthCheck}>
            Run health probe
          </button>
          <button type="button" className="button button--ghost" onClick={onRunPresentationPreview} disabled={previewing}>
            {previewing ? 'Previewing…' : 'Presentation preview'}
          </button>
          <button type="button" className="button button--primary" onClick={onSaveProvider} disabled={saving}>
            {saving ? 'Saving…' : 'Save provider'}
          </button>
        </>
      )}
    >
      {loadingDetail ? <div className="status-message">Loading provider detail…</div> : null}
      {actionStatus ? <div className="status-message status-message--success">{actionStatus}</div> : null}
      {actionError ? <div className="status-message status-message--danger">{actionError}</div> : null}

      {detail ? (
        <>
          <section className="sheet-section">
            <h3>Provider posture</h3>
            <div className="detail-grid">
              <div className="detail-grid__row"><span>Name</span><strong>{detail.name}</strong></div>
              <div className="detail-grid__row"><span>Format</span><strong>{detail.format}</strong></div>
              <div className="detail-grid__row"><span>Upstream</span><strong>{detail.upstream}</strong></div>
              <div className="detail-grid__row"><span>Auth profiles</span><strong>{detail.auth_profiles.length}</strong></div>
            </div>
          </section>

          <section className="sheet-section">
            <h3>Editable runtime fields</h3>
            <div className="sheet-form">
              <label className="sheet-field">
                <span>Base URL</span>
                <input
                  name="provider-base-url"
                  type="url"
                  autoComplete="url"
                  value={formState.baseUrl}
                  onChange={(event) => onFormStateChange({ baseUrl: event.target.value })}
                />
              </label>
              <label className="sheet-field">
                <span>Region</span>
                <input
                  name="provider-region"
                  autoComplete="off"
                  value={formState.region}
                  onChange={(event) => onFormStateChange({ region: event.target.value })}
                />
              </label>
              <label className="sheet-field">
                <span>Weight</span>
                <input
                  name="provider-weight"
                  inputMode="numeric"
                  autoComplete="off"
                  value={formState.weight}
                  onChange={(event) => onFormStateChange({ weight: event.target.value })}
                />
              </label>
              <label className="detail-grid__row">
                <span>Disabled</span>
                <input
                  type="checkbox"
                  checked={formState.disabled}
                  onChange={(event) => onFormStateChange({ disabled: event.target.checked })}
                />
              </label>
            </div>
          </section>

          <section className="sheet-section">
            <h3>Auth profiles</h3>
            <div className="probe-list">
              {detail.auth_profiles.length === 0 ? (
                <div className="probe-check">
                  <span>Profiles</span>
                  <strong>None configured</strong>
                </div>
              ) : (
                detail.auth_profiles.map((profile) => (
                  <div key={profile.qualified_name} className="probe-check">
                    <span>{profile.qualified_name}</span>
                    <strong>{profile.mode}</strong>
                    {profile.refresh_token_present ? (
                      <button
                        type="button"
                        className="button button--ghost"
                        onClick={() => onRefreshAuthProfile(detail.name, profile.id)}
                        disabled={refreshingProfileId === profileKey(detail.name, profile.id)}
                      >
                        {refreshingProfileId === profileKey(detail.name, profile.id) ? 'Refreshing…' : 'Refresh'}
                      </button>
                    ) : null}
                  </div>
                ))
              )}
            </div>
          </section>

          {runtimeInfo ? (
            <section className="sheet-section">
              <h3>Managed auth runtime</h3>
              <div className="detail-grid">
                <div className="detail-grid__row"><span>Storage dir</span><strong>{runtimeInfo.storage_dir ?? 'not configured'}</strong></div>
                <div className="detail-grid__row"><span>Codex auth file</span><strong>{runtimeInfo.codex_auth_file ?? 'not configured'}</strong></div>
                <div className="detail-grid__row"><span>Proxy URL</span><strong>{runtimeInfo.proxy_url ?? 'none'}</strong></div>
              </div>
            </section>
          ) : null}

          {selectedCapabilities ? (
            <section className="sheet-section">
              <h3>Capability snapshot</h3>
              <div className="detail-grid">
                <div className="detail-grid__row"><span>Probe status</span><strong>{selectedCapabilities.probe_status}</strong></div>
                <div className="detail-grid__row"><span>Presentation</span><strong>{selectedCapabilities.presentation_profile}</strong></div>
                <div className="detail-grid__row"><span>Models</span><strong>{selectedCapabilities.models.length}</strong></div>
                <div className="detail-grid__row"><span>Wire API</span><strong>{selectedCapabilities.wire_api}</strong></div>
              </div>
            </section>
          ) : null}

          {health ? (
            <section className="sheet-section">
              <h3>Health probe</h3>
              <div className="detail-grid">
                <div className="detail-grid__row"><span>Status</span><strong>{health.status}</strong></div>
                <div className="detail-grid__row"><span>Checked at</span><strong>{health.checked_at}</strong></div>
                <div className="detail-grid__row"><span>Latency</span><strong>{health.latency_ms} ms</strong></div>
              </div>
              <div className="probe-list">
                {health.checks.map((check) => (
                  <div key={check.capability} className="probe-check">
                    <span>{check.capability}</span>
                    <strong>{check.status}</strong>
                  </div>
                ))}
              </div>
            </section>
          ) : null}

          {preview ? (
            <section className="sheet-section">
              <h3>Presentation preview</h3>
              <div className="detail-grid">
                <div className="detail-grid__row"><span>Profile</span><strong>{preview.profile}</strong></div>
                <div className="detail-grid__row"><span>Activated</span><strong>{preview.activated ? 'yes' : 'no'}</strong></div>
                <div className="detail-grid__row"><span>Protected headers blocked</span><strong>{preview.protected_headers_blocked.length}</strong></div>
                <div className="detail-grid__row"><span>Mutations</span><strong>{preview.body_mutations.length}</strong></div>
              </div>
              <div className="probe-list">
                {preview.body_mutations.map((mutation) => (
                  <div key={`${mutation.kind}-${mutation.reason ?? 'none'}`} className="probe-check">
                    <span>{mutation.kind}</span>
                    <strong>{mutation.applied ? 'applied' : mutation.reason ?? 'skipped'}</strong>
                  </div>
                ))}
              </div>
            </section>
          ) : null}
        </>
      ) : null}
    </WorkbenchSheet>
  );
}
