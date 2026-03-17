import { useState, type Dispatch, type SetStateAction } from 'react';
import { confirmAction, navigateTo } from '../../lib/browser';
import {
  isManagedMode,
  profileKey,
  type AuthProfileFormState,
  type DeviceFlowState,
} from '../../lib/authProfileDraft';
import { authProfilesApi } from '../../services/authProfiles';
import { getApiErrorMessage } from '../../services/errors';
import { providersApi } from '../../services/providers';
import type { AuthProfileSummary, ProviderDetail } from '../../types/backend';

interface UseProviderAtlasAuthActionsOptions {
  selectedProvider: string | null;
  reload: () => Promise<void>;
  setDetail: Dispatch<SetStateAction<ProviderDetail | null>>;
  loadProfiles: () => Promise<unknown>;
  authEditorMode: 'create' | 'edit';
  authForm: AuthProfileFormState;
  setAuthForm: Dispatch<SetStateAction<AuthProfileFormState>>;
  selectedAuthProfile: AuthProfileSummary | null;
  setSelectedAuthProfileId: Dispatch<SetStateAction<string | null>>;
  connectSecret: string;
  setConnectSecret: Dispatch<SetStateAction<string>>;
  importPath: string;
  setDeviceFlow: Dispatch<SetStateAction<DeviceFlowState | null>>;
}

export function useProviderAtlasAuthActions({
  selectedProvider,
  reload,
  setDetail,
  loadProfiles,
  authEditorMode,
  authForm,
  setAuthForm,
  selectedAuthProfile,
  setSelectedAuthProfileId,
  connectSecret,
  setConnectSecret,
  importPath,
  setDeviceFlow,
}: UseProviderAtlasAuthActionsOptions) {
  const [refreshingProfileId, setRefreshingProfileId] = useState<string | null>(null);
  const [importingProfileId, setImportingProfileId] = useState<string | null>(null);
  const [authStatus, setAuthStatus] = useState<string | null>(null);
  const [authError, setAuthError] = useState<string | null>(null);
  const [authSaving, setAuthSaving] = useState(false);
  const [connectingProfileId, setConnectingProfileId] = useState<string | null>(null);

  const clearAuthMessages = () => {
    setAuthStatus(null);
    setAuthError(null);
  };

  const refreshAuthProfile = async (provider: string, profileId: string) => {
    setRefreshingProfileId(profileKey(provider, profileId));
    clearAuthMessages();
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
    clearAuthMessages();
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

    clearAuthMessages();
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
    clearAuthMessages();
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
    clearAuthMessages();
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
    clearAuthMessages();
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
    clearAuthMessages();
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

  return {
    refreshingProfileId,
    importingProfileId,
    authStatus,
    authError,
    authSaving,
    connectingProfileId,
    setAuthStatus,
    setAuthError,
    clearAuthMessages,
    refreshAuthProfile,
    importSelectedProfile,
    deleteSelectedProfile,
    saveAuthProfile,
    connectSelectedProfile,
    startBrowserOauth,
    startDeviceFlow,
  };
}
