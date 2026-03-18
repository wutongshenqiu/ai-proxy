import { useState, type Dispatch, type SetStateAction } from 'react';
import { useI18n } from '../../i18n';
import type {
  ProviderEditorFormState,
  ProviderRegistryFormState,
} from '../../components/provider-atlas/types';
import { confirmAction } from '../../lib/browser';
import { getApiErrorMessage } from '../../services/errors';
import { providersApi } from '../../services/providers';
import type {
  ProviderCreateRequest,
  ProviderDetail,
} from '../../types/backend';

export const emptyRegistryForm: ProviderRegistryFormState = {
  name: '',
  format: 'openai',
  upstream: 'openai',
  apiKey: '',
  baseUrl: '',
  models: '',
  disabled: true,
};

interface UseProviderAtlasRegistryWorkbenchOptions {
  selectedProvider: string | null;
  reload: () => Promise<void>;
  loadRuntimeSurfaces: () => Promise<void>;
  setSelectedProvider: Dispatch<SetStateAction<string | null>>;
  setDetail: Dispatch<SetStateAction<ProviderDetail | null>>;
}

export function useProviderAtlasRegistryWorkbench({
  selectedProvider,
  reload,
  loadRuntimeSurfaces,
  setSelectedProvider,
  setDetail,
}: UseProviderAtlasRegistryWorkbenchOptions) {
  const { t } = useI18n();
  const [registryOpen, setRegistryOpen] = useState(false);
  const [registryLoading, setRegistryLoading] = useState(false);
  const [registryStatus, setRegistryStatus] = useState<string | null>(null);
  const [registryError, setRegistryError] = useState<string | null>(null);
  const [registryForm, setRegistryForm] = useState<ProviderRegistryFormState>(emptyRegistryForm);
  const [formState, setFormState] = useState<ProviderEditorFormState>({
    baseUrl: '',
    region: '',
    weight: '1',
    disabled: false,
  });

  const openRegistryWorkbench = () => {
    setRegistryOpen(true);
    setRegistryStatus(null);
    setRegistryError(null);
    setRegistryForm(emptyRegistryForm);
  };

  const fetchModelsIntoDraft = async () => {
    if (!registryForm.apiKey.trim()) {
      setRegistryError(t('providerAtlas.registryError.apiKeyRequired'));
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
      setRegistryStatus(t('providerAtlas.registryStatus.modelsFetched', { count: result.models.length }));
    } catch (fetchError) {
      setRegistryError(getApiErrorMessage(fetchError, t('providerAtlas.registryError.fetchModels')));
    } finally {
      setRegistryLoading(false);
    }
  };

  const createProvider = async () => {
    if (!registryForm.name.trim()) {
      setRegistryError(t('providerAtlas.registryError.nameRequired'));
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
      setRegistryStatus(t('providerAtlas.registryStatus.createdProvider', { provider: body.name }));
      setSelectedProvider(body.name);
      setRegistryForm(emptyRegistryForm);
      await reload();
      await loadRuntimeSurfaces();
    } catch (createError) {
      setRegistryError(getApiErrorMessage(createError, t('providerAtlas.registryError.createProvider')));
    } finally {
      setRegistryLoading(false);
    }
  };

  const deleteSelectedProvider = async () => {
    if (!selectedProvider) {
      setRegistryError(t('providerAtlas.registryError.selectProvider'));
      return;
    }
    if (!confirmAction(t('providerAtlas.registryConfirm.deleteProvider', { provider: selectedProvider }))) {
      return;
    }

    setRegistryLoading(true);
    setRegistryError(null);
    setRegistryStatus(null);
    try {
      await providersApi.remove(selectedProvider);
      setRegistryStatus(t('providerAtlas.registryStatus.deletedProvider', { provider: selectedProvider }));
      setSelectedProvider(null);
      setDetail(null);
      await reload();
      await loadRuntimeSurfaces();
    } catch (deleteError) {
      setRegistryError(getApiErrorMessage(deleteError, t('providerAtlas.registryError.deleteProvider')));
    } finally {
      setRegistryLoading(false);
    }
  };

  return {
    registryOpen,
    setRegistryOpen,
    registryLoading,
    registryStatus,
    registryError,
    registryForm,
    setRegistryForm,
    formState,
    setFormState,
    openRegistryWorkbench,
    fetchModelsIntoDraft,
    createProvider,
    deleteSelectedProvider,
  };
}
