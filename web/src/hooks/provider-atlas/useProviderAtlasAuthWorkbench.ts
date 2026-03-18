import type { Dispatch, SetStateAction } from 'react';
import { useI18n } from '../../i18n';
import { getApiErrorMessage } from '../../services/errors';
import type {
  AuthProfilesRuntimeResponse,
  ProviderDetail,
} from '../../types/backend';
import type { ProviderAtlasResponse } from '../../types/controlPlane';
import { useProviderAtlasAuthActions } from './useProviderAtlasAuthActions';
import { useProviderAtlasAuthSelection } from './useProviderAtlasAuthSelection';
import { useProviderAtlasDeviceFlow } from './useProviderAtlasDeviceFlow';

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
  const { t } = useI18n();
  const selection = useProviderAtlasAuthSelection({
    providers,
    selectedProvider,
    setRuntimeInfo,
  });

  const {
    authWorkbenchOpen,
    setAuthWorkbenchOpen,
    authLoading,
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
    setDeviceFlow,
    loadProfiles,
    openAuthWorkbench: openSelectionWorkbench,
    startNewAuthProfileDraft: resetAuthDraft,
  } = selection;

  const actions = useProviderAtlasAuthActions({
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
  });

  const {
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
  } = actions;

  useProviderAtlasDeviceFlow({
    deviceFlow,
    loadProfiles,
    reload,
    setDeviceFlow,
    setAuthStatus,
    setAuthError,
  });

  const openAuthWorkbench = async () => {
    clearAuthMessages();
    try {
      await openSelectionWorkbench();
    } catch (loadError) {
      setAuthError(getApiErrorMessage(loadError, t('providerAtlas.authError.loadProfiles')));
    }
  };

  const startNewAuthProfileDraft = () => {
    clearAuthMessages();
    resetAuthDraft();
  };

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
