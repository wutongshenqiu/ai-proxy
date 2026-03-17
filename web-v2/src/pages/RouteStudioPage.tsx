import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Panel } from '../components/Panel';
import { StatusPill } from '../components/StatusPill';
import { WorkbenchSheet } from '../components/WorkbenchSheet';
import { useRouteStudioData } from '../hooks/useWorkspaceData';
import { writeRouteDraft } from '../lib/routeDraft';
import { getApiErrorMessage } from '../services/errors';
import { routingApi } from '../services/routing';
import type { RouteExplanation, RouteProfile, RouteRule, RoutingConfig } from '../types/backend';

function cloneRoutingConfig(config: RoutingConfig): RoutingConfig {
  return JSON.parse(JSON.stringify(config)) as RoutingConfig;
}

function prettyJson(value: unknown) {
  return JSON.stringify(value, null, 2);
}

function parseCsv(value: string) {
  return value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
}

function parseHeaders(value: string): Record<string, string[]> | undefined {
  const lines = value
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean);

  if (lines.length === 0) {
    return undefined;
  }

  return Object.fromEntries(
    lines.map((line) => {
      const [name, rawValues = ''] = line.split(':');
      return [
        name.trim(),
        rawValues
          .split(',')
          .map((item) => item.trim())
          .filter(Boolean),
      ];
    }),
  );
}

function headersToDraft(headers?: Record<string, string[]>) {
  if (!headers || Object.keys(headers).length === 0) {
    return '';
  }

  return Object.entries(headers)
    .map(([name, values]) => `${name}: ${values.join(', ')}`)
    .join('\n');
}

function buildNewRule(profileName: string, index: number): RouteRule {
  return {
    name: `draft-rule-${index + 1}`,
    priority: index + 1,
    match: {
      models: [],
      tenants: [],
      endpoints: [],
      regions: [],
      headers: {},
    },
    'use-profile': profileName,
  };
}

function parseJsonDraft<T>(label: string, raw: string): T {
  try {
    return JSON.parse(raw) as T;
  } catch (error) {
    throw new Error(`${label} JSON is invalid: ${error instanceof Error ? error.message : 'parse failed'}`);
  }
}

function extractValidationMessage(error: unknown, fallback: string) {
  if (
    error &&
    typeof error === 'object' &&
    'response' in error &&
    error.response &&
    typeof error.response === 'object' &&
    'data' in error.response &&
    error.response.data &&
    typeof error.response.data === 'object' &&
    'details' in error.response.data &&
    Array.isArray(error.response.data.details)
  ) {
    return error.response.data.details.join('; ');
  }

  return getApiErrorMessage(error, fallback);
}

export function RouteStudioPage() {
  const { data, error, loading } = useRouteStudioData();
  const navigate = useNavigate();
  const [selectedScenarioId, setSelectedScenarioId] = useState<string | null>(null);
  const [sheetOpen, setSheetOpen] = useState(false);
  const [simulationLoading, setSimulationLoading] = useState(false);
  const [simulationError, setSimulationError] = useState<string | null>(null);
  const [simulationStatus, setSimulationStatus] = useState<string | null>(null);
  const [explanation, setExplanation] = useState<RouteExplanation | null>(null);
  const [routingConfig, setRoutingConfig] = useState<RoutingConfig | null>(null);
  const [routingDraft, setRoutingDraft] = useState<RoutingConfig | null>(null);
  const [routingLoading, setRoutingLoading] = useState(false);
  const [routingError, setRoutingError] = useState<string | null>(null);
  const [routingStatus, setRoutingStatus] = useState<string | null>(null);
  const [savingDraft, setSavingDraft] = useState(false);
  const [selectedProfileName, setSelectedProfileName] = useState<string | null>(null);
  const [selectedRuleIndex, setSelectedRuleIndex] = useState<number | null>(null);
  const [profileJsonDraft, setProfileJsonDraft] = useState('');
  const [modelResolutionDraft, setModelResolutionDraft] = useState('');

  useEffect(() => {
    setSelectedScenarioId((current) => current ?? data?.scenarios[0]?.scenario ?? null);
  }, [data]);

  useEffect(() => {
    let cancelled = false;

    void (async () => {
      setRoutingLoading(true);
      setRoutingError(null);
      try {
        const config = await routingApi.get();
        if (cancelled) {
          return;
        }
        setRoutingConfig(config);
        setRoutingDraft(cloneRoutingConfig(config));
      } catch (loadError) {
        if (cancelled) {
          return;
        }
        setRoutingError(getApiErrorMessage(loadError, 'Failed to load routing config'));
      } finally {
        if (!cancelled) {
          setRoutingLoading(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  const selectedScenario = useMemo(
    () => data?.scenarios.find((scenario) => scenario.scenario === selectedScenarioId) ?? null,
    [data, selectedScenarioId],
  );
  const profileNames = useMemo(
    () => Object.keys(routingDraft?.profiles ?? {}),
    [routingDraft],
  );
  const selectedProfile = useMemo(
    () => (selectedProfileName && routingDraft ? routingDraft.profiles[selectedProfileName] ?? null : null),
    [routingDraft, selectedProfileName],
  );
  const selectedRule = useMemo(
    () => (routingDraft && selectedRuleIndex !== null ? routingDraft.rules[selectedRuleIndex] ?? null : null),
    [routingDraft, selectedRuleIndex],
  );

  useEffect(() => {
    if (!routingDraft) {
      return;
    }
    setSelectedProfileName((current) => {
      if (current && routingDraft.profiles[current]) {
        return current;
      }
      return routingDraft['default-profile'];
    });
    setSelectedRuleIndex((current) => {
      if (routingDraft.rules.length === 0) {
        return null;
      }
      if (current !== null && current < routingDraft.rules.length) {
        return current;
      }
      return 0;
    });
  }, [routingDraft]);

  useEffect(() => {
    setProfileJsonDraft(selectedProfile ? prettyJson(selectedProfile) : '');
  }, [selectedProfile]);

  useEffect(() => {
    setModelResolutionDraft(routingDraft ? prettyJson(routingDraft['model-resolution']) : '');
  }, [routingDraft]);

  const applyDraftMutation = (mutate: (draft: RoutingConfig) => void) => {
    setRoutingDraft((current) => {
      if (!current) {
        return current;
      }
      const next = cloneRoutingConfig(current);
      mutate(next);
      return next;
    });
    setRoutingStatus(null);
    setRoutingError(null);
  };

  const handleRuleFieldUpdate = (field: keyof RouteRule, value: string) => {
    if (selectedRuleIndex === null) {
      return;
    }
    applyDraftMutation((draft) => {
      const rule = draft.rules[selectedRuleIndex];
      if (!rule) {
        return;
      }
      if (field === 'priority') {
        rule.priority = value ? Number(value) : undefined;
        return;
      }
      if (field === 'name' || field === 'use-profile') {
        rule[field] = value;
      }
    });
  };

  const handleRuleMatchUpdate = (
    field: keyof RouteRule['match'],
    value: string | boolean,
  ) => {
    if (selectedRuleIndex === null) {
      return;
    }
    applyDraftMutation((draft) => {
      const rule = draft.rules[selectedRuleIndex];
      if (!rule) {
        return;
      }
      if (field === 'stream') {
        rule.match.stream = value ? true : undefined;
        return;
      }
      if (field === 'headers') {
        const parsed = parseHeaders(String(value));
        rule.match.headers = parsed;
        return;
      }
      rule.match[field] = parseCsv(String(value));
    });
  };

  const applyAdvancedDrafts = () => {
    if (!routingDraft) {
      return null;
    }

    const next = cloneRoutingConfig(routingDraft);
    if (selectedProfileName) {
      next.profiles[selectedProfileName] = parseJsonDraft<RouteProfile>('Profile policy', profileJsonDraft);
    }
    next['model-resolution'] = parseJsonDraft<RoutingConfig['model-resolution']>(
      'Model resolution',
      modelResolutionDraft,
    );
    return next;
  };

  const simulateDraft = async () => {
    if (!selectedScenario) {
      setSimulationError('Select a scenario first.');
      setSheetOpen(true);
      return;
    }

    setSheetOpen(true);
    setSimulationLoading(true);
    setSimulationError(null);
    setSimulationStatus(null);

    try {
      const nextDraft = applyAdvancedDrafts();
      const routeExplanation = await routingApi.explain({
        model: selectedScenario.model,
        endpoint: selectedScenario.endpoint,
        source_format: selectedScenario.source_format,
        tenant_id: selectedScenario.tenant_id,
        api_key_id: selectedScenario.api_key_id,
        region: selectedScenario.region,
        stream: selectedScenario.stream,
        routing_override: nextDraft ?? undefined,
      });
      if (nextDraft) {
        setRoutingDraft(nextDraft);
      }
      setExplanation(routeExplanation);
      setSimulationStatus(`Simulated route for ${selectedScenario.scenario}.`);
    } catch (actionError) {
      setSimulationError(extractValidationMessage(actionError, 'Route simulation failed'));
    } finally {
      setSimulationLoading(false);
    }
  };

  const saveRoutingDraft = async () => {
    if (!routingDraft) {
      return;
    }

    setSavingDraft(true);
    setRoutingError(null);
    setRoutingStatus(null);
    try {
      const nextDraft = applyAdvancedDrafts();
      if (!nextDraft) {
        throw new Error('Routing draft is not ready yet.');
      }
      const result = await routingApi.update({
        'default-profile': nextDraft['default-profile'],
        profiles: nextDraft.profiles,
        rules: nextDraft.rules,
        'model-resolution': nextDraft['model-resolution'],
      });
      setRoutingConfig(cloneRoutingConfig(nextDraft));
      setRoutingDraft(cloneRoutingConfig(nextDraft));
      setRoutingStatus(result.message ?? 'Routing draft saved.');
    } catch (saveError) {
      setRoutingError(extractValidationMessage(saveError, 'Failed to save routing draft'));
    } finally {
      setSavingDraft(false);
    }
  };

  const resetRoutingDraft = () => {
    if (!routingConfig) {
      return;
    }
    const reset = cloneRoutingConfig(routingConfig);
    setRoutingDraft(reset);
    setRoutingStatus('Reset route draft to runtime truth.');
    setRoutingError(null);
  };

  const promoteToChange = () => {
    if (!selectedScenario) {
      setSimulationError('Select a scenario before promoting it.');
      return;
    }

    writeRouteDraft({
      createdAt: new Date().toISOString(),
      scenario: selectedScenario,
      explanation,
    });
    navigate('/change-studio');
  };

  return (
    <div className="workspace-grid">
      <section className="hero">
        <div>
          <p className="workspace-eyebrow">PRISM / ROUTE STUDIO</p>
          <h1>Draft routing truth before publish</h1>
          <p className="workspace-summary">
            Route Studio owns the full authoring loop: default-profile selection, rule mutation, advanced policy editing, and draft simulation before promotion.
          </p>
        </div>
        <div className="hero-actions">
          <button className="button button--primary" onClick={() => void simulateDraft()}>
            Simulate draft
          </button>
          <button className="button button--ghost" onClick={() => void saveRoutingDraft()} disabled={savingDraft || routingLoading}>
            {savingDraft ? 'Saving…' : 'Save routing draft'}
          </button>
          <button className="button button--ghost" onClick={promoteToChange}>
            Promote to change
          </button>
        </div>
      </section>

      {selectedScenario ? (
        <div className="status-message status-message--warning">
          Active scenario: <strong>{selectedScenario.scenario}</strong> · winner {selectedScenario.winner} · {selectedScenario.delta}
        </div>
      ) : null}
      {routingStatus ? <div className="status-message status-message--success">{routingStatus}</div> : null}
      {routingError ? <div className="status-message status-message--danger">{routingError}</div> : null}

      <div className="two-column">
        <Panel title="Routing summary" subtitle="Current routing truth and selected draft posture.">
          <ul className="fact-list">
            {(data?.summary_facts ?? []).map((fact) => (
              <li key={fact.label}><span>{fact.label}</span><strong>{fact.value}</strong></li>
            ))}
            {routingDraft ? (
              <>
                <li><span>Default profile</span><strong>{routingDraft['default-profile']}</strong></li>
                <li><span>Profiles</span><strong>{profileNames.length}</strong></li>
                <li><span>Rules</span><strong>{routingDraft.rules.length}</strong></li>
              </>
            ) : null}
          </ul>
        </Panel>
        <Panel title="Explain posture" subtitle="Planner behavior distilled into operator-readable facts.">
          <ul className="fact-list">
            {(data?.explain_facts ?? []).map((fact) => (
              <li key={fact.label}><span>{fact.label}</span><strong>{fact.value}</strong></li>
            ))}
          </ul>
        </Panel>
      </div>

      <div className="two-column">
        <Panel title="Routing authoring" subtitle="Default profile switching and profile policy selection stay in the main workbench.">
          {routingLoading && !routingDraft ? <div className="status-message">Loading routing draft…</div> : null}
          <div className="sheet-form">
            <label className="sheet-field">
              <span>Default profile</span>
              <select
                value={routingDraft?.['default-profile'] ?? ''}
                onChange={(event) => {
                  const value = event.target.value;
                  applyDraftMutation((draft) => {
                    draft['default-profile'] = value;
                  });
                  setSelectedProfileName(value);
                }}
                disabled={!routingDraft}
              >
                {profileNames.map((name) => (
                  <option key={name} value={name}>{name}</option>
                ))}
              </select>
            </label>
            <label className="sheet-field">
              <span>Selected profile</span>
              <select
                value={selectedProfileName ?? ''}
                onChange={(event) => setSelectedProfileName(event.target.value)}
                disabled={!routingDraft}
              >
                {profileNames.map((name) => (
                  <option key={name} value={name}>{name}</option>
                ))}
              </select>
            </label>
          </div>
          <div className="inline-actions">
            <button type="button" className="button button--ghost" onClick={resetRoutingDraft} disabled={!routingConfig}>
              Reset draft
            </button>
            <button type="button" className="button button--primary" onClick={() => void saveRoutingDraft()} disabled={savingDraft || !routingDraft}>
              {savingDraft ? 'Saving…' : 'Save routing'}
            </button>
          </div>
        </Panel>

        <Panel title="Selected rule editor" subtitle="Rule CRUD belongs here, not in raw YAML.">
          {selectedRule ? (
            <div className="sheet-form">
              <label className="sheet-field">
                <span>Rule name</span>
                <input
                  name="route-rule-name"
                  autoComplete="off"
                  value={selectedRule.name}
                  onChange={(event) => handleRuleFieldUpdate('name', event.target.value)}
                />
              </label>
              <label className="sheet-field">
                <span>Priority</span>
                <input
                  name="route-rule-priority"
                  inputMode="numeric"
                  autoComplete="off"
                  value={selectedRule.priority?.toString() ?? ''}
                  onChange={(event) => handleRuleFieldUpdate('priority', event.target.value)}
                />
              </label>
              <label className="sheet-field">
                <span>Use profile</span>
                <select
                  value={selectedRule['use-profile']}
                  onChange={(event) => handleRuleFieldUpdate('use-profile', event.target.value)}
                >
                  {profileNames.map((name) => (
                    <option key={name} value={name}>{name}</option>
                  ))}
                </select>
              </label>
              <label className="sheet-field">
                <span>Models</span>
                <input
                  name="route-rule-models"
                  autoComplete="off"
                  value={selectedRule.match.models?.join(', ') ?? ''}
                  onChange={(event) => handleRuleMatchUpdate('models', event.target.value)}
                />
              </label>
              <label className="sheet-field">
                <span>Tenants</span>
                <input
                  name="route-rule-tenants"
                  autoComplete="off"
                  value={selectedRule.match.tenants?.join(', ') ?? ''}
                  onChange={(event) => handleRuleMatchUpdate('tenants', event.target.value)}
                />
              </label>
              <label className="sheet-field">
                <span>Endpoints</span>
                <input
                  name="route-rule-endpoints"
                  autoComplete="off"
                  value={selectedRule.match.endpoints?.join(', ') ?? ''}
                  onChange={(event) => handleRuleMatchUpdate('endpoints', event.target.value)}
                />
              </label>
              <label className="sheet-field">
                <span>Regions</span>
                <input
                  name="route-rule-regions"
                  autoComplete="off"
                  value={selectedRule.match.regions?.join(', ') ?? ''}
                  onChange={(event) => handleRuleMatchUpdate('regions', event.target.value)}
                />
              </label>
              <label className="sheet-field">
                <span>Headers</span>
                <textarea
                  className="yaml-editor"
                  value={headersToDraft(selectedRule.match.headers)}
                  onChange={(event) => handleRuleMatchUpdate('headers', event.target.value)}
                />
              </label>
              <label className="detail-grid__row">
                <span>Streaming only</span>
                <input
                  type="checkbox"
                  checked={selectedRule.match.stream ?? false}
                  onChange={(event) => handleRuleMatchUpdate('stream', event.target.checked)}
                />
              </label>
            </div>
          ) : (
            <div className="status-message">Select or create a rule to begin editing.</div>
          )}
        </Panel>
      </div>

      <div className="two-column">
        <Panel title="Rule registry" subtitle="Rules can be added, selected, and removed without losing blast-radius context.">
          <div className="inline-actions">
            <button
              type="button"
              className="button button--ghost"
              onClick={() => {
                if (!routingDraft) {
                  return;
                }
                const fallbackProfile = selectedProfileName ?? routingDraft['default-profile'];
                applyDraftMutation((draft) => {
                  draft.rules.push(buildNewRule(fallbackProfile, draft.rules.length));
                });
                setSelectedRuleIndex(routingDraft.rules.length);
              }}
              disabled={!routingDraft}
            >
              New rule
            </button>
            <button
              type="button"
              className="button button--ghost"
              onClick={() => {
                if (!routingDraft || selectedRuleIndex === null) {
                  return;
                }
                applyDraftMutation((draft) => {
                  draft.rules.splice(selectedRuleIndex, 1);
                });
                setSelectedRuleIndex((current) => {
                  if (current === null) return null;
                  if (routingDraft.rules.length <= 1) return null;
                  return Math.max(0, current - 1);
                });
              }}
              disabled={!routingDraft || selectedRuleIndex === null}
            >
              Delete selected
            </button>
          </div>
          <div className="table-grid table-grid--routes">
            <div className="table-grid__head">Rule</div>
            <div className="table-grid__head">Profile</div>
            <div className="table-grid__head">Priority</div>
            <div className="table-grid__head">Matchers</div>
            {(routingDraft?.rules ?? []).flatMap((rule, index) => {
              const selected = index === selectedRuleIndex;
              const cellClass = `table-grid__cell ${selected ? 'is-selected' : ''} is-clickable`;
              const matchers = [
                rule.match.models?.length ? `${rule.match.models.length} models` : null,
                rule.match.tenants?.length ? `${rule.match.tenants.length} tenants` : null,
                rule.match.endpoints?.length ? `${rule.match.endpoints.length} endpoints` : null,
              ].filter(Boolean).join(' · ') || 'default';
              return [
                <div key={`${rule.name}-name`} className={`${cellClass} table-grid__cell--strong`} onClick={() => setSelectedRuleIndex(index)}>
                  {rule.name}
                </div>,
                <div key={`${rule.name}-profile`} className={cellClass} onClick={() => setSelectedRuleIndex(index)}>
                  {rule['use-profile']}
                </div>,
                <div key={`${rule.name}-priority`} className={cellClass} onClick={() => setSelectedRuleIndex(index)}>
                  {rule.priority ?? 'n/a'}
                </div>,
                <div key={`${rule.name}-match`} className={cellClass} onClick={() => setSelectedRuleIndex(index)}>
                  {matchers}
                </div>,
              ];
            })}
          </div>
        </Panel>

        <Panel title="Scenario matrix" subtitle="Sampled route explanations from live traffic and configured models.">
          <div className="table-grid table-grid--routes">
            <div className="table-grid__head">Scenario</div>
            <div className="table-grid__head">Winner</div>
            <div className="table-grid__head">Delta</div>
            <div className="table-grid__head">Route state</div>
            {loading && !data ? <div className="table-grid__cell">Loading scenarios…</div> : null}
            {error && !data ? <div className="table-grid__cell">{error}</div> : null}
            {(data?.scenarios ?? []).flatMap((scenario) => {
              const selected = scenario.scenario === selectedScenarioId;
              const cellClass = `table-grid__cell ${selected ? 'is-selected' : ''} is-clickable`;
              return [
                <div
                  key={`${scenario.scenario}-scenario`}
                  className={`${cellClass} table-grid__cell--strong`}
                  onClick={() => setSelectedScenarioId(scenario.scenario)}
                >
                  {scenario.scenario}
                </div>,
                <div key={`${scenario.scenario}-winner`} className={cellClass} onClick={() => setSelectedScenarioId(scenario.scenario)}>
                  {scenario.winner}
                </div>,
                <div key={`${scenario.scenario}-delta`} className={cellClass} onClick={() => setSelectedScenarioId(scenario.scenario)}>
                  {scenario.delta}
                </div>,
                <div key={`${scenario.scenario}-decision`} className={cellClass} onClick={() => setSelectedScenarioId(scenario.scenario)}>
                  <StatusPill label={scenario.decision} tone={scenario.decision_tone} />
                </div>,
              ];
            })}
          </div>
        </Panel>
      </div>

      <div className="two-column">
        <Panel title="Advanced profile policy" subtitle="Edit the selected route profile directly when the structured fields are not enough.">
          <textarea
            className="yaml-editor"
            value={profileJsonDraft}
            onChange={(event) => setProfileJsonDraft(event.target.value)}
            spellCheck={false}
          />
        </Panel>
        <Panel title="Model resolution" subtitle="Alias, rewrite, fallback, and provider pins stay explicit in the same draft.">
          <textarea
            className="yaml-editor"
            value={modelResolutionDraft}
            onChange={(event) => setModelResolutionDraft(event.target.value)}
            spellCheck={false}
          />
        </Panel>
      </div>

      <WorkbenchSheet
        open={sheetOpen}
        onClose={() => setSheetOpen(false)}
        title="Route simulation workbench"
        subtitle="Run a real explain against the selected scenario using the current local draft, then promote it into change review."
        actions={(
          <>
            <button type="button" className="button button--ghost" onClick={promoteToChange} disabled={!selectedScenario}>
              Promote to change
            </button>
            <button type="button" className="button button--primary" onClick={() => void simulateDraft()} disabled={simulationLoading}>
              {simulationLoading ? 'Simulating…' : 'Re-run simulation'}
            </button>
          </>
        )}
      >
        {simulationStatus ? <div className="status-message status-message--success">{simulationStatus}</div> : null}
        {simulationError ? <div className="status-message status-message--danger">{simulationError}</div> : null}

        {selectedScenario ? (
          <section className="sheet-section">
            <h3>Scenario posture</h3>
            <div className="detail-grid">
              <div className="detail-grid__row"><span>Scenario</span><strong>{selectedScenario.scenario}</strong></div>
              <div className="detail-grid__row"><span>Model</span><strong>{selectedScenario.model}</strong></div>
              <div className="detail-grid__row"><span>Endpoint</span><strong>{selectedScenario.endpoint}</strong></div>
              <div className="detail-grid__row"><span>Source format</span><strong>{selectedScenario.source_format}</strong></div>
            </div>
          </section>
        ) : null}

        {explanation ? (
          <>
            <section className="sheet-section">
              <h3>Winning route</h3>
              <div className="detail-grid">
                <div className="detail-grid__row"><span>Profile</span><strong>{explanation.profile}</strong></div>
                <div className="detail-grid__row"><span>Matched rule</span><strong>{explanation.matched_rule ?? 'default'}</strong></div>
                <div className="detail-grid__row"><span>Provider</span><strong>{explanation.selected?.provider ?? 'none'}</strong></div>
                <div className="detail-grid__row"><span>Credential</span><strong>{explanation.selected?.credential_name ?? 'none'}</strong></div>
              </div>
            </section>

            <section className="sheet-section">
              <h3>Alternates and rejections</h3>
              <div className="probe-list">
                {explanation.alternates.slice(0, 3).map((alternate) => (
                  <div key={`${alternate.provider}-${alternate.credential_name}`} className="probe-check">
                    <span>{alternate.provider}</span>
                    <strong>{alternate.model}</strong>
                  </div>
                ))}
                {explanation.rejections.slice(0, 3).map((rejection) => (
                  <div key={`${rejection.candidate}-${JSON.stringify(rejection.reason)}`} className="probe-check">
                    <span>{rejection.candidate}</span>
                    <strong>{typeof rejection.reason === 'string' ? rejection.reason : 'missing capability'}</strong>
                  </div>
                ))}
              </div>
            </section>
          </>
        ) : null}
      </WorkbenchSheet>
    </div>
  );
}
