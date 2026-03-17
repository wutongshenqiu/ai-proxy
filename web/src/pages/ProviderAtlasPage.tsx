import { AuthProfileWorkbenchSheet } from '../components/provider-atlas/AuthProfileWorkbenchSheet';
import { ProviderAtlasOverview } from '../components/provider-atlas/ProviderAtlasOverview';
import { ProviderEditorSheet } from '../components/provider-atlas/ProviderEditorSheet';
import { ProviderRegistrySheet } from '../components/provider-atlas/ProviderRegistrySheet';
import { useProviderAtlasController } from '../hooks/useProviderAtlasController';
import { useProviderAtlasData } from '../hooks/useWorkspaceData';

export function ProviderAtlasPage() {
  const { data, error, loading, reload } = useProviderAtlasData();
  const controller = useProviderAtlasController({ data, reload });

  return (
    <div className="workspace-grid">
      <section className="hero">
        <div>
          <p className="workspace-eyebrow">PRISM / PROVIDER ATLAS</p>
          <h1>Runtime entities with identity and auth posture</h1>
          <p className="workspace-summary">
            Provider management should feel like runtime operations, not static CRUD. Coverage,
            auth state, protocol exposure, and routing participation stay visible together.
          </p>
        </div>
        <div className="hero-actions">
          <button className="button button--primary" onClick={() => void controller.openEditor()}>
            Open provider editor
          </button>
          <button
            className="button button--ghost"
            onClick={() => void controller.openAuthWorkbench()}
          >
            Auth profile workbench
          </button>
        </div>
      </section>

      <ProviderAtlasOverview
        loading={loading}
        error={error}
        data={data}
        selectedProvider={controller.selectedProvider}
        selectedRow={controller.selectedRow}
        selectedCapabilities={controller.selectedCapabilities}
        protocolFacts={controller.protocolFacts}
        filteredProtocolCoverage={controller.filteredProtocolCoverage}
        filteredModelInventory={controller.filteredModelInventory}
        protocolSearch={controller.protocolSearch}
        modelSearch={controller.modelSearch}
        onSelectProvider={controller.setSelectedProvider}
        onProtocolSearchChange={controller.setProtocolSearch}
        onModelSearchChange={controller.setModelSearch}
        onOpenRegistryWorkbench={controller.openRegistryWorkbench}
      />

      <ProviderEditorSheet
        open={controller.editorOpen}
        loadingDetail={controller.loadingDetail}
        actionStatus={controller.actionStatus}
        actionError={controller.actionError}
        detail={controller.detail}
        runtimeInfo={controller.runtimeInfo}
        health={controller.health}
        preview={controller.preview}
        previewing={controller.previewing}
        saving={controller.saving}
        selectedCapabilities={controller.selectedCapabilities}
        formState={controller.formState}
        refreshingProfileId={controller.refreshingProfileId}
        onClose={() => controller.setEditorOpen(false)}
        onRunHealthCheck={() => void controller.runHealthCheck()}
        onRunPresentationPreview={() => void controller.runPresentationPreview()}
        onSaveProvider={() => void controller.saveProvider()}
        onFormStateChange={(patch) =>
          controller.setFormState((current) => ({ ...current, ...patch }))
        }
        onRefreshAuthProfile={(provider, profileId) =>
          void controller.refreshAuthProfile(provider, profileId)
        }
      />

      <ProviderRegistrySheet
        open={controller.registryOpen}
        registryStatus={controller.registryStatus}
        registryError={controller.registryError}
        registryLoading={controller.registryLoading}
        registryForm={controller.registryForm}
        selectedProvider={controller.selectedProvider}
        selectedRow={controller.selectedRow}
        selectedProbeStatus={controller.selectedCapabilities?.probe_status ?? null}
        onClose={() => controller.setRegistryOpen(false)}
        onRegistryFormChange={(patch) =>
          controller.setRegistryForm((current) => ({ ...current, ...patch }))
        }
        onFetchModels={() => void controller.fetchModelsIntoDraft()}
        onDeleteSelectedProvider={() => void controller.deleteSelectedProvider()}
        onCreateProvider={() => void controller.createProvider()}
      />

      <AuthProfileWorkbenchSheet
        open={controller.authWorkbenchOpen}
        authLoading={controller.authLoading}
        authStatus={controller.authStatus}
        authError={controller.authError}
        authSaving={controller.authSaving}
        authEditorMode={controller.authEditorMode}
        runtimeInfo={controller.runtimeInfo}
        providers={controller.providers}
        authForm={controller.authForm}
        selectedAuthProfile={controller.selectedAuthProfile}
        selectedProfiles={controller.selectedProfiles}
        selectedAuthProfileId={controller.selectedAuthProfileId}
        selectedAuthProfileMode={controller.selectedAuthProfileMode}
        connectSecret={controller.connectSecret}
        importPath={controller.importPath}
        deviceFlow={controller.deviceFlow}
        importingProfileId={controller.importingProfileId}
        refreshingProfileId={controller.refreshingProfileId}
        connectingProfileId={controller.connectingProfileId}
        onClose={() => controller.setAuthWorkbenchOpen(false)}
        onStartNewDraft={controller.startNewAuthProfileDraft}
        onImportSelectedProfile={() => void controller.importSelectedProfile()}
        onStartBrowserOauth={() => void controller.startBrowserOauth()}
        onStartDeviceFlow={() => void controller.startDeviceFlow()}
        onRefreshSelectedProfile={() =>
          void controller.refreshAuthProfile(
            controller.selectedAuthProfile?.provider ?? '',
            controller.selectedAuthProfile?.id ?? '',
          )
        }
        onDeleteSelectedProfile={() => void controller.deleteSelectedProfile()}
        onSaveAuthProfile={() => void controller.saveAuthProfile()}
        onConnectSelectedProfile={() => void controller.connectSelectedProfile()}
        onAuthFormChange={(patch) =>
          controller.setAuthForm((current) => ({ ...current, ...patch }))
        }
        onConnectSecretChange={controller.setConnectSecret}
        onImportPathChange={controller.setImportPath}
        onSelectExistingProfile={(currentKey) => {
          controller.setSelectedAuthProfileId(currentKey);
        }}
      />
    </div>
  );
}
