import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type Dispatch,
  type SetStateAction,
} from 'react';
import { confirmAction, navigateTo } from '../../lib/browser';
import {
  emptyProfileForm,
  isManagedMode,
  profileKey,
  resolveDeviceFlowProfileLabel,
  type AuthProfileFormState,
  type DeviceFlowState,
} from '../../lib/authProfileDraft';
import { reconcileSelection } from '../../lib/selection';
import { authProfilesApi } from '../../services/authProfiles';
import { getApiErrorMessage } from '../../services/errors';
import { providersApi } from '../../services/providers';
import type {
  AuthProfileSummary,
  AuthProfilesRuntimeResponse,
  ProviderDetail,
} from '../../types/backend';
import type { ProviderAtlasResponse } from '../../types/controlPlane';

interface UseProviderAtlasAuthWorkbenchOptions {
  providers: ProviderAtlasResponse['providers'];
  selectedProvider: string | null;
  reload: () => Promise<void>;
  setDetail: Dispatch<SetStateAction<ProviderDetail | null>>;
  setRuntimeInfo: Dispatch<SetStateAction<AuthProfilesRuntimeResponse | null>>;
}

export function useProviderAtlasAuthWorkbench({
  providers,
  selectedProvider,
  reload,
  setDetail,
  setRuntimeInfo,
}: UseProviderAtlasAuthWorkbenchOptions) {
  const [authWorkbenchOpen, setAuthWorkbenchOpen] = useState(false);
  const [profiles, setProfiles] = useState<AuthProfileSummary[]>([]);
  const [refreshingProfileId, setRefreshingProfileId] = useState<string | null>(null);
  const [selectedAuthProfileId, setSelectedAuthProfileId] = useState<string | null>(null);
  const [importingProfileId, setImportingProfileId] = useState<string | null>(null);
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

  const loadProfiles = useCallback(async () => {
    const [runtime, profileList] = await Promise.all([
      authProfilesApi.runtime(),
      authProfilesApi.list(),
    ]);
    setRuntimeInfo(runtime);
    setProfiles(profileList.profiles);
    return profileList.profiles;
  }, [setRuntimeInfo]);

  useEffect(() => {
    let active = true;

    void loadProfiles().catch(() => {
      if (!active) {
        return;
      }
      setProfiles([]);
    });

    return () => {
      active = false;
    };
  }, [loadProfiles]);

  useEffect(() => {
    setAuthForm((current) => ({
      ...current,
      provider: current.provider || selectedProvider || providers[0]?.provider || '',
    }));
  }, [providers, selectedProvider]);

  const selectedProfiles = useMemo(
    () => profiles.filter((profile) => profile.provider === (authForm.provider || selectedProvider)),
    [authForm.provider, profiles, selectedProvider],
  );
  const selectedAuthProfile = useMemo(
    () =>
      selectedProfiles.find((profile) => profileKey(profile.provider, profile.id) === selectedAuthProfileId) ??
      null,
    [selectedAuthProfileId, selectedProfiles],
  );
  const selectedProviderName = authForm.provider || selectedProvider || providers[0]?.provider || '';
  const selectedAuthProfileMode = selectedAuthProfile?.mode ?? authForm.mode;

  useEffect(() => {
    setSelectedAuthProfileId((current) =>
      reconcileSelection(current, selectedProfiles, (profile) =>
        profileKey(profile.provider, profile.id),
      ),
    );
  }, [selectedProfiles]);

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
      const profileList = await loadProfiles();
      const preferredProvider =
        selectedProvider ?? profileList[0]?.provider ?? providers[0]?.provider ?? '';
      const preferredProfile =
        profileList.find((profile) => profile.provider === preferredProvider) ??
        profileList[0] ??
        null;
      setSelectedAuthProfileId(
        preferredProfile ? profileKey(preferredProfile.provider, preferredProfile.id) : null,
      );
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
      await loadProfiles();
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
      await loadProfiles();
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
    if (!confirmAction(`Delete auth profile "${selectedAuthProfile.qualified_name}"?`)) {
      return;
    }

    setAuthError(null);
    setAuthStatus(null);
    try {
      await authProfilesApi.remove(selectedAuthProfile.provider, selectedAuthProfile.id);
      setAuthStatus(`Deleted auth profile ${selectedAuthProfile.qualified_name}.`);
      setSelectedAuthProfileId(null);
      await loadProfiles();
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

      const response =
        authEditorMode === 'edit' && selectedAuthProfile
          ? await authProfilesApi.replace(
              selectedAuthProfile.provider,
              selectedAuthProfile.id,
              payload,
            )
          : await authProfilesApi.create({
              provider: authForm.provider.trim(),
              id: authForm.id.trim(),
              ...payload,
            });

      setAuthStatus(
        `${authEditorMode === 'edit' ? 'Saved' : 'Created'} auth profile ${response.profile.qualified_name}.`,
      );
      setSelectedAuthProfileId(profileKey(response.profile.provider, response.profile.id));
      setAuthForm((current) => ({ ...current, secret: '' }));
      await loadProfiles();
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
      await loadProfiles();
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
      navigateTo(response.auth_url);
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
      setDeviceFlow({
        ...response,
        status: 'pending',
        target_profile_key: currentKey,
        target_qualified_name: selectedAuthProfile.qualified_name,
      });
      setAuthStatus(`Started device flow for ${selectedAuthProfile.qualified_name}.`);
    } catch (startError) {
      setAuthError(getApiErrorMessage(startError, 'Failed to start device flow'));
    } finally {
      setConnectingProfileId(null);
    }
  };

  useEffect(() => {
    if (!deviceFlow) {
      return;
    }

    let cancelled = false;
    const interval = window.setInterval(() => {
      if (cancelled) {
        return;
      }
      void authProfilesApi
        .pollCodexDevice(deviceFlow.state)
        .then(async (result) => {
          if (cancelled || result.status !== 'completed') {
            return;
          }
          const profileLabel = resolveDeviceFlowProfileLabel(deviceFlow, result.profile);
          setAuthStatus(`Connected ${profileLabel} via device flow.`);
          setDeviceFlow(null);
          await loadProfiles();
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
  }, [deviceFlow, loadProfiles, reload]);

  return {
    authWorkbenchOpen,
    setAuthWorkbenchOpen,
    authLoading,
    authStatus,
    authError,
    authSaving,
    authEditorMode,
    authForm,
    setAuthForm,
    selectedProfiles,
    selectedAuthProfile,
    selectedAuthProfileId,
    setSelectedAuthProfileId,
    selectedAuthProfileMode,
    connectSecret,
    setConnectSecret,
    importPath,
    setImportPath,
    deviceFlow,
    importingProfileId,
    refreshingProfileId,
    connectingProfileId,
    openAuthWorkbench,
    refreshAuthProfile,
    importSelectedProfile,
    deleteSelectedProfile,
    startNewAuthProfileDraft,
    saveAuthProfile,
    connectSelectedProfile,
    startBrowserOauth,
    startDeviceFlow,
  };
}
