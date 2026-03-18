import { WorkbenchSheet } from '../WorkbenchSheet';
import { profileKey } from '../../lib/authProfileDraft';
import { useI18n } from '../../i18n';
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
  const { t, formatDateTime, formatDurationMs, formatNumber } = useI18n();

  return (
    <WorkbenchSheet
      open={open}
      onClose={onClose}
      title={t('providerAtlas.editor.title')}
      subtitle={t('providerAtlas.editor.subtitle')}
      actions={(
        <>
          <button type="button" className="button button--ghost" onClick={onRunHealthCheck}>
            {t('providerAtlas.editor.runHealthProbe')}
          </button>
          <button type="button" className="button button--ghost" onClick={onRunPresentationPreview} disabled={previewing}>
            {previewing ? t('providerAtlas.editor.previewing') : t('providerAtlas.editor.presentationPreview')}
          </button>
          <button type="button" className="button button--primary" onClick={onSaveProvider} disabled={saving}>
            {saving ? t('providerAtlas.editor.saving') : t('providerAtlas.editor.saveProvider')}
          </button>
        </>
      )}
    >
      {loadingDetail ? <div className="status-message">{t('providerAtlas.editor.loadingDetail')}</div> : null}
      {actionStatus ? <div className="status-message status-message--success">{actionStatus}</div> : null}
      {actionError ? <div className="status-message status-message--danger">{actionError}</div> : null}

      {detail ? (
        <>
          <section className="sheet-section">
            <h3>{t('providerAtlas.editor.providerPosture')}</h3>
            <div className="detail-grid">
              <div className="detail-grid__row"><span>{t('common.name')}</span><strong>{detail.name}</strong></div>
              <div className="detail-grid__row"><span>{t('common.format')}</span><strong>{detail.format}</strong></div>
              <div className="detail-grid__row"><span>{t('common.upstream')}</span><strong>{detail.upstream}</strong></div>
              <div className="detail-grid__row"><span>{t('providerAtlas.editor.authProfiles')}</span><strong>{formatNumber(detail.auth_profiles.length)}</strong></div>
            </div>
          </section>

          <section className="sheet-section">
            <h3>{t('providerAtlas.editor.editableRuntimeFields')}</h3>
            <div className="sheet-form">
              <label className="sheet-field">
                <span>{t('providerAtlas.editor.baseUrl')}</span>
                <input
                  name="provider-base-url"
                  type="url"
                  autoComplete="url"
                  value={formState.baseUrl}
                  onChange={(event) => onFormStateChange({ baseUrl: event.target.value })}
                />
              </label>
              <label className="sheet-field">
                <span>{t('common.region')}</span>
                <input
                  name="provider-region"
                  autoComplete="off"
                  value={formState.region}
                  onChange={(event) => onFormStateChange({ region: event.target.value })}
                />
              </label>
              <label className="sheet-field">
                <span>{t('common.weight')}</span>
                <input
                  name="provider-weight"
                  inputMode="numeric"
                  autoComplete="off"
                  value={formState.weight}
                  onChange={(event) => onFormStateChange({ weight: event.target.value })}
                />
              </label>
              <label className="detail-grid__row">
                <span>{t('common.disabled')}</span>
                <input
                  type="checkbox"
                  checked={formState.disabled}
                  onChange={(event) => onFormStateChange({ disabled: event.target.checked })}
                />
              </label>
            </div>
          </section>

          <section className="sheet-section">
            <h3>{t('providerAtlas.editor.authProfiles')}</h3>
            <div className="probe-list">
              {detail.auth_profiles.length === 0 ? (
                <div className="probe-check">
                  <span>{t('providerAtlas.editor.authProfiles')}</span>
                  <strong>{t('providerAtlas.editor.noneConfigured')}</strong>
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
                        {refreshingProfileId === profileKey(detail.name, profile.id)
                          ? t('providerAtlas.editor.refreshing')
                          : t('common.refresh')}
                      </button>
                    ) : null}
                  </div>
                ))
              )}
            </div>
          </section>

          {runtimeInfo ? (
            <section className="sheet-section">
              <h3>{t('providerAtlas.editor.managedAuthRuntime')}</h3>
              <div className="detail-grid">
                <div className="detail-grid__row"><span>{t('providerAtlas.editor.storageDir')}</span><strong>{runtimeInfo.storage_dir ?? t('common.notConfigured')}</strong></div>
                <div className="detail-grid__row"><span>{t('providerAtlas.editor.codexAuthFile')}</span><strong>{runtimeInfo.codex_auth_file ?? t('common.notConfigured')}</strong></div>
                <div className="detail-grid__row"><span>{t('providerAtlas.editor.proxyUrl')}</span><strong>{runtimeInfo.proxy_url ?? t('common.none')}</strong></div>
              </div>
            </section>
          ) : null}

          {selectedCapabilities ? (
            <section className="sheet-section">
              <h3>{t('providerAtlas.editor.capabilitySnapshot')}</h3>
              <div className="detail-grid">
                <div className="detail-grid__row"><span>{t('providerAtlas.coverage.probeStatus')}</span><strong>{selectedCapabilities.probe_status}</strong></div>
                <div className="detail-grid__row"><span>{t('providerAtlas.coverage.presentation')}</span><strong>{selectedCapabilities.presentation_profile}</strong></div>
                <div className="detail-grid__row"><span>{t('common.models')}</span><strong>{formatNumber(selectedCapabilities.models.length)}</strong></div>
                <div className="detail-grid__row"><span>{t('providerAtlas.editor.wireApi')}</span><strong>{selectedCapabilities.wire_api}</strong></div>
              </div>
            </section>
          ) : null}

          {health ? (
            <section className="sheet-section">
              <h3>{t('providerAtlas.editor.healthProbe')}</h3>
              <div className="detail-grid">
                <div className="detail-grid__row"><span>{t('common.status')}</span><strong>{health.status}</strong></div>
                <div className="detail-grid__row"><span>{t('providerAtlas.editor.checkedAt')}</span><strong>{formatDateTime(health.checked_at)}</strong></div>
                <div className="detail-grid__row"><span>{t('common.latency')}</span><strong>{formatDurationMs(health.latency_ms)}</strong></div>
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
              <h3>{t('providerAtlas.editor.presentationPreview')}</h3>
              <div className="detail-grid">
                <div className="detail-grid__row"><span>{t('common.profile')}</span><strong>{preview.profile}</strong></div>
                <div className="detail-grid__row"><span>{t('providerAtlas.editor.activated')}</span><strong>{preview.activated ? t('common.yes') : t('common.no')}</strong></div>
                <div className="detail-grid__row"><span>{t('providerAtlas.editor.protectedHeadersBlocked')}</span><strong>{formatNumber(preview.protected_headers_blocked.length)}</strong></div>
                <div className="detail-grid__row"><span>{t('providerAtlas.editor.mutations')}</span><strong>{formatNumber(preview.body_mutations.length)}</strong></div>
              </div>
              <div className="probe-list">
                {preview.body_mutations.map((mutation) => (
                  <div key={`${mutation.kind}-${mutation.reason ?? 'none'}`} className="probe-check">
                    <span>{mutation.kind}</span>
                    <strong>{mutation.applied ? t('providerAtlas.editor.applied') : mutation.reason ?? t('providerAtlas.editor.skipped')}</strong>
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
