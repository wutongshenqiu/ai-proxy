import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { KpiCard } from '../components/KpiCard';
import { Panel } from '../components/Panel';
import { StatusPill } from '../components/StatusPill';
import { WorkbenchSheet } from '../components/WorkbenchSheet';
import { useCommandCenterData } from '../hooks/useWorkspaceData';
import { configApi } from '../services/config';
import { getApiErrorMessage } from '../services/errors';
import { systemApi } from '../services/system';
import type { SystemHealthResponse, SystemLogEntry } from '../types/backend';

function targetPath(label?: string) {
  const value = label?.toLowerCase() ?? '';
  if (value.includes('traffic')) return '/traffic-lab';
  if (value.includes('provider')) return '/provider-atlas';
  if (value.includes('route')) return '/route-studio';
  if (value.includes('change')) return '/change-studio';
  return '/command-center';
}

export function CommandCenterPage() {
  const { data, error, loading } = useCommandCenterData();
  const navigate = useNavigate();
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false);
  const [systemHealth, setSystemHealth] = useState<SystemHealthResponse | null>(null);
  const [recentLogs, setRecentLogs] = useState<SystemLogEntry[]>([]);
  const [systemError, setSystemError] = useState<string | null>(null);
  const [diagnosticLogs, setDiagnosticLogs] = useState<SystemLogEntry[]>([]);
  const [diagnosticsLoading, setDiagnosticsLoading] = useState(false);
  const [diagnosticsError, setDiagnosticsError] = useState<string | null>(null);
  const [diagnosticSearch, setDiagnosticSearch] = useState('');
  const [diagnosticLevel, setDiagnosticLevel] = useState('');
  const [diagnosticTotal, setDiagnosticTotal] = useState(0);
  const [diagnosticFile, setDiagnosticFile] = useState<string | null>(null);
  const [diagnosticTruncated, setDiagnosticTruncated] = useState(false);
  const [repairing, setRepairing] = useState(false);
  const [repairStatus, setRepairStatus] = useState<string | null>(null);
  const firstSignal = data?.signals[0] ?? null;
  const investigationSignal = useMemo(
    () => data?.signals.find((signal) => targetPath(signal.target_workspace) !== '/command-center') ?? firstSignal,
    [data?.signals, firstSignal],
  );
  const quickActions = useMemo(
    () => [
      { label: 'Open live investigation', path: '/traffic-lab' },
      { label: 'Inspect provider roster', path: '/provider-atlas' },
      { label: 'Review route draft', path: '/route-studio' },
      { label: 'Open structured change', path: '/change-studio' },
    ],
    [],
  );

  useEffect(() => {
    void (async () => {
      try {
        const [health, logs] = await Promise.all([
          systemApi.health(),
          systemApi.logs({ page: 1, page_size: 3 }),
        ]);
        setSystemHealth(health);
        setRecentLogs(logs.logs);
      } catch (loadError) {
        setSystemError(getApiErrorMessage(loadError, 'Failed to load system watch'));
      }
    })();
  }, []);

  const loadDiagnostics = async (search = diagnosticSearch, level = diagnosticLevel) => {
    setDiagnosticsLoading(true);
    setDiagnosticsError(null);
    try {
      const response = await systemApi.logs({
        page: 1,
        page_size: 25,
        search: search || undefined,
        level: level || undefined,
      });
      setDiagnosticLogs(response.logs);
      setDiagnosticTotal(response.total);
      setDiagnosticFile(response.file ?? null);
      setDiagnosticTruncated(response.truncated ?? false);
    } catch (loadError) {
      setDiagnosticsError(getApiErrorMessage(loadError, 'Failed to load diagnostics logs'));
    } finally {
      setDiagnosticsLoading(false);
    }
  };

  const reloadRuntime = async () => {
    setRepairing(true);
    setRepairStatus(null);
    setDiagnosticsError(null);
    try {
      const result = await configApi.reload();
      setRepairStatus(result.message);
      const [health, logs] = await Promise.all([
        systemApi.health(),
        systemApi.logs({ page: 1, page_size: 3 }),
      ]);
      setSystemHealth(health);
      setRecentLogs(logs.logs);
      await loadDiagnostics();
    } catch (reloadError) {
      setDiagnosticsError(getApiErrorMessage(reloadError, 'Failed to reload runtime'));
    } finally {
      setRepairing(false);
    }
  };

  return (
    <div className="workspace-grid workspace-grid--command">
      <section className="hero">
        <div>
          <p className="workspace-eyebrow">PRISM / COMMAND CENTER</p>
          <h1>Runtime posture before navigation</h1>
          <p className="workspace-summary">
            The home workspace is not a KPI wall. It is the place where operators decide what requires action now.
          </p>
        </div>
        <div className="hero-actions">
          <button
            className="button button--primary"
            onClick={() => navigate(targetPath(investigationSignal?.target_workspace))}
          >
            Open investigation
          </button>
          <button className="button button--ghost" onClick={() => {
            setDiagnosticsOpen(true);
            void loadDiagnostics();
          }}>
            Diagnostics
          </button>
          <button className="button button--ghost" onClick={() => setPaletteOpen(true)}>
            Command palette
          </button>
        </div>
      </section>

      <section className="kpi-strip">
        {(data?.kpis ?? []).map((metric) => (
          <KpiCard key={metric.label} label={metric.label} value={metric.value} delta={metric.delta} />
        ))}
      </section>

      <Panel title="Urgent signal queue" subtitle="Signals should lead, not get buried under summary cards." className="panel--wide">
        <div className="signal-list">
          {loading && !data ? <p>Loading runtime signals…</p> : null}
          {error && !data ? <p>{error}</p> : null}
          {(data?.signals ?? []).map((signal) => (
            <article
              key={signal.id}
              className="signal-row signal-row--interactive"
              onClick={() => navigate(targetPath(signal.target_workspace))}
            >
              <div>
                <strong>{signal.title}</strong>
                <p>{signal.detail}</p>
              </div>
              <div className="signal-row__meta">
                <StatusPill
                  label={signal.severity}
                  tone={signal.severity_tone}
                />
                <span>{signal.target_workspace}</span>
              </div>
            </article>
          ))}
        </div>
      </Panel>

      <div className="two-column">
        <Panel title="Pressure map" subtitle="Source freshness, change load, and provider stress in one stack.">
          <ul className="fact-list">
            {(data?.pressure_map ?? []).map((fact) => (
              <li key={fact.label}><span>{fact.label}</span><strong>{fact.value}</strong></li>
            ))}
          </ul>
        </Panel>
        <Panel title="Watch windows" subtitle="Configuration and runtime truth stay visible until operators close the loop.">
          <ul className="fact-list">
            {(data?.watch_windows ?? []).map((fact) => (
              <li key={fact.label}><span>{fact.label}</span><strong>{fact.value}</strong></li>
            ))}
          </ul>
        </Panel>
      </div>

      <div className="two-column">
        <Panel title="System watch" subtitle="Health, uptime, and runtime posture should be visible without leaving the control center.">
          {systemError ? <div className="status-message status-message--danger">{systemError}</div> : null}
          {systemHealth ? (
            <ul className="fact-list">
              <li><span>Status</span><strong>{systemHealth.status}</strong></li>
              <li><span>Version</span><strong>{systemHealth.version}</strong></li>
              <li><span>Uptime</span><strong>{systemHealth.uptime_seconds}s</strong></li>
              <li><span>Providers</span><strong>{systemHealth.providers.length}</strong></li>
            </ul>
          ) : (
            <div className="status-message">Loading system posture…</div>
          )}
        </Panel>

        <Panel title="Recent logs" subtitle="Operators should see the most recent runtime events before jumping into a deeper drill-down.">
          <div className="inline-actions">
            <button type="button" className="button button--ghost" onClick={() => {
              setDiagnosticsOpen(true);
              void loadDiagnostics();
            }}>
              Open diagnostics
            </button>
          </div>
          {recentLogs.length === 0 ? (
            <div className="status-message">No file-backed system logs are available right now.</div>
          ) : (
            <div className="probe-list">
              {recentLogs.map((entry, index) => (
                <div key={`${entry.timestamp}-${index}`} className="probe-check">
                  <span>{entry.level}</span>
                  <strong>{entry.message}</strong>
                </div>
              ))}
            </div>
          )}
        </Panel>
      </div>

      <WorkbenchSheet
        open={paletteOpen}
        onClose={() => setPaletteOpen(false)}
        title="Command palette"
        subtitle="Jump to the next operator workflow without losing shell context."
      >
        <section className="sheet-section">
          <h3>Quick actions</h3>
          <div className="action-stack">
            {quickActions.map((action) => (
              <button
                key={action.label}
                type="button"
                className="button button--secondary button--block"
                onClick={() => {
                  setPaletteOpen(false);
                  navigate(action.path);
                }}
              >
                {action.label}
              </button>
            ))}
          </div>
        </section>

        {firstSignal ? (
          <section className="sheet-section">
            <h3>Top live signal</h3>
            <div className="detail-grid">
              <div className="detail-grid__row"><span>Title</span><strong>{firstSignal.title}</strong></div>
              <div className="detail-grid__row"><span>Workspace</span><strong>{firstSignal.target_workspace}</strong></div>
              <div className="detail-grid__row"><span>Severity</span><strong>{firstSignal.severity}</strong></div>
            </div>
          </section>
        ) : null}
      </WorkbenchSheet>

      <WorkbenchSheet
        open={diagnosticsOpen}
        onClose={() => setDiagnosticsOpen(false)}
        title="Diagnostics workbench"
        subtitle="Search runtime logs, inspect file freshness, and run repair actions without leaving Command Center."
        actions={(
          <>
            <button type="button" className="button button--ghost" onClick={() => void loadDiagnostics()} disabled={diagnosticsLoading}>
              {diagnosticsLoading ? 'Loading…' : 'Refresh logs'}
            </button>
            <button type="button" className="button button--primary" onClick={() => void reloadRuntime()} disabled={repairing}>
              {repairing ? 'Reloading…' : 'Reload runtime'}
            </button>
          </>
        )}
      >
        {repairStatus ? <div className="status-message status-message--success">{repairStatus}</div> : null}
        {diagnosticsError ? <div className="status-message status-message--danger">{diagnosticsError}</div> : null}
        <section className="sheet-section">
          <h3>Log search</h3>
          <div className="sheet-form">
            <label className="sheet-field">
              <span>Search</span>
              <input
                name="diagnostic-log-search"
                autoComplete="off"
                value={diagnosticSearch}
                onChange={(event) => setDiagnosticSearch(event.target.value)}
              />
            </label>
            <label className="sheet-field">
              <span>Level</span>
              <select value={diagnosticLevel} onChange={(event) => setDiagnosticLevel(event.target.value)}>
                <option value="">all</option>
                <option value="ERROR">ERROR</option>
                <option value="WARN">WARN</option>
                <option value="INFO">INFO</option>
                <option value="DEBUG">DEBUG</option>
              </select>
            </label>
          </div>
          <div className="inline-actions">
            <button type="button" className="button button--ghost" onClick={() => void loadDiagnostics(diagnosticSearch, diagnosticLevel)}>
              Apply filters
            </button>
          </div>
          <div className="detail-grid">
            <div className="detail-grid__row"><span>Total hits</span><strong>{diagnosticTotal}</strong></div>
            <div className="detail-grid__row"><span>File</span><strong>{diagnosticFile ?? 'not available'}</strong></div>
            <div className="detail-grid__row"><span>Truncated tail</span><strong>{diagnosticTruncated ? 'yes' : 'no'}</strong></div>
          </div>
        </section>

        <section className="sheet-section">
          <h3>Matching log lines</h3>
          {diagnosticsLoading ? <div className="status-message">Loading diagnostics logs…</div> : null}
          {diagnosticLogs.length === 0 && !diagnosticsLoading ? (
            <div className="status-message">No log lines matched the current filters.</div>
          ) : (
            <div className="probe-list">
              {diagnosticLogs.map((entry, index) => (
                <div key={`${entry.timestamp}-${entry.level}-${index}`} className="probe-check">
                  <span>{entry.level} · {entry.target || 'runtime'}</span>
                  <strong>{entry.message}</strong>
                </div>
              ))}
            </div>
          )}
        </section>
      </WorkbenchSheet>
    </div>
  );
}
