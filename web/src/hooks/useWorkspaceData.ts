import { useCallback, useEffect, useState } from 'react';
import { useI18n } from '../i18n';
import { controlPlaneApi } from '../services/controlPlane';
import { getApiErrorMessage, isAbortLikeError } from '../services/errors';
import { useShellStore } from '../stores/shellStore';
import type {
  ChangeStudioResponse,
  CommandCenterResponse,
  ProviderAtlasResponse,
  RouteStudioResponse,
  TrafficLabResponse,
  WorkspaceQuery,
} from '../types/controlPlane';
import type { WorkspaceId } from '../types/shell';

interface WorkspaceState<T> {
  data: T | null;
  error: string | null;
  loading: boolean;
  reload: () => Promise<void>;
}

type Fetcher<T> = (query: WorkspaceQuery, signal?: AbortSignal) => Promise<T>;

function useWorkspaceResource<T extends { inspector: import('../types/shell').ShellInspectorState }>(
  workspaceId: WorkspaceId,
  fetcher: Fetcher<T>,
): WorkspaceState<T> {
  const { t } = useI18n();
  const timeRange = useShellStore((state) => state.timeRange);
  const sourceMode = useShellStore((state) => state.sourceMode);
  const live = useShellStore((state) => state.live);
  const setInspector = useShellStore((state) => state.setInspector);
  const [state, setState] = useState<WorkspaceState<T>>({
    data: null,
    error: null,
    loading: true,
    reload: async () => {},
  });

  const load = useCallback(
    async (silent = false, signal?: AbortSignal) => {
      if (!silent) {
        setState((current) => ({
          ...current,
          data: current.data,
          error: null,
          loading: current.data === null,
        }));
      }

      try {
        const data = await fetcher({ range: timeRange, sourceMode }, signal);
        if (signal?.aborted) {
          return;
        }
        setInspector(workspaceId, data.inspector);
        setState((current) => ({
          ...current,
          data,
          error: null,
          loading: false,
        }));
      } catch (error) {
        if (isAbortLikeError(error, signal)) {
          return;
        }

        const message = getApiErrorMessage(error, t('workspace.error.load'));
        setState((current) => ({
          ...current,
          data: current.data,
          error: message,
          loading: current.data === null,
        }));
      }
    },
    [fetcher, setInspector, sourceMode, t, timeRange, workspaceId],
  );

  useEffect(() => {
    let active = true;
    let intervalId: number | undefined;
    let inFlight: AbortController | null = null;

    const runLoad = async (silent = false) => {
      inFlight?.abort();
      const controller = new AbortController();
      inFlight = controller;
      try {
        await load(silent, controller.signal);
      } finally {
        if (active && inFlight === controller) {
          inFlight = null;
        }
      }
    };

    setState((current) => ({
      ...current,
      reload: async () => {
        await runLoad(false);
      },
    }));

    void runLoad();
    if (live) {
      intervalId = window.setInterval(() => {
        void runLoad(true);
      }, 10_000);
    }

    return () => {
      active = false;
      inFlight?.abort();
      if (intervalId !== undefined) {
        window.clearInterval(intervalId);
      }
    };
  }, [live, load]);

  return state;
}

export function useCommandCenterData() {
  return useWorkspaceResource<CommandCenterResponse>('command-center', controlPlaneApi.commandCenter);
}

export function useTrafficLabData() {
  return useWorkspaceResource<TrafficLabResponse>('traffic-lab', controlPlaneApi.trafficLab);
}

export function useProviderAtlasData() {
  return useWorkspaceResource<ProviderAtlasResponse>('provider-atlas', controlPlaneApi.providerAtlas);
}

export function useRouteStudioData() {
  return useWorkspaceResource<RouteStudioResponse>('route-studio', controlPlaneApi.routeStudio);
}

export function useChangeStudioData() {
  return useWorkspaceResource<ChangeStudioResponse>('change-studio', controlPlaneApi.changeStudio);
}
