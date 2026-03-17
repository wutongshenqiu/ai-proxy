import {
  AccessControlSheet,
  ChangeEditorSheet,
} from '../components/change-studio/ChangeStudioSheets';
import { ChangeStudioOverview } from '../components/change-studio/ChangeStudioOverview';
import { useChangeStudioController } from '../hooks/useChangeStudioController';
import { useChangeStudioData } from '../hooks/useWorkspaceData';

export function ChangeStudioPage() {
  const { data, error, loading, reload } = useChangeStudioData();
  const controller = useChangeStudioController({ data, reload });

  return (
    <div className="workspace-grid">
      <section className="hero">
        <div>
          <p className="workspace-eyebrow">PRISM / CHANGE STUDIO</p>
          <h1>Registry, structured edit, publish, observe</h1>
          <p className="workspace-summary">
            Change management is richer than validate and apply. The workspace is designed
            around object discovery, structured editing, staged publish, and watch windows.
          </p>
        </div>
        <div className="hero-actions">
          <button
            className="button button--primary"
            onClick={() => void controller.loadEditor('structured')}
          >
            Create structured change
          </button>
          <button
            className="button button--ghost"
            onClick={() => void controller.loadEditor('yaml')}
          >
            Open YAML escape hatch
          </button>
        </div>
      </section>

      <ChangeStudioOverview
        loading={loading}
        error={error}
        data={data}
        selectedFamily={controller.selectedFamily}
        selectedRegistry={controller.selectedRegistry}
        authKeys={controller.authKeys}
        selectedAuthKeyId={controller.selectedAuthKeyId}
        tenants={controller.tenants}
        selectedTenantId={controller.selectedTenantId}
        tenantMetrics={controller.tenantMetrics}
        tenantLoading={controller.tenantLoading}
        tenantError={controller.tenantError}
        refreshingAccess={controller.refreshingAccess}
        onSelectFamily={controller.setSelectedFamily}
        onOpenAccessWorkbench={() => void controller.openAccessWorkbench()}
        onSelectAuthKey={controller.setSelectedAuthKeyId}
        onRefreshAccessPosture={() => void controller.refreshAccessPosture()}
        onSelectTenant={(tenantId) => void controller.loadTenantMetrics(tenantId)}
      />

      <ChangeEditorSheet
        open={controller.editorOpen}
        editorMode={controller.editorMode}
        loadingEditor={controller.loadingEditor}
        actionStatus={controller.actionStatus}
        actionError={controller.actionError}
        validating={controller.validating}
        reloading={controller.reloading}
        applying={controller.applying}
        yaml={controller.yaml}
        configPath={controller.configPath}
        configVersion={controller.configVersion}
        selectedRegistry={controller.selectedRegistry}
        routeDraft={controller.routeDraft}
        validationResult={controller.validationResult}
        applyResult={controller.applyResult}
        onClose={() => controller.setEditorOpen(false)}
        onValidate={() => void controller.validateDraft()}
        onReloadRuntime={() => void controller.reloadRuntime()}
        onApply={() => void controller.applyDraft()}
        onYamlChange={controller.setYaml}
        onDiscardRouteDraft={controller.discardRouteDraft}
      />

      <AccessControlSheet
        open={controller.accessOpen}
        accessEditorMode={controller.accessEditorMode}
        accessStatus={controller.accessStatus}
        accessError={controller.accessError}
        revealedKey={controller.revealedKey}
        revealingKey={controller.revealingKey}
        deletingKey={controller.deletingKey}
        savingKey={controller.savingKey}
        accessForm={controller.accessForm}
        selectedAuthKey={controller.selectedAuthKey}
        authKeys={controller.authKeys}
        selectedAuthKeyId={controller.selectedAuthKeyId}
        onClose={() => controller.setAccessOpen(false)}
        onStartNewDraft={controller.startNewAccessDraft}
        onRevealSelected={() => void controller.revealAuthKey()}
        onDeleteSelected={() => void controller.deleteAuthKey()}
        onSaveKey={() => void controller.saveAuthKey()}
        onAccessFormChange={(patch) =>
          controller.setAccessForm((current) => ({ ...current, ...patch }))
        }
        onSelectAuthKey={(authKeyId) => {
          controller.setSelectedAuthKeyId(authKeyId);
        }}
      />
    </div>
  );
}
