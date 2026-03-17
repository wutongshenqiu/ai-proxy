import { useEffect, useMemo, useState } from 'react';
import { Panel } from '../components/Panel';
import { StatusPill } from '../components/StatusPill';
import { WorkbenchSheet } from '../components/WorkbenchSheet';
import { useChangeStudioData } from '../hooks/useWorkspaceData';
import { clearRouteDraft, readRouteDraft } from '../lib/routeDraft';
import { authKeysApi } from '../services/authKeys';
import { configApi } from '../services/config';
import { getApiErrorMessage } from '../services/errors';
import { tenantsApi } from '../services/tenants';
import type { RouteDraft } from '../lib/routeDraft';
import type {
  AuthKeyCreateRequest,
  AuthKeySummary,
  AuthKeyUpdateRequest,
  BudgetConfig,
  ConfigApplyResponse,
  ConfigValidateResponse,
  KeyRateLimitConfig,
  TenantMetricsResponse,
  TenantSummary,
} from '../types/backend';

interface AccessPolicyFormState {
  name: string;
  tenantId: string;
  allowedModels: string;
  allowedCredentials: string;
  rpm: string;
  tpm: string;
  costPerDayUsd: string;
  budgetEnabled: boolean;
  budgetTotalUsd: string;
  budgetPeriod: 'daily' | 'monthly';
  expiresAt: string;
}

const emptyAccessForm: AccessPolicyFormState = {
  name: 'e2e-temp-key',
  tenantId: '',
  allowedModels: '',
  allowedCredentials: '',
  rpm: '',
  tpm: '',
  costPerDayUsd: '',
  budgetEnabled: false,
  budgetTotalUsd: '',
  budgetPeriod: 'daily',
  expiresAt: '',
};

function parseListField(value: string) {
  return value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
}

function formFromAuthKey(key: AuthKeySummary): AccessPolicyFormState {
  return {
    name: key.name ?? '',
    tenantId: key.tenant_id ?? '',
    allowedModels: key.allowed_models.join(', '),
    allowedCredentials: key.allowed_credentials.join(', '),
    rpm: key.rate_limit?.rpm?.toString() ?? '',
    tpm: key.rate_limit?.tpm?.toString() ?? '',
    costPerDayUsd: key.rate_limit?.cost_per_day_usd?.toString() ?? '',
    budgetEnabled: key.budget != null,
    budgetTotalUsd: key.budget?.total_usd?.toString() ?? '',
    budgetPeriod: key.budget?.period ?? 'daily',
    expiresAt: key.expires_at ? key.expires_at.slice(0, 16) : '',
  };
}

function buildRateLimit(form: AccessPolicyFormState): KeyRateLimitConfig | undefined {
  const rpm = form.rpm ? Number(form.rpm) : undefined;
  const tpm = form.tpm ? Number(form.tpm) : undefined;
  const cost = form.costPerDayUsd ? Number(form.costPerDayUsd) : undefined;
  if (rpm === undefined && tpm === undefined && cost === undefined) {
    return undefined;
  }
  return {
    rpm,
    tpm,
    cost_per_day_usd: cost,
  };
}

function buildBudget(form: AccessPolicyFormState): BudgetConfig | undefined {
  if (!form.budgetEnabled || !form.budgetTotalUsd) {
    return undefined;
  }
  return {
    total_usd: Number(form.budgetTotalUsd),
    period: form.budgetPeriod,
  };
}

export function ChangeStudioPage() {
  const { data, error, loading, reload } = useChangeStudioData();
  const [selectedFamily, setSelectedFamily] = useState<string | null>(null);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editorMode, setEditorMode] = useState<'structured' | 'yaml'>('structured');
  const [yaml, setYaml] = useState('');
  const [configVersion, setConfigVersion] = useState<string | undefined>();
  const [configPath, setConfigPath] = useState('');
  const [routeDraft, setRouteDraft] = useState<RouteDraft | null>(null);
  const [loadingEditor, setLoadingEditor] = useState(false);
  const [validating, setValidating] = useState(false);
  const [applying, setApplying] = useState(false);
  const [reloading, setReloading] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [actionStatus, setActionStatus] = useState<string | null>(null);
  const [validationResult, setValidationResult] = useState<ConfigValidateResponse | null>(null);
  const [applyResult, setApplyResult] = useState<ConfigApplyResponse | null>(null);
  const [authKeys, setAuthKeys] = useState<AuthKeySummary[]>([]);
  const [tenants, setTenants] = useState<TenantSummary[]>([]);
  const [selectedTenantId, setSelectedTenantId] = useState<string | null>(null);
  const [tenantMetrics, setTenantMetrics] = useState<TenantMetricsResponse | null>(null);
  const [tenantLoading, setTenantLoading] = useState(false);
  const [tenantError, setTenantError] = useState<string | null>(null);
  const [refreshingAccess, setRefreshingAccess] = useState(false);
  const [selectedAuthKeyId, setSelectedAuthKeyId] = useState<number | null>(null);
  const [accessOpen, setAccessOpen] = useState(false);
  const [accessEditorMode, setAccessEditorMode] = useState<'create' | 'edit'>('create');
  const [accessForm, setAccessForm] = useState<AccessPolicyFormState>(emptyAccessForm);
  const [accessStatus, setAccessStatus] = useState<string | null>(null);
  const [accessError, setAccessError] = useState<string | null>(null);
  const [revealedKey, setRevealedKey] = useState<string | null>(null);
  const [savingKey, setSavingKey] = useState(false);
  const [revealingKey, setRevealingKey] = useState(false);
  const [deletingKey, setDeletingKey] = useState(false);

  useEffect(() => {
    setSelectedFamily((current) => current ?? data?.registry[0]?.family ?? null);
  }, [data]);

  const selectedRegistry = useMemo(
    () => data?.registry.find((item) => item.family === selectedFamily) ?? null,
    [data, selectedFamily],
  );
  const selectedAuthKey = useMemo(
    () => authKeys.find((item) => item.id === selectedAuthKeyId) ?? null,
    [authKeys, selectedAuthKeyId],
  );

  useEffect(() => {
    if (!selectedAuthKey) {
      return;
    }
    setAccessEditorMode('edit');
    setAccessForm(formFromAuthKey(selectedAuthKey));
  }, [selectedAuthKey]);

  const loadTenantMetrics = async (tenantId: string) => {
    setTenantLoading(true);
    setTenantError(null);
    try {
      const response = await tenantsApi.metrics(tenantId);
      setTenantMetrics(response);
      setSelectedTenantId(tenantId);
    } catch (loadError) {
      setTenantError(getApiErrorMessage(loadError, 'Failed to load tenant metrics'));
    } finally {
      setTenantLoading(false);
    }
  };

  const loadAccessData = async () => {
    const [keysResponse, tenantsResponse] = await Promise.all([
      authKeysApi.list(),
      tenantsApi.list(),
    ]);
    setAuthKeys(keysResponse.auth_keys);
    setTenants(tenantsResponse.tenants);
    setSelectedAuthKeyId((current) => current ?? keysResponse.auth_keys[0]?.id ?? null);
    setSelectedTenantId((current) => current ?? tenantsResponse.tenants[0]?.id ?? null);
  };

  const loadEditor = async (mode: 'structured' | 'yaml') => {
    setEditorMode(mode);
    setEditorOpen(true);
    setLoadingEditor(true);
    setActionError(null);
    setActionStatus(null);
    setValidationResult(null);
    setApplyResult(null);

    try {
      const [rawConfig] = await Promise.all([configApi.raw()]);
      setYaml(rawConfig.content);
      setConfigVersion(rawConfig.config_version);
      setConfigPath(rawConfig.path);
      setRouteDraft(readRouteDraft());
    } catch (loadError) {
      setActionError(getApiErrorMessage(loadError, 'Failed to load configuration draft'));
    } finally {
      setLoadingEditor(false);
    }
  };

  useEffect(() => {
    void loadAccessData().catch(() => {
      setAuthKeys([]);
      setTenants([]);
    });
  }, []);

  const refreshAccessPosture = async () => {
    setRefreshingAccess(true);
    setAccessError(null);
    setTenantError(null);
    try {
      await loadAccessData();
      if (selectedTenantId) {
        await loadTenantMetrics(selectedTenantId);
      }
    } catch (refreshError) {
      setAccessError(getApiErrorMessage(refreshError, 'Failed to refresh access posture'));
    } finally {
      setRefreshingAccess(false);
    }
  };

  const validateDraft = async () => {
    setValidating(true);
    setActionError(null);
    setActionStatus(null);
    try {
      const result = await configApi.validate(yaml);
      setValidationResult(result);
      setActionStatus(result.valid ? 'Validation passed.' : 'Validation returned issues.');
    } catch (validationError) {
      setActionError(getApiErrorMessage(validationError, 'Validation failed'));
    } finally {
      setValidating(false);
    }
  };

  const applyDraft = async () => {
    setApplying(true);
    setActionError(null);
    setActionStatus(null);
    try {
      const result = await configApi.apply(yaml, configVersion);
      setApplyResult(result);
      setConfigVersion(result.config_version);
      setActionStatus(result.message);
      await reload();
    } catch (applyError) {
      setActionError(getApiErrorMessage(applyError, 'Apply failed'));
    } finally {
      setApplying(false);
    }
  };

  const reloadRuntime = async () => {
    setReloading(true);
    setActionError(null);
    setActionStatus(null);
    try {
      const result = await configApi.reload();
      setActionStatus(result.message);
      await reload();
    } catch (reloadError) {
      setActionError(getApiErrorMessage(reloadError, 'Runtime reload failed'));
    } finally {
      setReloading(false);
    }
  };

  const openAccessWorkbench = async () => {
    setAccessOpen(true);
    setAccessError(null);
    setAccessStatus(null);
    setRevealedKey(null);
    setAccessEditorMode('create');
    setAccessForm(emptyAccessForm);
    try {
      await loadAccessData();
    } catch (loadError) {
      setAccessError(getApiErrorMessage(loadError, 'Failed to load access controls'));
    }
  };

  const startNewAccessDraft = () => {
    setAccessEditorMode('create');
    setAccessError(null);
    setAccessStatus(null);
    setRevealedKey(null);
    setAccessForm(emptyAccessForm);
  };

  const saveAuthKey = async () => {
    setSavingKey(true);
    setAccessError(null);
    setAccessStatus(null);
    setRevealedKey(null);
    try {
      const body = {
        name: accessForm.name.trim() || undefined,
        tenant_id: accessForm.tenantId.trim() || undefined,
        allowed_models: parseListField(accessForm.allowedModels),
        allowed_credentials: parseListField(accessForm.allowedCredentials),
        rate_limit: buildRateLimit(accessForm),
        budget: buildBudget(accessForm),
        expires_at: accessForm.expiresAt ? new Date(accessForm.expiresAt).toISOString() : undefined,
      } satisfies AuthKeyCreateRequest;

      if (accessEditorMode === 'edit' && selectedAuthKeyId !== null) {
        const update: AuthKeyUpdateRequest = {
          name: body.name,
          tenant_id: accessForm.tenantId.trim() ? accessForm.tenantId.trim() : null,
          allowed_models: body.allowed_models,
          allowed_credentials: body.allowed_credentials,
          rate_limit: body.rate_limit ?? null,
          budget: body.budget ?? null,
          expires_at: body.expires_at ?? null,
        };
        await authKeysApi.update(selectedAuthKeyId, update);
        setAccessStatus(`Saved auth key ${accessForm.name || selectedAuthKeyId}.`);
      } else {
        const response = await authKeysApi.create(body);
        setAccessStatus(response.message);
        setRevealedKey(response.key);
      }
      await loadAccessData();
      const latestKeys = await authKeysApi.list();
      const matchingKey = latestKeys.auth_keys.find((item) => item.name === (body.name ?? null));
      if (matchingKey) {
        setSelectedAuthKeyId(matchingKey.id);
      }
      if (accessForm.tenantId.trim()) {
        await loadTenantMetrics(accessForm.tenantId.trim());
      }
    } catch (createError) {
      setAccessError(getApiErrorMessage(createError, 'Failed to save auth key'));
    } finally {
      setSavingKey(false);
    }
  };

  const revealAuthKey = async () => {
    if (selectedAuthKeyId === null) {
      setAccessError('Select an auth key first.');
      return;
    }

    setRevealingKey(true);
    setAccessError(null);
    setAccessStatus(null);
    try {
      const response = await authKeysApi.reveal(selectedAuthKeyId);
      setRevealedKey(response.key);
      setAccessStatus(`Revealed auth key ${selectedAuthKey?.name ?? selectedAuthKeyId}.`);
    } catch (revealError) {
      setAccessError(getApiErrorMessage(revealError, 'Failed to reveal auth key'));
    } finally {
      setRevealingKey(false);
    }
  };

  const deleteAuthKey = async () => {
    if (selectedAuthKeyId === null) {
      setAccessError('Select an auth key first.');
      return;
    }

    setDeletingKey(true);
    setAccessError(null);
    setAccessStatus(null);
    try {
      await authKeysApi.remove(selectedAuthKeyId);
      setAccessStatus(`Deleted auth key ${selectedAuthKey?.name ?? selectedAuthKeyId}.`);
      setRevealedKey(null);
      await loadAccessData();
      setSelectedAuthKeyId(null);
    } catch (deleteError) {
      setAccessError(getApiErrorMessage(deleteError, 'Failed to delete auth key'));
    } finally {
      setDeletingKey(false);
    }
  };

  return (
    <div className="workspace-grid">
      <section className="hero">
        <div>
          <p className="workspace-eyebrow">PRISM / CHANGE STUDIO</p>
          <h1>Registry, structured edit, publish, observe</h1>
          <p className="workspace-summary">
            Change management is richer than validate and apply. The workspace is designed around object discovery, structured editing, staged publish, and watch windows.
          </p>
        </div>
        <div className="hero-actions">
          <button className="button button--primary" onClick={() => void loadEditor('structured')}>
            Create structured change
          </button>
          <button className="button button--ghost" onClick={() => void loadEditor('yaml')}>
            Open YAML escape hatch
          </button>
        </div>
      </section>

      {selectedRegistry ? (
        <div className="status-message status-message--warning">
          Active family: <strong>{selectedRegistry.family}</strong> · {selectedRegistry.record} · {selectedRegistry.dependents} dependents
        </div>
      ) : null}

      <div className="two-column">
        <Panel title="Config registry" subtitle="Object families should be browsable and impact-aware." className="panel--wide">
          <div className="table-grid table-grid--changes">
            <div className="table-grid__head">Family</div>
            <div className="table-grid__head">Record</div>
            <div className="table-grid__head">State</div>
            <div className="table-grid__head">Dependents</div>
            {loading && !data ? <div className="table-grid__cell">Loading registry…</div> : null}
            {error && !data ? <div className="table-grid__cell">{error}</div> : null}
            {(data?.registry ?? []).flatMap((item) => {
              const selected = item.family === selectedFamily;
              const cellClass = `table-grid__cell ${selected ? 'is-selected' : ''} is-clickable`;
              return [
                <div
                  key={`${item.family}-family`}
                  className={`${cellClass} table-grid__cell--strong`}
                  onClick={() => setSelectedFamily(item.family)}
                >
                  {item.family}
                </div>,
                <div key={`${item.family}-record`} className={cellClass} onClick={() => setSelectedFamily(item.family)}>
                  {item.record}
                </div>,
                <div key={`${item.family}-state`} className={cellClass} onClick={() => setSelectedFamily(item.family)}>
                  <StatusPill label={item.state} tone={item.state_tone} />
                </div>,
                <div key={`${item.family}-deps`} className={cellClass} onClick={() => setSelectedFamily(item.family)}>
                  {item.dependents}
                </div>,
              ];
            })}
          </div>
        </Panel>

        <Panel title="Transaction posture" subtitle="Current config transaction truth and delivery controls.">
          <ul className="fact-list">
            {(data?.publish_facts ?? []).map((fact) => (
              <li key={fact.label}><span>{fact.label}</span><strong>{fact.value}</strong></li>
            ))}
          </ul>
        </Panel>
      </div>

      <div className="two-column">
        <Panel title="Runtime access keys" subtitle="Gateway keys stay tied to tenants and can be created, revealed, and revoked without leaving the control plane.">
          <div className="inline-actions">
            <button type="button" className="button button--primary" onClick={() => void openAccessWorkbench()}>
              Manage access keys
            </button>
          </div>
          <div className="table-grid table-grid--keys">
            <div className="table-grid__head">Key</div>
            <div className="table-grid__head">Name</div>
            <div className="table-grid__head">Tenant</div>
            <div className="table-grid__head">Models</div>
            {authKeys.length === 0 ? <div className="table-grid__cell">No gateway auth keys configured.</div> : null}
            {authKeys.flatMap((item) => {
              const selected = item.id === selectedAuthKeyId;
              const cellClass = `table-grid__cell ${selected ? 'is-selected' : ''} is-clickable`;
              return [
                <div key={`${item.id}-key`} className={`${cellClass} table-grid__cell--strong`} onClick={() => setSelectedAuthKeyId(item.id)}>
                  {item.key_masked}
                </div>,
                <div key={`${item.id}-name`} className={cellClass} onClick={() => setSelectedAuthKeyId(item.id)}>
                  {item.name ?? 'unnamed'}
                </div>,
                <div key={`${item.id}-tenant`} className={cellClass} onClick={() => setSelectedAuthKeyId(item.id)}>
                  {item.tenant_id ?? 'global'}
                </div>,
                <div key={`${item.id}-models`} className={cellClass} onClick={() => setSelectedAuthKeyId(item.id)}>
                  {item.allowed_models.length || 'all'}
                </div>,
              ];
            })}
          </div>
        </Panel>

        <Panel title="Tenant posture" subtitle="Tenant-scoped demand and cost should stay visible next to access control work.">
          <div className="inline-actions">
            <button type="button" className="button button--ghost" onClick={() => void refreshAccessPosture()} disabled={refreshingAccess}>
              {refreshingAccess ? 'Refreshing…' : 'Refresh access posture'}
            </button>
          </div>
          {tenants.length === 0 ? (
            <div className="status-message">No tenant-scoped traffic has been recorded yet.</div>
          ) : (
            <ul className="fact-list fact-list--interactive">
              {tenants.map((tenant) => (
                <li
                  key={tenant.id}
                  className={tenant.id === selectedTenantId ? 'is-selected' : ''}
                  onClick={() => void loadTenantMetrics(tenant.id)}
                >
                  <span>{tenant.id}</span>
                  <strong>{tenant.requests} req · ${tenant.cost_usd}</strong>
                </li>
              ))}
            </ul>
          )}
          {tenantLoading ? <div className="status-message">Loading tenant metrics…</div> : null}
          {tenantError ? <div className="status-message status-message--danger">{tenantError}</div> : null}
          {tenantMetrics?.metrics ? (
            <div className="detail-grid">
              <div className="detail-grid__row"><span>Tenant</span><strong>{tenantMetrics.tenant_id}</strong></div>
              <div className="detail-grid__row"><span>Requests</span><strong>{tenantMetrics.metrics.requests}</strong></div>
              <div className="detail-grid__row"><span>Tokens</span><strong>{tenantMetrics.metrics.tokens}</strong></div>
              <div className="detail-grid__row"><span>Cost</span><strong>${tenantMetrics.metrics.cost_usd}</strong></div>
            </div>
          ) : null}
        </Panel>
      </div>

      <WorkbenchSheet
        open={editorOpen}
        onClose={() => setEditorOpen(false)}
        title={editorMode === 'structured' ? 'Structured change workbench' : 'YAML transaction workbench'}
        subtitle="Every change follows the same loop: load current truth, validate it, apply it, then observe runtime reload."
        actions={(
          <>
            <button type="button" className="button button--ghost" onClick={() => void validateDraft()} disabled={validating || loadingEditor}>
              {validating ? 'Validating…' : 'Validate'}
            </button>
            <button type="button" className="button button--ghost" onClick={() => void reloadRuntime()} disabled={reloading || loadingEditor}>
              {reloading ? 'Reloading…' : 'Reload runtime'}
            </button>
            <button type="button" className="button button--primary" onClick={() => void applyDraft()} disabled={applying || loadingEditor}>
              {applying ? 'Applying…' : 'Apply draft'}
            </button>
          </>
        )}
      >
        {loadingEditor ? <div className="status-message">Loading current config and linked drafts…</div> : null}
        {actionStatus ? <div className="status-message status-message--success">{actionStatus}</div> : null}
        {actionError ? <div className="status-message status-message--danger">{actionError}</div> : null}

        <section className="sheet-section">
          <h3>Change brief</h3>
          <div className="detail-grid">
            <div className="detail-grid__row"><span>Mode</span><strong>{editorMode === 'structured' ? 'structured' : 'yaml'}</strong></div>
            <div className="detail-grid__row"><span>Family</span><strong>{selectedRegistry?.family ?? 'none selected'}</strong></div>
            <div className="detail-grid__row"><span>Record</span><strong>{selectedRegistry?.record ?? 'n/a'}</strong></div>
            <div className="detail-grid__row"><span>Config path</span><strong>{configPath || 'loading…'}</strong></div>
            <div className="detail-grid__row"><span>Version</span><strong>{configVersion ?? 'pending'}</strong></div>
          </div>
          {editorMode === 'structured' ? (
            <div className="status-message">
              Structured mode keeps operator intent, affected family, and linked route context visible while the transaction is still applied as first-class YAML.
            </div>
          ) : null}
        </section>

        {routeDraft ? (
          <section className="sheet-section">
            <h3>Linked route draft</h3>
            <div className="detail-grid">
              <div className="detail-grid__row"><span>Scenario</span><strong>{routeDraft.scenario.scenario}</strong></div>
              <div className="detail-grid__row"><span>Winner</span><strong>{routeDraft.explanation?.selected?.provider ?? routeDraft.scenario.winner}</strong></div>
              <div className="detail-grid__row"><span>Created at</span><strong>{routeDraft.createdAt}</strong></div>
            </div>
            <div className="inline-actions">
              <button
                type="button"
                className="button button--ghost"
                onClick={() => {
                  clearRouteDraft();
                  setRouteDraft(null);
                }}
              >
                Discard linked draft
              </button>
            </div>
          </section>
        ) : null}

        <section className="sheet-section">
          <h3>Config transaction</h3>
          <textarea
            className="yaml-editor"
            value={yaml}
            onChange={(event) => setYaml(event.target.value)}
            spellCheck={false}
          />
        </section>

        {validationResult ? (
          <section className="sheet-section">
            <h3>Validation result</h3>
            <div className={`status-message ${validationResult.valid ? 'status-message--success' : 'status-message--warning'}`}>
              {validationResult.valid ? 'Configuration is valid.' : 'Validation returned issues.'}
            </div>
            {validationResult.errors.length > 0 ? (
              <div className="yaml-errors">
                {validationResult.errors.map((item) => (
                  <div key={item} className="probe-check">
                    <span>Issue</span>
                    <strong>{item}</strong>
                  </div>
                ))}
              </div>
            ) : null}
          </section>
        ) : null}

        {applyResult ? (
          <section className="sheet-section">
            <h3>Last apply</h3>
            <div className="detail-grid">
              <div className="detail-grid__row"><span>Message</span><strong>{applyResult.message}</strong></div>
              <div className="detail-grid__row"><span>Config version</span><strong>{applyResult.config_version}</strong></div>
            </div>
          </section>
        ) : null}
      </WorkbenchSheet>

      <WorkbenchSheet
        open={accessOpen}
        onClose={() => setAccessOpen(false)}
        title="Access control workbench"
        subtitle="Create, edit, reveal, and revoke gateway auth keys while keeping tenant scope and budgets visible."
        actions={(
          <>
            <button type="button" className="button button--ghost" onClick={startNewAccessDraft}>
              New draft
            </button>
            <button type="button" className="button button--ghost" onClick={() => void revealAuthKey()} disabled={revealingKey}>
              {revealingKey ? 'Revealing…' : 'Reveal selected'}
            </button>
            <button type="button" className="button button--ghost" onClick={() => void deleteAuthKey()} disabled={deletingKey}>
              {deletingKey ? 'Deleting…' : 'Delete selected'}
            </button>
            <button type="button" className="button button--primary" onClick={() => void saveAuthKey()} disabled={savingKey}>
              {savingKey ? 'Saving…' : accessEditorMode === 'edit' ? 'Save key' : 'Create key'}
            </button>
          </>
        )}
      >
        {accessStatus ? <div className="status-message status-message--success">{accessStatus}</div> : null}
        {accessError ? <div className="status-message status-message--danger">{accessError}</div> : null}
        {revealedKey ? <div className="status-message status-message--warning">Save this key now: <strong>{revealedKey}</strong></div> : null}

        <section className="sheet-section">
          <h3>{accessEditorMode === 'edit' ? 'Edit key policy' : 'Create key'}</h3>
          <div className="sheet-form">
            <label className="sheet-field">
              <span>Name</span>
              <input
                name="auth-key-name"
                autoComplete="off"
                value={accessForm.name}
                onChange={(event) => setAccessForm((current) => ({ ...current, name: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>Tenant ID</span>
              <input
                name="auth-key-tenant-id"
                autoComplete="off"
                value={accessForm.tenantId}
                onChange={(event) => setAccessForm((current) => ({ ...current, tenantId: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>Allowed models</span>
              <input
                name="auth-key-models"
                autoComplete="off"
                value={accessForm.allowedModels}
                onChange={(event) => setAccessForm((current) => ({ ...current, allowedModels: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>Allowed credentials</span>
              <input
                name="auth-key-credentials"
                autoComplete="off"
                value={accessForm.allowedCredentials}
                onChange={(event) => setAccessForm((current) => ({ ...current, allowedCredentials: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>RPM</span>
              <input
                name="auth-key-rpm"
                inputMode="numeric"
                autoComplete="off"
                value={accessForm.rpm}
                onChange={(event) => setAccessForm((current) => ({ ...current, rpm: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>TPM</span>
              <input
                name="auth-key-tpm"
                inputMode="numeric"
                autoComplete="off"
                value={accessForm.tpm}
                onChange={(event) => setAccessForm((current) => ({ ...current, tpm: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>Cost / day USD</span>
              <input
                name="auth-key-cost-per-day"
                inputMode="decimal"
                autoComplete="off"
                value={accessForm.costPerDayUsd}
                onChange={(event) => setAccessForm((current) => ({ ...current, costPerDayUsd: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>Expires at</span>
              <input
                name="auth-key-expires-at"
                type="datetime-local"
                value={accessForm.expiresAt}
                onChange={(event) => setAccessForm((current) => ({ ...current, expiresAt: event.target.value }))}
              />
            </label>
            <label className="detail-grid__row">
              <span>Budget enabled</span>
              <input
                type="checkbox"
                checked={accessForm.budgetEnabled}
                onChange={(event) => setAccessForm((current) => ({ ...current, budgetEnabled: event.target.checked }))}
              />
            </label>
            <label className="sheet-field">
              <span>Budget total USD</span>
              <input
                name="auth-key-budget-total"
                inputMode="decimal"
                autoComplete="off"
                value={accessForm.budgetTotalUsd}
                onChange={(event) => setAccessForm((current) => ({ ...current, budgetTotalUsd: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>Budget period</span>
              <select
                value={accessForm.budgetPeriod}
                onChange={(event) => setAccessForm((current) => ({ ...current, budgetPeriod: event.target.value as AccessPolicyFormState['budgetPeriod'] }))}
              >
                <option value="daily">daily</option>
                <option value="monthly">monthly</option>
              </select>
            </label>
          </div>
        </section>

        {selectedAuthKey ? (
          <section className="sheet-section">
            <h3>Selected key posture</h3>
            <div className="detail-grid">
              <div className="detail-grid__row"><span>Key</span><strong>{selectedAuthKey.key_masked}</strong></div>
              <div className="detail-grid__row"><span>Tenant</span><strong>{selectedAuthKey.tenant_id ?? 'global'}</strong></div>
              <div className="detail-grid__row"><span>Model allowlist</span><strong>{selectedAuthKey.allowed_models.length || 'all'}</strong></div>
              <div className="detail-grid__row"><span>Credential allowlist</span><strong>{selectedAuthKey.allowed_credentials.length || 'all'}</strong></div>
            </div>
          </section>
        ) : null}

        <section className="sheet-section">
          <h3>Existing keys</h3>
          <div className="probe-list">
            {authKeys.length === 0 ? (
              <div className="probe-check">
                <span>Keys</span>
                <strong>None configured</strong>
              </div>
            ) : (
              authKeys.map((item) => (
                <div key={item.id} className={`probe-check ${item.id === selectedAuthKeyId ? 'probe-check--selected' : ''}`}>
                  <span>{item.key_masked}</span>
                  <strong>{item.name ?? 'unnamed'} · {item.tenant_id ?? 'global'}</strong>
                  <button
                    type="button"
                    className="button button--ghost"
                    onClick={() => {
                      setSelectedAuthKeyId(item.id);
                      setAccessEditorMode('edit');
                    }}
                  >
                    Select
                  </button>
                </div>
              ))
            )}
          </div>
        </section>
      </WorkbenchSheet>
    </div>
  );
}
