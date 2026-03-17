import { useCallback, useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Panel } from '../components/Panel';
import { StatusPill } from '../components/StatusPill';
import { WorkbenchSheet } from '../components/WorkbenchSheet';
import { useTrafficLabData } from '../hooks/useWorkspaceData';
import { getApiErrorMessage } from '../services/errors';
import { logsApi } from '../services/logs';
import { routingApi } from '../services/routing';
import { useShellStore } from '../stores/shellStore';
import type { RequestLog, RouteExplanation } from '../types/backend';

function endpointFromPath(path: string) {
  if (path.includes('/messages')) return 'messages';
  if (path.includes('/responses')) return 'responses';
  if (path.includes('streamGenerateContent')) return 'stream-generate-content';
  if (path.includes('generateContent')) return 'generate-content';
  return 'chat-completions';
}

function sourceFormatFromPath(path: string) {
  if (path.includes('/messages')) return 'claude';
  if (path.includes('generateContent')) return 'gemini';
  return 'openai';
}

export function TrafficLabPage() {
  const { data, error, loading } = useTrafficLabData();
  const timeRange = useShellStore((state) => state.timeRange);
  const sourceMode = useShellStore((state) => state.sourceMode);
  const live = useShellStore((state) => state.live);
  const [searchParams, setSearchParams] = useSearchParams();
  const [lensStatus, setLensStatus] = useState<string | null>(null);
  const [replayOpen, setReplayOpen] = useState(false);
  const [replayLoading, setReplayLoading] = useState(false);
  const [replayError, setReplayError] = useState<string | null>(null);
  const [requestRecord, setRequestRecord] = useState<RequestLog | null>(null);
  const [explanation, setExplanation] = useState<RouteExplanation | null>(null);
  const [detailOpen, setDetailOpen] = useState(false);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [detailRecord, setDetailRecord] = useState<RequestLog | null>(null);
  const selectedRequestId = searchParams.get('request');
  const compareRequestId = searchParams.get('compare');
  const sessionFilter = searchParams.get('q') ?? '';

  const updateSearch = useCallback((patch: Record<string, string | null | undefined>) => {
    const next = new URLSearchParams(searchParams);
    Object.entries(patch).forEach(([key, value]) => {
      if (!value) {
        next.delete(key);
      } else {
        next.set(key, value);
      }
    });
    setSearchParams(next, { replace: true });
  }, [searchParams, setSearchParams]);

  useEffect(() => {
    const fallback = data?.selected_request_id ?? data?.sessions[0]?.request_id ?? null;
    if (!selectedRequestId && fallback) {
      updateSearch({ request: fallback });
    }
  }, [data, selectedRequestId, updateSearch]);

  const visibleSessions = useMemo(() => {
    const needle = sessionFilter.trim().toLowerCase();
    if (!needle) {
      return data?.sessions ?? [];
    }
    return (data?.sessions ?? []).filter((session) => {
      const haystack = [
        session.request_id,
        session.model,
        session.decision,
        session.result,
      ].join(' ').toLowerCase();
      return haystack.includes(needle);
    });
  }, [data?.sessions, sessionFilter]);

  const selectedSession = useMemo(
    () => data?.sessions.find((session) => session.request_id === selectedRequestId) ?? null,
    [data, selectedRequestId],
  );
  const compareSession = useMemo(
    () => data?.sessions.find((session) => session.request_id === compareRequestId) ?? null,
    [compareRequestId, data],
  );

  const handleSaveLens = () => {
    const payload = {
      timeRange,
      sourceMode,
      selectedRequestId,
      compareRequestId,
      sessionFilter,
      savedAt: new Date().toISOString(),
    };
    localStorage.setItem('prism-control-plane:traffic-lens', JSON.stringify(payload));
    setLensStatus('Saved the current traffic lens to local storage.');
  };

  const handleReplay = async () => {
    if (!selectedRequestId) {
      setReplayError('Select a request session first.');
      setReplayOpen(true);
      return;
    }

    setReplayOpen(true);
    setReplayLoading(true);
    setReplayError(null);

    try {
      const record = await logsApi.getRequest(selectedRequestId);
      if (!record) {
        throw new Error('Selected request no longer exists in the log window.');
      }
      setRequestRecord(record);
      const routeExplanation = await routingApi.explain({
        model: record.requested_model ?? record.model ?? 'unknown-model',
        endpoint: endpointFromPath(record.path),
        source_format: sourceFormatFromPath(record.path),
        tenant_id: record.tenant_id,
        api_key_id: record.api_key_id,
        region: record.client_region ?? null,
        stream: record.stream,
      });
      setExplanation(routeExplanation);
    } catch (actionError) {
      setReplayError(getApiErrorMessage(actionError, 'Replay failed'));
    } finally {
      setReplayLoading(false);
    }
  };

  const openSessionDetail = async () => {
    if (!selectedRequestId) {
      setDetailError('Select a request session first.');
      setDetailOpen(true);
      return;
    }

    setDetailOpen(true);
    setDetailLoading(true);
    setDetailError(null);
    try {
      const record = await logsApi.getRequest(selectedRequestId);
      if (!record) {
        throw new Error('Selected request no longer exists in the log window.');
      }
      setDetailRecord(record);
    } catch (loadError) {
      setDetailError(getApiErrorMessage(loadError, 'Failed to load request detail'));
    } finally {
      setDetailLoading(false);
    }
  };

  return (
    <div className="workspace-grid">
      <section className="hero">
        <div>
          <p className="workspace-eyebrow">PRISM / TRAFFIC LAB</p>
          <h1>Request sessions, not log rows</h1>
          <p className="workspace-summary">
            Debugging is the shortest operator loop. This workspace keeps request selection, trace reasoning, replay, and compare in one screen.
          </p>
        </div>
        <div className="hero-actions">
          <button className="button button--primary" onClick={() => void handleReplay()}>
            Replay with draft
          </button>
          <button className="button button--ghost" onClick={() => void openSessionDetail()}>
            Inspect selected session
          </button>
          <button className="button button--ghost" onClick={handleSaveLens}>
            Save lens
          </button>
        </div>
      </section>

      {lensStatus ? <div className="status-message status-message--success">{lensStatus}</div> : null}
      {selectedSession ? (
        <div className="status-message status-message--warning">
          Active session: <strong>{selectedSession.request_id}</strong> · {selectedSession.model} · {selectedSession.latency_ms} ms
        </div>
      ) : null}

      <div className="two-column two-column--70-30">
        <Panel title="Request sessions" subtitle="URL-shareable filters, live state, and evidence-first drill-down.">
          <div className="inline-actions">
            <input
              name="traffic-session-filter"
              placeholder="Filter by request, model, or result"
              autoComplete="off"
              value={sessionFilter}
              onChange={(event) => updateSearch({ q: event.target.value || null })}
            />
          </div>
          <div className="table-grid table-grid--sessions">
            <div className="table-grid__head">Session</div>
            <div className="table-grid__head">Model</div>
            <div className="table-grid__head">Decision</div>
            <div className="table-grid__head">Result</div>
            <div className="table-grid__head">Latency</div>
            {loading && !data ? <div className="table-grid__cell">Loading sessions…</div> : null}
            {error && !data ? <div className="table-grid__cell">{error}</div> : null}
            {visibleSessions.flatMap((session) => {
              const selected = session.request_id === selectedRequestId;
              const cellClass = `table-grid__cell ${selected ? 'is-selected' : ''} is-clickable`;
              return [
                <div
                  key={`${session.request_id}-id`}
                  className={`${cellClass} table-grid__cell--strong`}
                  onClick={() => updateSearch({ request: session.request_id })}
                >
                  {session.request_id}
                </div>,
                <div key={`${session.request_id}-model`} className={cellClass} onClick={() => updateSearch({ request: session.request_id })}>
                  {session.model}
                </div>,
                <div key={`${session.request_id}-decision`} className={cellClass} onClick={() => updateSearch({ request: session.request_id })}>
                  {session.decision}
                </div>,
                <div key={`${session.request_id}-result`} className={cellClass} onClick={() => updateSearch({ request: session.request_id })}>
                  <StatusPill label={session.result} tone={session.result_tone} />
                </div>,
                <div
                  key={`${session.request_id}-latency`}
                  className={`${cellClass} table-grid__cell--mono`}
                  onClick={() => updateSearch({ request: session.request_id })}
                >
                  {session.latency_ms} ms
                </div>,
              ];
            })}
          </div>
        </Panel>

        <Panel title="Current window facts" subtitle="Range-level truth, shareable filters, and compare mode for the selected request stream.">
          <ul className="fact-list">
            {(data?.compare_facts ?? []).map((fact) => (
              <li key={fact.label}><span>{fact.label}</span><strong>{fact.value}</strong></li>
            ))}
            <li><span>Live updates</span><strong>{live ? 'connected' : 'paused'}</strong></li>
            <li><span>Filter</span><strong>{sessionFilter || 'none'}</strong></li>
          </ul>
          <div className="sheet-form">
            <label className="sheet-field">
              <span>Compare request</span>
              <select
                value={compareRequestId ?? ''}
                onChange={(event) => updateSearch({ compare: event.target.value || null })}
              >
                <option value="">none</option>
                {visibleSessions
                  .filter((session) => session.request_id !== selectedRequestId)
                  .map((session) => (
                    <option key={session.request_id} value={session.request_id}>
                      {session.request_id} · {session.model}
                    </option>
                  ))}
              </select>
            </label>
          </div>
          {selectedSession && compareSession ? (
            <div className="detail-grid">
              <div className="detail-grid__row"><span>Primary</span><strong>{selectedSession.request_id}</strong></div>
              <div className="detail-grid__row"><span>Compare</span><strong>{compareSession.request_id}</strong></div>
              <div className="detail-grid__row"><span>Latency delta</span><strong>{selectedSession.latency_ms - compareSession.latency_ms} ms</strong></div>
              <div className="detail-grid__row"><span>Result delta</span><strong>{selectedSession.result} vs {compareSession.result}</strong></div>
            </div>
          ) : null}
        </Panel>
      </div>

      <Panel title="Execution trace" subtitle="Reason as a decision timeline, not as disconnected cards.">
        <div className="timeline">
          {(data?.trace ?? []).map((step) => (
            <article key={`${step.label}-${step.title}`} className="timeline-step">
              <StatusPill label={step.label} tone={step.tone} />
              <div>
                <strong>{step.title}</strong>
                <p>{step.detail}</p>
              </div>
            </article>
          ))}
        </div>
      </Panel>

      <WorkbenchSheet
        open={replayOpen}
        onClose={() => setReplayOpen(false)}
        title="Request replay workbench"
        subtitle="Use the selected live request as the baseline for a real route explain."
      >
        {replayLoading ? <div className="status-message">Loading request and route explanation…</div> : null}
        {replayError ? <div className="status-message status-message--danger">{replayError}</div> : null}

        {requestRecord ? (
          <section className="sheet-section">
            <h3>Selected request</h3>
            <div className="detail-grid">
              <div className="detail-grid__row"><span>Request</span><strong>{requestRecord.request_id}</strong></div>
              <div className="detail-grid__row"><span>Path</span><strong>{requestRecord.path}</strong></div>
              <div className="detail-grid__row"><span>Model</span><strong>{requestRecord.requested_model ?? requestRecord.model ?? 'unknown'}</strong></div>
              <div className="detail-grid__row"><span>Status</span><strong>{requestRecord.status}</strong></div>
            </div>
          </section>
        ) : null}

        {explanation ? (
          <>
            <section className="sheet-section">
              <h3>Route explanation</h3>
              <div className="detail-grid">
                <div className="detail-grid__row"><span>Profile</span><strong>{explanation.profile}</strong></div>
                <div className="detail-grid__row"><span>Matched rule</span><strong>{explanation.matched_rule ?? 'default'}</strong></div>
                <div className="detail-grid__row"><span>Winner</span><strong>{explanation.selected?.provider ?? 'none'}</strong></div>
                <div className="detail-grid__row"><span>Credential</span><strong>{explanation.selected?.credential_name ?? 'none'}</strong></div>
              </div>
            </section>

            <section className="sheet-section">
              <h3>Rejections</h3>
              {explanation.rejections.length === 0 ? (
                <div className="status-message status-message--success">No rejected candidates for the selected request.</div>
              ) : (
                <div className="probe-list">
                  {explanation.rejections.map((rejection) => (
                    <div key={`${rejection.candidate}-${JSON.stringify(rejection.reason)}`} className="probe-check">
                      <span>{rejection.candidate}</span>
                      <strong>{typeof rejection.reason === 'string' ? rejection.reason : 'missing capability'}</strong>
                    </div>
                  ))}
                </div>
              )}
            </section>
          </>
        ) : null}
      </WorkbenchSheet>

      <WorkbenchSheet
        open={detailOpen}
        onClose={() => setDetailOpen(false)}
        title="Request session detail"
        subtitle="Inspect the raw request, upstream transformation, response body, and retry chain without leaving the traffic workspace."
      >
        {detailLoading ? <div className="status-message">Loading request detail…</div> : null}
        {detailError ? <div className="status-message status-message--danger">{detailError}</div> : null}

        {detailRecord ? (
          <>
            <section className="sheet-section">
              <h3>Request posture</h3>
              <div className="detail-grid">
                <div className="detail-grid__row"><span>Request</span><strong>{detailRecord.request_id}</strong></div>
                <div className="detail-grid__row"><span>Path</span><strong>{detailRecord.path}</strong></div>
                <div className="detail-grid__row"><span>Provider</span><strong>{detailRecord.provider ?? 'n/a'}</strong></div>
                <div className="detail-grid__row"><span>Tenant</span><strong>{detailRecord.tenant_id ?? 'global'}</strong></div>
                <div className="detail-grid__row"><span>Status</span><strong>{detailRecord.status}</strong></div>
                <div className="detail-grid__row"><span>Latency</span><strong>{detailRecord.latency_ms} ms</strong></div>
              </div>
            </section>

            <section className="sheet-section">
              <h3>Retry chain</h3>
              {detailRecord.attempts && detailRecord.attempts.length > 0 ? (
                <div className="probe-list">
                  {detailRecord.attempts.map((attempt) => (
                    <div key={`${attempt.attempt_index}-${attempt.provider}-${attempt.model}`} className="probe-check">
                      <span>{attempt.provider} / {attempt.model}</span>
                      <strong>{attempt.status ?? 'error'} · {attempt.latency_ms} ms</strong>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="status-message">No retry chain recorded for this request.</div>
              )}
            </section>

            <section className="sheet-section">
              <h3>Payloads</h3>
              <div className="code-block">
                <strong>Request body</strong>
                <pre>{detailRecord.request_body ?? 'No request body captured.'}</pre>
              </div>
              <div className="code-block">
                <strong>Upstream request</strong>
                <pre>{detailRecord.upstream_request_body ?? 'No upstream request body captured.'}</pre>
              </div>
              <div className="code-block">
                <strong>Response body</strong>
                <pre>{detailRecord.response_body ?? detailRecord.stream_content_preview ?? 'No response body captured.'}</pre>
              </div>
            </section>
          </>
        ) : null}
      </WorkbenchSheet>
    </div>
  );
}
