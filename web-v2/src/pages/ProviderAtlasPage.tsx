import { useEffect, useMemo, useState } from 'react';
import { Panel } from '../components/Panel';
import { StatusPill } from '../components/StatusPill';
import { WorkbenchSheet } from '../components/WorkbenchSheet';
import { useProviderAtlasData } from '../hooks/useWorkspaceData';
import {
  emptyProfileForm,
  isManagedMode,
  profileKey,
  type AuthProfileFormState,
  type DeviceFlowState,
} from '../lib/authProfileDraft';
import { authProfilesApi } from '../services/authProfiles';
import { getApiErrorMessage } from '../services/errors';
import { protocolsApi } from '../services/protocols';
import { providersApi } from '../services/providers';
import type {
  AuthProfileSummary,
  AuthProfilesRuntimeResponse,
  PresentationPreviewResponse,
  ProtocolMatrixResponse,
  ProviderCapabilityEntry,
  ProviderCreateRequest,
  ProviderDetail,
  ProviderHealthResult,
} from '../types/backend';

interface ProviderRegistryFormState {
  name: string;
  format: 'openai' | 'claude' | 'gemini';
  upstream: string;
  apiKey: string;
  baseUrl: string;
  models: string;
  disabled: boolean;
}

const emptyRegistryForm: ProviderRegistryFormState = {
  name: '',
  format: 'openai',
  upstream: 'openai',
  apiKey: '',
  baseUrl: '',
  models: '',
  disabled: true,
};

function protocolCoverageLabel(mode?: string | null) {
  if (!mode) return 'unsupported';
  if (mode === 'native') return 'native';
  return 'adapted';
}

export function ProviderAtlasPage() {
  const { data, error, loading, reload } = useProviderAtlasData();
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null);
  const [editorOpen, setEditorOpen] = useState(false);
  const [registryOpen, setRegistryOpen] = useState(false);
  const [authWorkbenchOpen, setAuthWorkbenchOpen] = useState(false);
  const [detail, setDetail] = useState<ProviderDetail | null>(null);
  const [health, setHealth] = useState<ProviderHealthResult | null>(null);
  const [preview, setPreview] = useState<PresentationPreviewResponse | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [actionStatus, setActionStatus] = useState<string | null>(null);
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [saving, setSaving] = useState(false);
  const [previewing, setPreviewing] = useState(false);
  const [runtimeInfo, setRuntimeInfo] = useState<AuthProfilesRuntimeResponse | null>(null);
  const [capabilityEntries, setCapabilityEntries] = useState<ProviderCapabilityEntry[]>([]);
  const [protocolMatrix, setProtocolMatrix] = useState<ProtocolMatrixResponse | null>(null);
  const [profiles, setProfiles] = useState<AuthProfileSummary[]>([]);
  const [refreshingProfileId, setRefreshingProfileId] = useState<string | null>(null);
  const [selectedAuthProfileId, setSelectedAuthProfileId] = useState<string | null>(null);
  const [importingProfileId, setImportingProfileId] = useState<string | null>(null);
  const [formState, setFormState] = useState({
    baseUrl: '',
    region: '',
    weight: '1',
    disabled: false,
  });
  const [registryForm, setRegistryForm] = useState<ProviderRegistryFormState>(emptyRegistryForm);
  const [registryLoading, setRegistryLoading] = useState(false);
  const [registryStatus, setRegistryStatus] = useState<string | null>(null);
  const [registryError, setRegistryError] = useState<string | null>(null);
  const [authForm, setAuthForm] = useState<AuthProfileFormState>(emptyProfileForm);
  const [authStatus, setAuthStatus] = useState<string | null>(null);
  const [authError, setAuthError] = useState<string | null>(null);
  const [authLoading, setAuthLoading] = useState(false);
  const [authSaving, setAuthSaving] = useState(false);
  const [authEditorMode, setAuthEditorMode] = useState<'create' | 'edit'>('create');
  const [connectSecret, setConnectSecret] = useState('');
  const [importPath, setImportPath] = useState('');
  const [connectingProfileId, setConnectingProfileId] = useState<string | null>(null);
  const [deviceFlow, setDeviceFlow] = useState<DeviceFlowState | null>(null);
  const [protocolSearch, setProtocolSearch] = useState('');
  const [modelSearch, setModelSearch] = useState('');

  useEffect(() => {
    setSelectedProvider((current) => current ?? data?.providers[0]?.provider ?? null);
  }, [data]);

  const loadRuntimeSurfaces = async () => {
    const [capabilities, protocols, profileList] = await Promise.all([
      providersApi.capabilities(),
      protocolsApi.matrix(),
      authProfilesApi.list(),
    ]);
    setCapabilityEntries(capabilities.providers);
    setProtocolMatrix(protocols);
    setProfiles(profileList.profiles);
  };

  useEffect(() => {
    let active = true;

    void (async () => {
      try {
        const [capabilities, protocols, profileList] = await Promise.all([
          providersApi.capabilities(),
          protocolsApi.matrix(),
          authProfilesApi.list(),
        ]);
        if (!active) {
          return;
        }
        setCapabilityEntries(capabilities.providers);
        setProtocolMatrix(protocols);
        setProfiles(profileList.profiles);
      } catch {
        if (!active) {
          return;
        }
        setCapabilityEntries([]);
        setProtocolMatrix(null);
        setProfiles([]);
      }
    })();

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    setAuthForm((current) => ({
      ...current,
      provider: current.provider || selectedProvider || data?.providers[0]?.provider || '',
    }));
  }, [data?.providers, selectedProvider]);

  const selectedRow = useMemo(
    () => data?.providers.find((provider) => provider.provider === selectedProvider) ?? null,
    [data, selectedProvider],
  );
  const selectedCapabilities = useMemo(
    () => capabilityEntries.find((provider) => provider.name === selectedProvider) ?? null,
    [capabilityEntries, selectedProvider],
  );
  const selectedProfiles = useMemo(
    () => profiles.filter((profile) => profile.provider === (authForm.provider || selectedProvider)),
    [authForm.provider, profiles, selectedProvider],
  );
  const selectedAuthProfile = useMemo(
    () => selectedProfiles.find((profile) => profileKey(profile.provider, profile.id) === selectedAuthProfileId) ?? null,
    [selectedAuthProfileId, selectedProfiles],
  );
  const selectedProviderName = authForm.provider || selectedProvider || data?.providers[0]?.provider || '';
  const selectedAuthProfileMode = selectedAuthProfile?.mode ?? authForm.mode;
  const protocolFacts = useMemo(() => {
    const endpoints = protocolMatrix?.endpoints ?? [];
    const coverage = protocolMatrix?.coverage.filter((entry) => !entry.disabled) ?? [];
    return {
      publicRoutes: endpoints.filter((entry) => entry.scope === 'public').length,
      providerRoutes: endpoints.filter((entry) => entry.scope === 'provider_scoped').length,
      nativeSurfaces: coverage.filter((entry) => entry.execution_mode === 'native').length,
      adaptedSurfaces: coverage.filter((entry) => entry.execution_mode && entry.execution_mode !== 'native').length,
    };
  }, [protocolMatrix]);
  const modelInventory = useMemo(() => {
    const rows = capabilityEntries
      .filter((entry) => !entry.disabled)
      .flatMap((entry) =>
        entry.models.map((model) => ({
          id: model.alias ?? model.id,
          provider: entry.name,
          upstream: entry.upstream,
          probe: entry.probe_status,
        })),
      );
    return rows.slice(0, 8);
  }, [capabilityEntries]);
  const filteredProtocolCoverage = useMemo(() => {
    const needle = protocolSearch.trim().toLowerCase();
    return (protocolMatrix?.coverage ?? [])
      .filter((entry) => entry.provider === selectedProvider)
      .filter((entry) => {
        if (!needle) {
          return true;
        }
        return [
          entry.surface_label,
          entry.surface_id,
          entry.execution_mode ?? '',
          entry.upstream,
        ].join(' ').toLowerCase().includes(needle);
      })
      .slice(0, 8);
  }, [protocolMatrix?.coverage, protocolSearch, selectedProvider]);
  const filteredModelInventory = useMemo(() => {
    const needle = modelSearch.trim().toLowerCase();
    return modelInventory.filter((item) => {
      if (!needle) {
        return true;
      }
      return [item.id, item.provider, item.upstream, item.probe].join(' ').toLowerCase().includes(needle);
    });
  }, [modelInventory, modelSearch]);

  useEffect(() => {
    if (!selectedAuthProfile) {
      return;
    }
    setAuthEditorMode('edit');
    setAuthForm({
      provider: selectedAuthProfile.provider,
      id: selectedAuthProfile.id,
      mode: selectedAuthProfile.mode,
      secret: '',
      disabled: selectedAuthProfile.disabled,
      weight: String(selectedAuthProfile.weight ?? 1),
      region: selectedAuthProfile.region ?? '',
      prefix: selectedAuthProfile.prefix ?? '',
    });
    setConnectSecret('');
  }, [selectedAuthProfile]);

  const openEditor = async () => {
    if (!selectedProvider) {
      return;
    }
    setEditorOpen(true);
    setLoadingDetail(true);
    setActionError(null);
    setActionStatus(null);
    setHealth(null);
    setPreview(null);

    try {
      const [provider, runtime] = await Promise.all([
        providersApi.get(selectedProvider),
        authProfilesApi.runtime(),
      ]);
      setDetail(provider);
      setRuntimeInfo(runtime);
      setFormState({
        baseUrl: provider.base_url ?? '',
        region: provider.region ?? '',
        weight: String(provider.weight ?? 1),
        disabled: provider.disabled,
      });
    } catch (editorError) {
      setActionError(getApiErrorMessage(editorError, 'Failed to load provider detail'));
    } finally {
      setLoadingDetail(false);
    }
  };

  const openRegistryWorkbench = () => {
    setRegistryOpen(true);
    setRegistryStatus(null);
    setRegistryError(null);
    setRegistryForm(emptyRegistryForm);
  };

  const openAuthWorkbench = async () => {
    setAuthWorkbenchOpen(true);
    setAuthStatus(null);
    setAuthError(null);
    setAuthEditorMode('create');
    setConnectSecret('');
    setImportPath('');
    setDeviceFlow(null);
    setSelectedAuthProfileId(null);
    setAuthLoading(true);
    try {
      const [runtime, profileList] = await Promise.all([
        authProfilesApi.runtime(),
        authProfilesApi.list(),
      ]);
      setRuntimeInfo(runtime);
      setProfiles(profileList.profiles);
      const preferredProvider = selectedProvider ?? profileList.profiles[0]?.provider ?? data?.providers[0]?.provider ?? '';
      const preferredProfile = profileList.profiles.find((profile) => profile.provider === preferredProvider) ?? profileList.profiles[0] ?? null;
      setSelectedAuthProfileId(preferredProfile ? profileKey(preferredProfile.provider, preferredProfile.id) : null);
      setAuthForm({
        ...emptyProfileForm,
        provider: preferredProvider,
      });
    } catch (loadError) {
      setAuthError(getApiErrorMessage(loadError, 'Failed to load auth profiles'));
    } finally {
      setAuthLoading(false);
    }
  };

  const refreshAuthProfile = async (provider: string, profileId: string) => {
    setRefreshingProfileId(profileKey(provider, profileId));
    setAuthError(null);
    setAuthStatus(null);
    try {
      const response = await authProfilesApi.refresh(provider, profileId);
      setAuthStatus(`Refreshed auth profile ${response.profile.qualified_name}.`);
      await loadRuntimeSurfaces();
      if (selectedProvider === provider) {
        const refreshed = await providersApi.get(provider);
        setDetail(refreshed);
      }
    } catch (refreshError) {
      setAuthError(getApiErrorMessage(refreshError, 'Failed to refresh auth profile'));
    } finally {
      setRefreshingProfileId(null);
    }
  };

  const importSelectedProfile = async () => {
    if (!selectedAuthProfile) {
      setAuthError('Select an auth profile first.');
      return;
    }

    setImportingProfileId(profileKey(selectedAuthProfile.provider, selectedAuthProfile.id));
    setAuthError(null);
    setAuthStatus(null);
    try {
      const response = await authProfilesApi.importLocal(
        selectedAuthProfile.provider,
        selectedAuthProfile.id,
        importPath.trim() || undefined,
      );
      setAuthStatus(`Imported local credentials into ${response.profile.qualified_name}.`);
      await loadRuntimeSurfaces();
    } catch (importError) {
      setAuthError(getApiErrorMessage(importError, 'Failed to import local auth state'));
    } finally {
      setImportingProfileId(null);
    }
  };

  const deleteSelectedProfile = async () => {
    if (!selectedAuthProfile) {
      setAuthError('Select an auth profile first.');
      return;
    }
    if (!window.confirm(`Delete auth profile "${selectedAuthProfile.qualified_name}"?`)) {
      return;
    }

    setAuthError(null);
    setAuthStatus(null);
    try {
      await authProfilesApi.remove(selectedAuthProfile.provider, selectedAuthProfile.id);
      setAuthStatus(`Deleted auth profile ${selectedAuthProfile.qualified_name}.`);
      setSelectedAuthProfileId(null);
      await loadRuntimeSurfaces();
      await reload();
    } catch (deleteError) {
      setAuthError(getApiErrorMessage(deleteError, 'Failed to delete auth profile'));
    }
  };

  const startNewAuthProfileDraft = () => {
    setAuthEditorMode('create');
    setSelectedAuthProfileId(null);
    setConnectSecret('');
    setImportPath('');
    setDeviceFlow(null);
    setAuthError(null);
    setAuthStatus(null);
    setAuthForm({
      ...emptyProfileForm,
      provider: selectedProviderName,
    });
  };

  const saveAuthProfile = async () => {
    if (!authForm.provider.trim() || !authForm.id.trim()) {
      setAuthError('Provider and profile id are required.');
      return;
    }

    if (!isManagedMode(authForm.mode) && authEditorMode === 'create' && !authForm.secret.trim()) {
      setAuthError('Secret is required for API key and bearer token auth profiles.');
      return;
    }

    setAuthSaving(true);
    setAuthError(null);
    setAuthStatus(null);
    try {
      const payload = {
        mode: authForm.mode,
        secret: isManagedMode(authForm.mode) ? undefined : authForm.secret.trim() || undefined,
        disabled: authForm.disabled,
        weight: Number(authForm.weight) || 1,
        region: authForm.region.trim() || null,
        prefix: authForm.prefix.trim() || null,
      };

      const response = authEditorMode === 'edit' && selectedAuthProfile
        ? await authProfilesApi.replace(selectedAuthProfile.provider, selectedAuthProfile.id, payload)
        : await authProfilesApi.create({
            provider: authForm.provider.trim(),
            id: authForm.id.trim(),
            ...payload,
          });

      setAuthStatus(`${authEditorMode === 'edit' ? 'Saved' : 'Created'} auth profile ${response.profile.qualified_name}.`);
      setSelectedAuthProfileId(profileKey(response.profile.provider, response.profile.id));
      setAuthForm((current) => ({ ...current, secret: '' }));
      await loadRuntimeSurfaces();
      await reload();
    } catch (createError) {
      setAuthError(getApiErrorMessage(createError, 'Failed to save auth profile'));
    } finally {
      setAuthSaving(false);
    }
  };

  const connectSelectedProfile = async () => {
    if (!selectedAuthProfile) {
      setAuthError('Select an auth profile first.');
      return;
    }
    if (selectedAuthProfile.mode !== 'anthropic-claude-subscription') {
      setAuthError('Secret connect is only supported for Claude subscription profiles.');
      return;
    }
    if (!connectSecret.trim()) {
      setAuthError('Enter the subscription token first.');
      return;
    }

    const currentKey = profileKey(selectedAuthProfile.provider, selectedAuthProfile.id);
    setConnectingProfileId(currentKey);
    setAuthError(null);
    setAuthStatus(null);
    try {
      const response = await authProfilesApi.connect(selectedAuthProfile.provider, selectedAuthProfile.id, {
        secret: connectSecret.trim(),
      });
      setAuthStatus(`Connected ${response.profile.qualified_name}.`);
      setConnectSecret('');
      await loadRuntimeSurfaces();
      await reload();
    } catch (connectError) {
      setAuthError(getApiErrorMessage(connectError, 'Failed to connect auth profile'));
    } finally {
      setConnectingProfileId(null);
    }
  };

  const startBrowserOauth = async () => {
    if (!selectedAuthProfile) {
      setAuthError('Select an auth profile first.');
      return;
    }
    if (selectedAuthProfile.mode !== 'codex-oauth') {
      setAuthError('Browser OAuth is only available for Codex OAuth profiles.');
      return;
    }

    const currentKey = profileKey(selectedAuthProfile.provider, selectedAuthProfile.id);
    setConnectingProfileId(currentKey);
    setAuthError(null);
    setAuthStatus(null);
    try {
      const redirectUri = `${window.location.origin}/provider-atlas/callback`;
      const response = await authProfilesApi.startCodexOauth({
        provider: selectedAuthProfile.provider,
        profile_id: selectedAuthProfile.id,
        redirect_uri: redirectUri,
      });
      window.location.assign(response.auth_url);
    } catch (startError) {
      setAuthError(getApiErrorMessage(startError, 'Failed to start browser OAuth'));
      setConnectingProfileId(null);
    }
  };

  const startDeviceFlow = async () => {
    if (!selectedAuthProfile) {
      setAuthError('Select an auth profile first.');
      return;
    }
    if (selectedAuthProfile.mode !== 'codex-oauth') {
      setAuthError('Device flow is only available for Codex OAuth profiles.');
      return;
    }

    const currentKey = profileKey(selectedAuthProfile.provider, selectedAuthProfile.id);
    setConnectingProfileId(currentKey);
    setAuthError(null);
    setAuthStatus(null);
    try {
      const response = await authProfilesApi.startCodexDevice({
        provider: selectedAuthProfile.provider,
        profile_id: selectedAuthProfile.id,
      });
      setDeviceFlow({ ...response, status: 'pending' });
      setAuthStatus(`Started device flow for ${selectedAuthProfile.qualified_name}.`);
    } catch (startError) {
      setAuthError(getApiErrorMessage(startError, 'Failed to start device flow'));
    } finally {
      setConnectingProfileId(null);
    }
  };

  useEffect(() => {
    if (!selectedAuthProfile || !deviceFlow) {
      return;
    }

    let cancelled = false;
    const interval = window.setInterval(() => {
      if (cancelled) {
        return;
      }
      void authProfilesApi.pollCodexDevice(deviceFlow.state)
        .then(async (result) => {
          if (cancelled || result.status !== 'completed') {
            return;
          }
          setAuthStatus(`Connected ${selectedAuthProfile.qualified_name} via device flow.`);
          setDeviceFlow(null);
          await loadRuntimeSurfaces();
          await reload();
        })
        .catch((pollError) => {
          if (cancelled) {
            return;
          }
          setAuthError(getApiErrorMessage(pollError, 'Device flow polling failed'));
        });
    }, Math.max(deviceFlow.interval_secs, 2) * 1000);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [deviceFlow, reload, selectedAuthProfile]);

  const runHealthCheck = async () => {
    if (!selectedProvider) {
      return;
    }
    setActionError(null);
    setActionStatus('Running real provider health probe…');
    try {
      const result = await providersApi.healthCheck(selectedProvider);
      setHealth(result);
      setActionStatus(`Health probe completed with status ${result.status}.`);
    } catch (probeError) {
      setActionError(getApiErrorMessage(probeError, 'Health probe failed'));
    }
  };

  const runPresentationPreview = async () => {
    if (!selectedProvider) {
      return;
    }
    setPreviewing(true);
    setActionError(null);
    setActionStatus(null);
    try {
      const result = await providersApi.presentationPreview(selectedProvider, {
        model: detail?.models[0]?.id ?? selectedCapabilities?.models[0]?.id ?? 'gpt-5',
        user_agent: 'prism-control-plane-v2',
        sample_body: {
          input: 'hello',
          messages: [{ role: 'user', content: 'hello' }],
        },
      });
      setPreview(result);
      setActionStatus(`Presentation preview generated for ${selectedProvider}.`);
    } catch (previewError) {
      setActionError(getApiErrorMessage(previewError, 'Presentation preview failed'));
    } finally {
      setPreviewing(false);
    }
  };

  const saveProvider = async () => {
    if (!selectedProvider) {
      return;
    }
    setSaving(true);
    setActionError(null);
    setActionStatus(null);
    try {
      await providersApi.update(selectedProvider, {
        base_url: formState.baseUrl.trim() || null,
        region: formState.region.trim() || null,
        weight: Number(formState.weight) || 1,
        disabled: formState.disabled,
      });
      setActionStatus(`Saved provider ${selectedProvider}.`);
      await reload();
      await loadRuntimeSurfaces();
      const refreshed = await providersApi.get(selectedProvider);
      setDetail(refreshed);
    } catch (saveError) {
      setActionError(getApiErrorMessage(saveError, 'Failed to save provider'));
    } finally {
      setSaving(false);
    }
  };

  const fetchModelsIntoDraft = async () => {
    if (!registryForm.apiKey.trim()) {
      setRegistryError('An API key is required to fetch models.');
      return;
    }

    setRegistryLoading(true);
    setRegistryError(null);
    setRegistryStatus(null);
    try {
      const result = await providersApi.fetchModels({
        format: registryForm.format,
        upstream: registryForm.upstream,
        api_key: registryForm.apiKey.trim(),
        base_url: registryForm.baseUrl.trim() || undefined,
      });
      setRegistryForm((current) => ({ ...current, models: result.models.join(', ') }));
      setRegistryStatus(`Fetched ${result.models.length} models from upstream.`);
    } catch (fetchError) {
      setRegistryError(getApiErrorMessage(fetchError, 'Failed to fetch models'));
    } finally {
      setRegistryLoading(false);
    }
  };

  const createProvider = async () => {
    if (!registryForm.name.trim()) {
      setRegistryError('Provider name is required.');
      return;
    }

    const body: ProviderCreateRequest = {
      name: registryForm.name.trim(),
      format: registryForm.format,
      upstream: registryForm.upstream,
      api_key: registryForm.apiKey.trim() || undefined,
      base_url: registryForm.baseUrl.trim() || null,
      models: registryForm.models
        .split(',')
        .map((item) => item.trim())
        .filter(Boolean),
      disabled: registryForm.disabled,
    };

    setRegistryLoading(true);
    setRegistryError(null);
    setRegistryStatus(null);
    try {
      await providersApi.create(body);
      setRegistryStatus(`Created provider ${body.name}.`);
      setSelectedProvider(body.name);
      setRegistryForm(emptyRegistryForm);
      await reload();
      await loadRuntimeSurfaces();
    } catch (createError) {
      setRegistryError(getApiErrorMessage(createError, 'Failed to create provider'));
    } finally {
      setRegistryLoading(false);
    }
  };

  const deleteSelectedProvider = async () => {
    if (!selectedProvider) {
      setRegistryError('Select a provider first.');
      return;
    }
    if (!window.confirm(`Delete provider "${selectedProvider}"?`)) {
      return;
    }

    setRegistryLoading(true);
    setRegistryError(null);
    setRegistryStatus(null);
    try {
      await providersApi.remove(selectedProvider);
      setRegistryStatus(`Deleted provider ${selectedProvider}.`);
      setSelectedProvider(null);
      setDetail(null);
      await reload();
      await loadRuntimeSurfaces();
    } catch (deleteError) {
      setRegistryError(getApiErrorMessage(deleteError, 'Failed to delete provider'));
    } finally {
      setRegistryLoading(false);
    }
  };

  return (
    <div className="workspace-grid">
      <section className="hero">
        <div>
          <p className="workspace-eyebrow">PRISM / PROVIDER ATLAS</p>
          <h1>Runtime entities with identity and auth posture</h1>
          <p className="workspace-summary">
            Provider management should feel like runtime operations, not static CRUD. Coverage, auth state, protocol exposure, and routing participation stay visible together.
          </p>
        </div>
        <div className="hero-actions">
          <button className="button button--primary" onClick={() => void openEditor()}>
            Open provider editor
          </button>
          <button className="button button--ghost" onClick={() => void openAuthWorkbench()}>
            Auth profile workbench
          </button>
        </div>
      </section>

      {selectedRow ? (
        <div className="status-message status-message--warning">
          Active provider: <strong>{selectedRow.provider}</strong> · {selectedRow.status} · {selectedRow.auth}
        </div>
      ) : null}

      <div className="two-column">
        <Panel title="Provider roster" subtitle="Entity graph for providers, auth profiles, and live probe posture." className="panel--wide">
          <div className="inline-actions">
            <button type="button" className="button button--ghost" onClick={openRegistryWorkbench}>
              Provider registry
            </button>
          </div>
          <div className="table-grid table-grid--providers">
            <div className="table-grid__head">Provider</div>
            <div className="table-grid__head">Format</div>
            <div className="table-grid__head">Auth</div>
            <div className="table-grid__head">Status</div>
            <div className="table-grid__head">Rotation</div>
            {loading && !data ? <div className="table-grid__cell">Loading providers…</div> : null}
            {error && !data ? <div className="table-grid__cell">{error}</div> : null}
            {(data?.providers ?? []).flatMap((provider) => {
              const selected = provider.provider === selectedProvider;
              const cellClass = `table-grid__cell ${selected ? 'is-selected' : ''} is-clickable`;
              return [
                <div
                  key={`${provider.provider}-name`}
                  className={`${cellClass} table-grid__cell--strong`}
                  onClick={() => setSelectedProvider(provider.provider)}
                >
                  {provider.provider}
                </div>,
                <div key={`${provider.provider}-format`} className={cellClass} onClick={() => setSelectedProvider(provider.provider)}>
                  {provider.format}
                </div>,
                <div key={`${provider.provider}-auth`} className={cellClass} onClick={() => setSelectedProvider(provider.provider)}>
                  {provider.auth}
                </div>,
                <div key={`${provider.provider}-status`} className={cellClass} onClick={() => setSelectedProvider(provider.provider)}>
                  <StatusPill label={provider.status} tone={provider.status_tone} />
                </div>,
                <div key={`${provider.provider}-rotation`} className={cellClass} onClick={() => setSelectedProvider(provider.provider)}>
                  {provider.rotation}
                </div>,
              ];
            })}
          </div>
        </Panel>

        <Panel title="Capability coverage" subtitle="Protocol truth, model surface, and auth/runtime readiness.">
          <ul className="fact-list">
            {(data?.coverage ?? []).map((fact) => (
              <li key={fact.label}><span>{fact.label}</span><strong>{fact.value}</strong></li>
            ))}
            {selectedCapabilities ? (
              <>
                <li><span>Probe status</span><strong>{selectedCapabilities.probe_status}</strong></li>
                <li><span>Presentation</span><strong>{selectedCapabilities.presentation_profile}</strong></li>
                <li><span>Model surface</span><strong>{selectedCapabilities.models.length}</strong></li>
                <li><span>Tool support</span><strong>{selectedCapabilities.probe.tools.status}</strong></li>
              </>
            ) : null}
          </ul>
        </Panel>
      </div>

      <div className="two-column">
        <Panel title="Protocol surfaces" subtitle="Ingress routes, execution modes, and provider truth should be visible without opening legacy pages.">
          <div className="inline-actions">
            <input
              name="provider-protocol-search"
              placeholder="Filter protocol surfaces"
              autoComplete="off"
              value={protocolSearch}
              onChange={(event) => setProtocolSearch(event.target.value)}
            />
          </div>
          <ul className="fact-list">
            <li><span>Public routes</span><strong>{protocolFacts.publicRoutes}</strong></li>
            <li><span>Provider routes</span><strong>{protocolFacts.providerRoutes}</strong></li>
            <li><span>Native surfaces</span><strong>{protocolFacts.nativeSurfaces}</strong></li>
            <li><span>Adapted surfaces</span><strong>{protocolFacts.adaptedSurfaces}</strong></li>
          </ul>
          <div className="probe-list">
            {filteredProtocolCoverage.map((entry) => (
              <div key={`${entry.provider}-${entry.surface_id}`} className="probe-check">
                <span>{entry.surface_label}</span>
                <strong>{protocolCoverageLabel(entry.execution_mode)}</strong>
              </div>
            ))}
          </div>
        </Panel>

        <Panel title="Model inventory" subtitle="Unique model mappings and runtime capability truth are part of provider operations, not a separate admin silo.">
          <div className="inline-actions">
            <input
              name="provider-model-search"
              placeholder="Filter model inventory"
              autoComplete="off"
              value={modelSearch}
              onChange={(event) => setModelSearch(event.target.value)}
            />
          </div>
          {filteredModelInventory.length === 0 ? (
            <div className="status-message">No provider model inventory is configured yet.</div>
          ) : (
            <div className="probe-list">
              {filteredModelInventory.map((item) => (
                <div key={`${item.provider}-${item.id}`} className="probe-check">
                  <span>{item.id}</span>
                  <strong>{item.provider} · {item.probe}</strong>
                </div>
              ))}
            </div>
          )}
        </Panel>
      </div>

      <WorkbenchSheet
        open={editorOpen}
        onClose={() => setEditorOpen(false)}
        title="Provider editor"
        subtitle="Edit runtime-facing provider fields, run a real upstream health probe, and preview presentation mutations."
        actions={(
          <>
            <button type="button" className="button button--ghost" onClick={() => void runHealthCheck()}>
              Run health probe
            </button>
            <button type="button" className="button button--ghost" onClick={() => void runPresentationPreview()} disabled={previewing}>
              {previewing ? 'Previewing…' : 'Presentation preview'}
            </button>
            <button type="button" className="button button--primary" onClick={() => void saveProvider()} disabled={saving}>
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
                    onChange={(event) => setFormState((current) => ({ ...current, baseUrl: event.target.value }))}
                  />
                </label>
                <label className="sheet-field">
                  <span>Region</span>
                  <input
                    name="provider-region"
                    autoComplete="off"
                    value={formState.region}
                    onChange={(event) => setFormState((current) => ({ ...current, region: event.target.value }))}
                  />
                </label>
                <label className="sheet-field">
                  <span>Weight</span>
                  <input
                    name="provider-weight"
                    inputMode="numeric"
                    autoComplete="off"
                    value={formState.weight}
                    onChange={(event) => setFormState((current) => ({ ...current, weight: event.target.value }))}
                  />
                </label>
                <label className="detail-grid__row">
                  <span>Disabled</span>
                  <input
                    type="checkbox"
                    checked={formState.disabled}
                    onChange={(event) => setFormState((current) => ({ ...current, disabled: event.target.checked }))}
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
                          onClick={() => void refreshAuthProfile(detail.name, profile.id)}
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

      <WorkbenchSheet
        open={registryOpen}
        onClose={() => setRegistryOpen(false)}
        title="Provider registry workbench"
        subtitle="Create disabled providers, fetch model inventories, and remove obsolete runtime entities without leaving the atlas."
        actions={(
          <>
            <button type="button" className="button button--ghost" onClick={() => void fetchModelsIntoDraft()} disabled={registryLoading}>
              {registryLoading ? 'Working…' : 'Fetch models'}
            </button>
            <button type="button" className="button button--ghost" onClick={() => void deleteSelectedProvider()} disabled={registryLoading || !selectedProvider}>
              Delete selected
            </button>
            <button type="button" className="button button--primary" onClick={() => void createProvider()} disabled={registryLoading}>
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
                  void createProvider();
                }}
              >
                <label className="sheet-field">
                  <span>Name</span>
                  <input
                    name="provider-name"
                    autoComplete="organization"
                    value={registryForm.name}
                    onChange={(event) => setRegistryForm((current) => ({ ...current, name: event.target.value }))}
                  />
                </label>
                <label className="sheet-field">
                  <span>Format</span>
                  <select value={registryForm.format} onChange={(event) => setRegistryForm((current) => ({ ...current, format: event.target.value as ProviderRegistryFormState['format'] }))}>
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
                    onChange={(event) => setRegistryForm((current) => ({ ...current, upstream: event.target.value }))}
                  />
                </label>
                <label className="sheet-field">
                  <span>API key</span>
                  <input
                    name="provider-api-key"
                    type="password"
                    autoComplete="new-password"
                    value={registryForm.apiKey}
                    onChange={(event) => setRegistryForm((current) => ({ ...current, apiKey: event.target.value }))}
                  />
                </label>
                <label className="sheet-field">
                  <span>Base URL</span>
                  <input
                    name="registry-base-url"
                    type="url"
                    autoComplete="url"
                    value={registryForm.baseUrl}
                    onChange={(event) => setRegistryForm((current) => ({ ...current, baseUrl: event.target.value }))}
                  />
                </label>
                <label className="sheet-field">
                  <span>Models</span>
                  <input
                    name="provider-models"
                    autoComplete="off"
                    value={registryForm.models}
                    onChange={(event) => setRegistryForm((current) => ({ ...current, models: event.target.value }))}
                  />
                </label>
                <label className="detail-grid__row">
                  <span>Disabled</span>
                  <input
                    type="checkbox"
                    checked={registryForm.disabled}
                    onChange={(event) => setRegistryForm((current) => ({ ...current, disabled: event.target.checked }))}
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
            <div className="detail-grid__row"><span>Coverage</span><strong>{selectedCapabilities?.probe_status ?? 'n/a'}</strong></div>
          </div>
        </section>
      </WorkbenchSheet>

      <WorkbenchSheet
        open={authWorkbenchOpen}
        onClose={() => setAuthWorkbenchOpen(false)}
        title="Auth profile workbench"
        subtitle="Managed auth should be operated as first-class provider identity, not hidden behind provider config blobs."
        actions={(
          <>
            <button type="button" className="button button--ghost" onClick={startNewAuthProfileDraft}>
              New draft
            </button>
            <button
              type="button"
              className="button button--ghost"
              onClick={() => void importSelectedProfile()}
              disabled={!selectedAuthProfile || importingProfileId !== null}
            >
              {importingProfileId ? 'Importing…' : 'Import local'}
            </button>
            <button
              type="button"
              className="button button--ghost"
              onClick={() => void startBrowserOauth()}
              disabled={!selectedAuthProfile || selectedAuthProfileMode !== 'codex-oauth' || connectingProfileId !== null}
            >
              {connectingProfileId ? 'Connecting…' : 'Browser OAuth'}
            </button>
            <button
              type="button"
              className="button button--ghost"
              onClick={() => void startDeviceFlow()}
              disabled={!selectedAuthProfile || selectedAuthProfileMode !== 'codex-oauth' || connectingProfileId !== null}
            >
              {deviceFlow ? 'Device active' : 'Device flow'}
            </button>
            <button
              type="button"
              className="button button--ghost"
              onClick={() => void refreshAuthProfile(selectedAuthProfile?.provider ?? '', selectedAuthProfile?.id ?? '')}
              disabled={!selectedAuthProfile || refreshingProfileId !== null}
            >
              {refreshingProfileId ? 'Refreshing…' : 'Refresh selected'}
            </button>
            <button type="button" className="button button--ghost" onClick={() => void deleteSelectedProfile()} disabled={!selectedAuthProfile}>
              Delete selected
            </button>
            <button type="button" className="button button--primary" onClick={() => void saveAuthProfile()} disabled={authSaving}>
              {authSaving ? 'Saving…' : authEditorMode === 'edit' ? 'Save profile' : 'Create profile'}
            </button>
          </>
        )}
      >
        {authLoading ? <div className="status-message">Loading auth profiles…</div> : null}
        {authStatus ? <div className="status-message status-message--success">{authStatus}</div> : null}
        {authError ? <div className="status-message status-message--danger">{authError}</div> : null}

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

        <section className="sheet-section">
          <h3>{authEditorMode === 'edit' ? 'Edit profile' : 'Create profile'}</h3>
          <form
            className="sheet-form"
            onSubmit={(event) => {
              event.preventDefault();
              void saveAuthProfile();
            }}
          >
            <label className="sheet-field">
              <span>Provider</span>
              <select value={authForm.provider} onChange={(event) => setAuthForm((current) => ({ ...current, provider: event.target.value }))}>
                {(data?.providers ?? []).map((provider) => (
                  <option key={provider.provider} value={provider.provider}>{provider.provider}</option>
                ))}
              </select>
            </label>
            <label className="sheet-field">
              <span>Profile id</span>
              <input
                name="auth-profile-id"
                autoComplete="username"
                value={authForm.id}
                onChange={(event) => setAuthForm((current) => ({ ...current, id: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>Mode</span>
              <select value={authForm.mode} onChange={(event) => setAuthForm((current) => ({ ...current, mode: event.target.value }))}>
                <option value="api-key">api-key</option>
                <option value="bearer-token">bearer-token</option>
                <option value="codex-oauth">codex-oauth</option>
                <option value="anthropic-claude-subscription">anthropic-claude-subscription</option>
              </select>
            </label>
            <label className="sheet-field">
              <span>{isManagedMode(authForm.mode) ? 'Secret (optional on create)' : 'Secret'}</span>
              <input
                name="auth-profile-secret"
                type="password"
                autoComplete="new-password"
                value={authForm.secret}
                onChange={(event) => setAuthForm((current) => ({ ...current, secret: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>Weight</span>
              <input
                name="auth-profile-weight"
                inputMode="numeric"
                autoComplete="off"
                value={authForm.weight}
                onChange={(event) => setAuthForm((current) => ({ ...current, weight: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>Region</span>
              <input
                name="auth-profile-region"
                autoComplete="off"
                value={authForm.region}
                onChange={(event) => setAuthForm((current) => ({ ...current, region: event.target.value }))}
              />
            </label>
            <label className="sheet-field">
              <span>Prefix</span>
              <input
                name="auth-profile-prefix"
                autoComplete="off"
                value={authForm.prefix}
                onChange={(event) => setAuthForm((current) => ({ ...current, prefix: event.target.value }))}
              />
            </label>
            <label className="detail-grid__row">
              <span>Disabled</span>
              <input
                type="checkbox"
                checked={authForm.disabled}
                onChange={(event) => setAuthForm((current) => ({ ...current, disabled: event.target.checked }))}
              />
            </label>
          </form>
        </section>

        {selectedAuthProfile ? (
          <section className="sheet-section">
            <h3>Selected profile posture</h3>
            <div className="detail-grid">
              <div className="detail-grid__row"><span>Profile</span><strong>{selectedAuthProfile.qualified_name}</strong></div>
              <div className="detail-grid__row"><span>Mode</span><strong>{selectedAuthProfile.mode}</strong></div>
              <div className="detail-grid__row"><span>Connected</span><strong>{selectedAuthProfile.connected ? 'yes' : 'no'}</strong></div>
              <div className="detail-grid__row"><span>Account</span><strong>{selectedAuthProfile.email ?? selectedAuthProfile.account_id ?? 'unknown'}</strong></div>
            </div>

            {selectedAuthProfile.mode === 'anthropic-claude-subscription' ? (
              <div className="sheet-form">
                <label className="sheet-field">
                  <span>Subscription token</span>
                  <input
                    name="auth-profile-connect-secret"
                    type="password"
                    autoComplete="new-password"
                    value={connectSecret}
                    onChange={(event) => setConnectSecret(event.target.value)}
                  />
                </label>
                <button
                  type="button"
                  className="button button--secondary"
                  onClick={() => void connectSelectedProfile()}
                  disabled={connectingProfileId === profileKey(selectedAuthProfile.provider, selectedAuthProfile.id)}
                >
                  {connectingProfileId === profileKey(selectedAuthProfile.provider, selectedAuthProfile.id) ? 'Connecting…' : 'Connect secret'}
                </button>
              </div>
            ) : null}

            {selectedAuthProfile.mode === 'codex-oauth' ? (
              <>
                <div className="sheet-form">
                  <label className="sheet-field">
                    <span>Import path</span>
                    <input
                      name="auth-profile-import-path"
                      autoComplete="off"
                      value={importPath}
                      onChange={(event) => setImportPath(event.target.value)}
                    />
                  </label>
                </div>
                {deviceFlow ? (
                  <div className="status-message status-message--warning">
                    Device flow active. Visit <strong>{deviceFlow.verification_url}</strong> and enter code <strong>{deviceFlow.user_code}</strong>.
                  </div>
                ) : null}
              </>
            ) : null}
          </section>
        ) : null}

        <section className="sheet-section">
          <h3>Existing profiles</h3>
          <div className="probe-list">
            {selectedProfiles.length === 0 ? (
              <div className="probe-check">
                <span>Profiles</span>
                <strong>None configured for this provider</strong>
              </div>
            ) : (
              selectedProfiles.map((profile) => {
                const currentKey = profileKey(profile.provider, profile.id);
                return (
                  <div key={currentKey} className={`probe-check ${selectedAuthProfileId === currentKey ? 'probe-check--selected' : ''}`}>
                    <span>{profile.qualified_name}</span>
                    <strong>{profile.mode} · {profile.connected ? 'connected' : 'disconnected'}</strong>
                    <button
                      type="button"
                      className="button button--ghost"
                      onClick={() => {
                        setSelectedAuthProfileId(currentKey);
                        setAuthEditorMode('edit');
                      }}
                    >
                      Select
                    </button>
                  </div>
                );
              })
            )}
          </div>
        </section>
      </WorkbenchSheet>
    </div>
  );
}
