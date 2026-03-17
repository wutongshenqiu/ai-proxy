import { WorkbenchSheet } from '../WorkbenchSheet';
import type { ProviderAtlasRow } from '../../types/controlPlane';
import type { ProviderRegistryFormState } from './types';

interface ProviderRegistrySheetProps {
  open: boolean;
  registryStatus: string | null;
  registryError: string | null;
  registryLoading: boolean;
  registryForm: ProviderRegistryFormState;
  selectedProvider: string | null;
  selectedRow: ProviderAtlasRow | null;
  selectedProbeStatus: string | null;
  onClose: () => void;
  onRegistryFormChange: (patch: Partial<ProviderRegistryFormState>) => void;
  onFetchModels: () => void;
  onDeleteSelectedProvider: () => void;
  onCreateProvider: () => void;
}

export function ProviderRegistrySheet({
  open,
  registryStatus,
  registryError,
  registryLoading,
  registryForm,
  selectedProvider,
  selectedRow,
  selectedProbeStatus,
  onClose,
  onRegistryFormChange,
  onFetchModels,
  onDeleteSelectedProvider,
  onCreateProvider,
}: ProviderRegistrySheetProps) {
  return (
    <WorkbenchSheet
      open={open}
      onClose={onClose}
      title="Provider registry workbench"
      subtitle="Create disabled providers, fetch model inventories, and remove obsolete runtime entities without leaving the atlas."
      actions={(
        <>
          <button type="button" className="button button--ghost" onClick={onFetchModels} disabled={registryLoading}>
            {registryLoading ? 'Working…' : 'Fetch models'}
          </button>
          <button type="button" className="button button--ghost" onClick={onDeleteSelectedProvider} disabled={registryLoading || !selectedProvider}>
            Delete selected
          </button>
          <button type="button" className="button button--primary" onClick={onCreateProvider} disabled={registryLoading}>
            {registryLoading ? 'Saving…' : 'Create provider'}
          </button>
        </>
      )}
    >
      {registryStatus ? <div className="status-message status-message--success">{registryStatus}</div> : null}
      {registryError ? <div className="status-message status-message--danger">{registryError}</div> : null}

      <section className="sheet-section">
        <h3>New provider draft</h3>
        <form
          className="sheet-form"
          onSubmit={(event) => {
            event.preventDefault();
            onCreateProvider();
          }}
        >
          <label className="sheet-field">
            <span>Name</span>
            <input
              name="provider-name"
              autoComplete="organization"
              value={registryForm.name}
              onChange={(event) => onRegistryFormChange({ name: event.target.value })}
            />
          </label>
          <label className="sheet-field">
            <span>Format</span>
            <select value={registryForm.format} onChange={(event) => onRegistryFormChange({ format: event.target.value as ProviderRegistryFormState['format'] })}>
              <option value="openai">openai</option>
              <option value="claude">claude</option>
              <option value="gemini">gemini</option>
            </select>
          </label>
          <label className="sheet-field">
            <span>Upstream</span>
            <input
              name="provider-upstream"
              autoComplete="off"
              value={registryForm.upstream}
              onChange={(event) => onRegistryFormChange({ upstream: event.target.value })}
            />
          </label>
          <label className="sheet-field">
            <span>API key</span>
            <input
              name="provider-api-key"
              type="password"
              autoComplete="new-password"
              value={registryForm.apiKey}
              onChange={(event) => onRegistryFormChange({ apiKey: event.target.value })}
            />
          </label>
          <label className="sheet-field">
            <span>Base URL</span>
            <input
              name="registry-base-url"
              type="url"
              autoComplete="url"
              value={registryForm.baseUrl}
              onChange={(event) => onRegistryFormChange({ baseUrl: event.target.value })}
            />
          </label>
          <label className="sheet-field">
            <span>Models</span>
            <input
              name="provider-models"
              autoComplete="off"
              value={registryForm.models}
              onChange={(event) => onRegistryFormChange({ models: event.target.value })}
            />
          </label>
          <label className="detail-grid__row">
            <span>Disabled</span>
            <input
              type="checkbox"
              checked={registryForm.disabled}
              onChange={(event) => onRegistryFormChange({ disabled: event.target.checked })}
            />
          </label>
        </form>
      </section>

      <section className="sheet-section">
        <h3>Selected provider</h3>
        <div className="detail-grid">
          <div className="detail-grid__row"><span>Name</span><strong>{selectedProvider ?? 'none selected'}</strong></div>
          <div className="detail-grid__row"><span>Status</span><strong>{selectedRow?.status ?? 'n/a'}</strong></div>
          <div className="detail-grid__row"><span>Auth posture</span><strong>{selectedRow?.auth ?? 'n/a'}</strong></div>
          <div className="detail-grid__row"><span>Coverage</span><strong>{selectedProbeStatus ?? 'n/a'}</strong></div>
        </div>
      </section>
    </WorkbenchSheet>
  );
}
