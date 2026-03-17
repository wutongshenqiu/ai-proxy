import { useEffect, type Dispatch, type SetStateAction } from 'react';
import { resolveDeviceFlowProfileLabel, type DeviceFlowState } from '../../lib/authProfileDraft';
import { authProfilesApi } from '../../services/authProfiles';
import { getApiErrorMessage } from '../../services/errors';

interface UseProviderAtlasDeviceFlowOptions {
  deviceFlow: DeviceFlowState | null;
  loadProfiles: () => Promise<unknown>;
  reload: () => Promise<void>;
  setDeviceFlow: Dispatch<SetStateAction<DeviceFlowState | null>>;
  setAuthStatus: Dispatch<SetStateAction<string | null>>;
  setAuthError: Dispatch<SetStateAction<string | null>>;
}

export function useProviderAtlasDeviceFlow({
  deviceFlow,
  loadProfiles,
  reload,
  setDeviceFlow,
  setAuthStatus,
  setAuthError,
}: UseProviderAtlasDeviceFlowOptions) {
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
  }, [deviceFlow, loadProfiles, reload, setAuthError, setAuthStatus, setDeviceFlow]);
}
