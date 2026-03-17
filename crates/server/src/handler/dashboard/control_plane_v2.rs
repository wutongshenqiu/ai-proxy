use std::collections::{BTreeSet, HashSet};

use axum::{
    Json,
    extract::{Query, State},
};
use chrono::{DateTime, Duration, Utc};
use prism_core::{
    auth_profile::{AuthMode, AuthProfileEntry},
    config::{Config, ProviderKeyEntry},
    request_log::{LogQuery, SortField, SortOrder, StatsQuery},
    request_record::{AttemptSummary, RequestRecord},
    routing::{
        explain::explain,
        planner::RoutePlanner,
        types::{RouteEndpoint, RouteExplanation, RouteRequestFeatures},
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    handler::dashboard::{
        config_tx,
        providers::{ProbeStatus, cached_probe_result},
    },
};

const DEFAULT_TRAFFIC_LIMIT: usize = 12;

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceQuery {
    #[serde(default = "default_range")]
    pub range: String,
    #[serde(default = "default_source_mode")]
    pub source_mode: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_range() -> String {
    "1h".to_string()
}

fn default_source_mode() -> String {
    "hybrid".to_string()
}

fn default_limit() -> usize {
    DEFAULT_TRAFFIC_LIMIT
}

#[derive(Debug, Clone, Serialize)]
pub struct FactRow {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectorRow {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectorSection {
    pub title: String,
    pub rows: Vec<InspectorRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceInspector {
    pub eyebrow: String,
    pub title: String,
    pub summary: String,
    pub sections: Vec<InspectorSection>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KpiMetric {
    pub label: String,
    pub value: String,
    pub delta: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalItem {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub severity: String,
    pub severity_tone: String,
    pub target_workspace: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandCenterResponse {
    pub kpis: Vec<KpiMetric>,
    pub signals: Vec<SignalItem>,
    pub pressure_map: Vec<FactRow>,
    pub watch_windows: Vec<FactRow>,
    pub inspector: WorkspaceInspector,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrafficSessionItem {
    pub request_id: String,
    pub model: String,
    pub decision: String,
    pub result: String,
    pub result_tone: String,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimelineStep {
    pub label: String,
    pub tone: String,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrafficLabResponse {
    pub selected_request_id: Option<String>,
    pub sessions: Vec<TrafficSessionItem>,
    pub compare_facts: Vec<FactRow>,
    pub trace: Vec<TimelineStep>,
    pub inspector: WorkspaceInspector,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderAtlasRow {
    pub provider: String,
    pub format: String,
    pub auth: String,
    pub status: String,
    pub status_tone: String,
    pub rotation: String,
    pub region: String,
    pub wire_api: String,
    pub model_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderAtlasResponse {
    pub providers: Vec<ProviderAtlasRow>,
    pub coverage: Vec<FactRow>,
    pub inspector: WorkspaceInspector,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteScenarioRow {
    pub scenario: String,
    pub winner: String,
    pub delta: String,
    pub decision: String,
    pub decision_tone: String,
    pub endpoint: String,
    pub source_format: String,
    pub stream: bool,
    pub model: String,
    pub tenant_id: Option<String>,
    pub api_key_id: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteStudioResponse {
    pub summary_facts: Vec<FactRow>,
    pub explain_facts: Vec<FactRow>,
    pub scenarios: Vec<RouteScenarioRow>,
    pub inspector: WorkspaceInspector,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryRow {
    pub family: String,
    pub record: String,
    pub state: String,
    pub state_tone: String,
    pub dependents: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeStudioResponse {
    pub registry: Vec<RegistryRow>,
    pub publish_facts: Vec<FactRow>,
    pub inspector: WorkspaceInspector,
}

pub async fn command_center(
    State(state): State<AppState>,
    Query(query): Query<WorkspaceQuery>,
) -> Json<CommandCenterResponse> {
    let from = range_start_timestamp(&query.range);
    let stats = state
        .log_store
        .stats(&StatsQuery {
            from: Some(from),
            ..Default::default()
        })
        .await;
    let recent = query_recent_logs(&state, from, 24).await;
    let config = state.config.load();
    let active_providers = config
        .providers
        .iter()
        .filter(|provider| !provider.disabled)
        .count();
    let degraded_providers = config
        .providers
        .iter()
        .filter(|provider| provider_runtime_status(&state, provider).0 != "healthy")
        .count();
    let fallback_count = recent
        .iter()
        .filter(|record| record.total_attempts > 1)
        .count();
    let fallback_rate = percentage(fallback_count, recent.len());
    let freshness = latest_freshness(&recent)
        .map(|seconds| format!("{seconds}s"))
        .unwrap_or_else(|| "n/a".to_string());

    let mut signals = build_signal_items(&state, &config, &recent, &stats);
    if signals.is_empty() {
        signals.push(SignalItem {
            id: "signal-runtime-stable".to_string(),
            title: "Runtime posture is stable".to_string(),
            detail: "No degraded providers or recent request errors were detected in the selected window.".to_string(),
            severity: "healthy".to_string(),
            severity_tone: "success".to_string(),
            target_workspace: "Command Center".to_string(),
        });
    }

    let inspector = inspector_from_signal(&signals[0], &query, &recent);
    let kpis = vec![
        KpiMetric {
            label: "Signals".to_string(),
            value: signals.len().to_string(),
            delta: format!("{degraded_providers} providers require follow-up"),
        },
        KpiMetric {
            label: "Fallback rate".to_string(),
            value: fallback_rate,
            delta: format!(
                "{fallback_count} of {} recent requests retried",
                recent.len()
            ),
        },
        KpiMetric {
            label: "Ingest freshness".to_string(),
            value: freshness,
            delta: format!("source mode {}", query.source_mode),
        },
        KpiMetric {
            label: "Active providers".to_string(),
            value: active_providers.to_string(),
            delta: format!("{} configured total", config.providers.len()),
        },
    ];

    let pressure_map = vec![
        FactRow {
            label: "Providers under watch".to_string(),
            value: degraded_providers.to_string(),
        },
        FactRow {
            label: "Recent request errors".to_string(),
            value: stats.error_count.to_string(),
        },
        FactRow {
            label: "Tracked tenants".to_string(),
            value: state
                .metrics
                .tenant_snapshot()
                .as_object()
                .map(|value| value.len())
                .unwrap_or_default()
                .to_string(),
        },
        FactRow {
            label: "Auth profiles".to_string(),
            value: config
                .providers
                .iter()
                .map(|provider| provider.expanded_auth_profiles().len())
                .sum::<usize>()
                .to_string(),
        },
    ];

    let config_version = config_tx::read_config_versioned(&state)
        .map(|(_, version)| version)
        .unwrap_or_else(|_| "unavailable".to_string());
    let top_error = stats
        .top_errors
        .first()
        .map(|entry| format!("{} ({})", entry.error_type, entry.count))
        .unwrap_or_else(|| "none".to_string());
    let latest_request = recent
        .first()
        .map(|entry| entry.request_id.clone())
        .unwrap_or_else(|| "none".to_string());
    let watch_windows = vec![
        FactRow {
            label: "Config version".to_string(),
            value: config_version,
        },
        FactRow {
            label: "Top error".to_string(),
            value: top_error,
        },
        FactRow {
            label: "Latest request".to_string(),
            value: latest_request,
        },
        FactRow {
            label: "Window".to_string(),
            value: query.range.clone(),
        },
    ];

    Json(CommandCenterResponse {
        kpis,
        signals,
        pressure_map,
        watch_windows,
        inspector,
    })
}

pub async fn traffic_lab(
    State(state): State<AppState>,
    Query(query): Query<WorkspaceQuery>,
) -> Json<TrafficLabResponse> {
    let from = range_start_timestamp(&query.range);
    let stats = state
        .log_store
        .stats(&StatsQuery {
            from: Some(from),
            ..Default::default()
        })
        .await;
    let recent = query_recent_logs(&state, from, query.limit.max(1)).await;
    let selected = recent.first().cloned();
    let sessions = recent.iter().map(traffic_session_item).collect::<Vec<_>>();
    let trace = selected
        .as_ref()
        .map(|record| build_traffic_trace(&state, record))
        .unwrap_or_else(|| {
            vec![TimelineStep {
                label: "No traffic".to_string(),
                tone: "neutral".to_string(),
                title: "No request sessions available".to_string(),
                detail: "The selected time range has no request log entries yet.".to_string(),
            }]
        });
    let compare_facts = vec![
        FactRow {
            label: "Window".to_string(),
            value: query.range.clone(),
        },
        FactRow {
            label: "Entries".to_string(),
            value: stats.total_entries.to_string(),
        },
        FactRow {
            label: "Errors".to_string(),
            value: stats.error_count.to_string(),
        },
        FactRow {
            label: "Avg latency".to_string(),
            value: format!("{} ms", stats.avg_latency_ms),
        },
    ];
    let inspector = traffic_inspector(selected.as_ref(), &query);

    Json(TrafficLabResponse {
        selected_request_id: selected.as_ref().map(|record| record.request_id.clone()),
        sessions,
        compare_facts,
        trace,
        inspector,
    })
}

pub async fn provider_atlas(
    State(state): State<AppState>,
    Query(query): Query<WorkspaceQuery>,
) -> Json<ProviderAtlasResponse> {
    let config = state.config.load();
    let rows = config
        .providers
        .iter()
        .map(|provider| provider_row(&state, provider))
        .collect::<Vec<_>>();

    let healthy = rows.iter().filter(|row| row.status == "Healthy").count();
    let managed = config
        .providers
        .iter()
        .flat_map(|provider| provider.expanded_auth_profiles())
        .filter(|profile| profile.mode.is_managed())
        .count();
    let stream_ready = config
        .providers
        .iter()
        .filter(|provider| {
            cached_probe_result(&state, &provider.name)
                .map(|probe| probe.capability_status("stream") == ProbeStatus::Verified)
                .unwrap_or(false)
        })
        .count();
    let coverage = vec![
        FactRow {
            label: "Healthy providers".to_string(),
            value: healthy.to_string(),
        },
        FactRow {
            label: "Managed auth profiles".to_string(),
            value: managed.to_string(),
        },
        FactRow {
            label: "Verified stream surfaces".to_string(),
            value: stream_ready.to_string(),
        },
        FactRow {
            label: "Source mode".to_string(),
            value: query.source_mode.clone(),
        },
    ];
    let inspector = rows
        .first()
        .map(|row| provider_inspector(row, &config))
        .unwrap_or_else(default_provider_inspector);

    Json(ProviderAtlasResponse {
        providers: rows,
        coverage,
        inspector,
    })
}

pub async fn route_studio(
    State(state): State<AppState>,
    Query(query): Query<WorkspaceQuery>,
) -> Json<RouteStudioResponse> {
    let config = state.config.load();
    let scenarios = build_route_scenarios(&state, &config, &query.range).await;
    let routable = scenarios
        .iter()
        .filter(|scenario| scenario.decision != "Blocked")
        .count();
    let summary_facts = vec![
        FactRow {
            label: "Default profile".to_string(),
            value: config.routing.default_profile.clone(),
        },
        FactRow {
            label: "Profiles".to_string(),
            value: config.routing.profiles.len().to_string(),
        },
        FactRow {
            label: "Rules".to_string(),
            value: config.routing.rules.len().to_string(),
        },
        FactRow {
            label: "Model transforms".to_string(),
            value: total_model_resolution_steps(&config).to_string(),
        },
    ];
    let explain_facts = vec![
        FactRow {
            label: "Sampled scenarios".to_string(),
            value: scenarios.len().to_string(),
        },
        FactRow {
            label: "Routable".to_string(),
            value: routable.to_string(),
        },
        FactRow {
            label: "Blocked".to_string(),
            value: scenarios.len().saturating_sub(routable).to_string(),
        },
        FactRow {
            label: "Window".to_string(),
            value: query.range.clone(),
        },
    ];
    let inspector = route_inspector(&config, scenarios.first());

    Json(RouteStudioResponse {
        summary_facts,
        explain_facts,
        scenarios,
        inspector,
    })
}

pub async fn change_studio(
    State(state): State<AppState>,
    Query(query): Query<WorkspaceQuery>,
) -> Json<ChangeStudioResponse> {
    let config = state.config.load();
    let config_version = config_tx::read_config_versioned(&state)
        .map(|(_, version)| version)
        .unwrap_or_else(|_| "unavailable".to_string());
    let registry = build_registry_rows(&config);
    let publish_facts = vec![
        FactRow {
            label: "Config version".to_string(),
            value: config_version.clone(),
        },
        FactRow {
            label: "Config path".to_string(),
            value: state
                .config_path
                .lock()
                .map(|path| path.clone())
                .unwrap_or_else(|_| "unavailable".to_string()),
        },
        FactRow {
            label: "Providers".to_string(),
            value: config.providers.len().to_string(),
        },
        FactRow {
            label: "Selected window".to_string(),
            value: query.range.clone(),
        },
    ];
    let inspector = change_inspector(&config, config_version);

    Json(ChangeStudioResponse {
        registry,
        publish_facts,
        inspector,
    })
}

fn range_start_timestamp(range: &str) -> i64 {
    let now = Utc::now();
    let from = match range {
        "15m" => now - Duration::minutes(15),
        "6h" => now - Duration::hours(6),
        "24h" => now - Duration::hours(24),
        _ => now - Duration::hours(1),
    };
    from.timestamp_millis()
}

async fn query_recent_logs(state: &AppState, from: i64, limit: usize) -> Vec<RequestRecord> {
    state
        .log_store
        .query(&LogQuery {
            page: Some(1),
            page_size: Some(limit),
            from: Some(from),
            sort_by: Some(SortField::Timestamp),
            sort_order: Some(SortOrder::Desc),
            ..Default::default()
        })
        .await
        .data
}

fn percentage(numerator: usize, denominator: usize) -> String {
    if denominator == 0 {
        return "0%".to_string();
    }
    format!("{:.1}%", (numerator as f64 / denominator as f64) * 100.0)
}

fn latest_freshness(records: &[RequestRecord]) -> Option<i64> {
    records
        .first()
        .map(|record| (Utc::now() - record.timestamp).num_seconds().max(0))
}

fn build_signal_items(
    state: &AppState,
    config: &Config,
    recent: &[RequestRecord],
    stats: &prism_core::request_log::LogStats,
) -> Vec<SignalItem> {
    let mut signals = Vec::new();

    for provider in &config.providers {
        let (status, tone, detail) = provider_runtime_status(state, provider);
        if status != "healthy" {
            signals.push(SignalItem {
                id: format!("provider-{}", provider.name),
                title: format!("Provider {} is {}", provider.name, status),
                detail,
                severity: status.to_string(),
                severity_tone: tone.to_string(),
                target_workspace: "Provider Atlas".to_string(),
            });
        }
    }

    if let Some(top_error) = stats.top_errors.first() {
        signals.push(SignalItem {
            id: format!("error-{}", top_error.error_type),
            title: format!(
                "Recent {} requests need investigation",
                top_error.error_type
            ),
            detail: format!(
                "{} requests in the current window hit this error type.",
                top_error.count
            ),
            severity: "watch".to_string(),
            severity_tone: "warning".to_string(),
            target_workspace: "Traffic Lab".to_string(),
        });
    }

    let fallback_sessions = recent
        .iter()
        .filter(|record| record.total_attempts > 1)
        .count();
    if fallback_sessions > 0 {
        signals.push(SignalItem {
            id: "fallback-surge".to_string(),
            title: "Fallback traffic is above zero".to_string(),
            detail: format!(
                "{} recent request sessions retried across providers.",
                fallback_sessions
            ),
            severity: "watch".to_string(),
            severity_tone: "info".to_string(),
            target_workspace: "Route Studio".to_string(),
        });
    }

    signals.truncate(6);
    signals
}

fn inspector_from_signal(
    signal: &SignalItem,
    query: &WorkspaceQuery,
    recent: &[RequestRecord],
) -> WorkspaceInspector {
    WorkspaceInspector {
        eyebrow: "SIGNAL / ACTIVE".to_string(),
        title: signal.title.clone(),
        summary: signal.detail.clone(),
        sections: vec![
            InspectorSection {
                title: "Posture".to_string(),
                rows: vec![
                    InspectorRow {
                        label: "Severity".to_string(),
                        value: signal.severity.clone(),
                    },
                    InspectorRow {
                        label: "Target".to_string(),
                        value: signal.target_workspace.clone(),
                    },
                    InspectorRow {
                        label: "Source".to_string(),
                        value: query.source_mode.clone(),
                    },
                ],
            },
            InspectorSection {
                title: "Runtime".to_string(),
                rows: vec![
                    InspectorRow {
                        label: "Latest request".to_string(),
                        value: recent
                            .first()
                            .map(|record| record.request_id.clone())
                            .unwrap_or_else(|| "none".to_string()),
                    },
                    InspectorRow {
                        label: "Window".to_string(),
                        value: query.range.clone(),
                    },
                ],
            },
        ],
        actions: vec![
            "Open investigation".to_string(),
            "Jump to workspace".to_string(),
            "Refresh signal queue".to_string(),
        ],
    }
}

fn traffic_session_item(record: &RequestRecord) -> TrafficSessionItem {
    let decision = if record.total_attempts > 1 {
        format!("Fallback after {} attempts", record.total_attempts)
    } else if let Some(provider) = &record.provider {
        format!("Primary {} served request", provider)
    } else {
        "Provider not resolved".to_string()
    };
    let (result, result_tone) = request_result(record);

    TrafficSessionItem {
        request_id: record.request_id.clone(),
        model: record
            .requested_model
            .clone()
            .or_else(|| record.model.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        decision,
        result,
        result_tone: result_tone.to_string(),
        latency_ms: record.latency_ms,
    }
}

fn request_result(record: &RequestRecord) -> (String, &'static str) {
    if record.status >= 500 || record.error.is_some() {
        return ("Failed".to_string(), "danger");
    }
    if record.total_attempts > 1 {
        return ("Recovered".to_string(), "warning");
    }
    ("Success".to_string(), "success")
}

fn build_traffic_trace(state: &AppState, record: &RequestRecord) -> Vec<TimelineStep> {
    let mut steps = Vec::new();
    steps.push(TimelineStep {
        label: "Ingress".to_string(),
        tone: "info".to_string(),
        title: format!(
            "{} {}",
            record.method,
            record
                .requested_model
                .clone()
                .or_else(|| record.model.clone())
                .unwrap_or_else(|| "unknown-model".to_string())
        ),
        detail: format!(
            "{} request entered {}{}",
            record.path,
            record
                .tenant_id
                .as_deref()
                .map(|tenant| format!("tenant {}", tenant))
                .unwrap_or_else(|| "gateway scope".to_string()),
            if record.stream { " as stream" } else { "" }
        ),
    });

    if let Some(explanation) = explain_record(state, record)
        && let Some(selected) = explanation.selected
    {
        let rejection_count = explanation.rejections.len();
        steps.push(TimelineStep {
            label: "Route explain".to_string(),
            tone: if rejection_count > 0 {
                "warning"
            } else {
                "success"
            }
            .to_string(),
            title: format!("{} selected {}", explanation.profile, selected.provider),
            detail: if rejection_count > 0 {
                format!(
                    "{} rejections observed before final route selection.",
                    rejection_count
                )
            } else {
                "Planner selected a provider without rejections.".to_string()
            },
        });
    }

    if record.attempts.is_empty() {
        steps.push(TimelineStep {
            label: "Execution".to_string(),
            tone: request_result(record).1.to_string(),
            title: record
                .provider
                .clone()
                .unwrap_or_else(|| "No upstream attempt captured".to_string()),
            detail: format!(
                "Finished with HTTP {} in {} ms.",
                record.status, record.latency_ms
            ),
        });
    } else {
        steps.extend(record.attempts.iter().take(4).map(attempt_timeline_step));
    }

    steps
}

fn attempt_timeline_step(attempt: &AttemptSummary) -> TimelineStep {
    let tone = if attempt.status.unwrap_or_default() >= 500 || attempt.error.is_some() {
        "danger"
    } else if attempt.status.unwrap_or_default() >= 400 {
        "warning"
    } else {
        "success"
    };

    let detail = attempt.error.clone().unwrap_or_else(|| {
        format!(
            "status {} in {} ms",
            attempt
                .status
                .map(|status| status.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            attempt.latency_ms
        )
    });

    TimelineStep {
        label: format!("Attempt {}", attempt.attempt_index + 1),
        tone: tone.to_string(),
        title: format!("{} / {}", attempt.provider, attempt.model),
        detail,
    }
}

fn explain_record(state: &AppState, record: &RequestRecord) -> Option<RouteExplanation> {
    let requested_model = record
        .requested_model
        .clone()
        .or_else(|| record.model.clone())?;
    let endpoint = endpoint_from_path(&record.path);
    let features = RouteRequestFeatures {
        requested_model,
        endpoint,
        source_format: source_format_from_path(&record.path),
        tenant_id: record.tenant_id.clone(),
        api_key_id: record.api_key_id.clone(),
        region: record.client_region.clone(),
        stream: record.stream,
        headers: Default::default(),
        allowed_credentials: Vec::new(),
        required_capabilities: None,
    };
    let config = state.config.load();
    let inventory = state.catalog.snapshot();
    let health = state.health_manager.snapshot();
    let plan = RoutePlanner::plan(&features, &config.routing, &inventory, &health);
    Some(explain(&plan))
}

fn endpoint_from_path(path: &str) -> RouteEndpoint {
    match path {
        "/v1/messages" => RouteEndpoint::Messages,
        "/v1/responses" | "/v1/responses/ws" => RouteEndpoint::Responses,
        value if value.contains(":generateContent") => RouteEndpoint::GenerateContent,
        value if value.contains(":streamGenerateContent") => RouteEndpoint::StreamGenerateContent,
        _ => RouteEndpoint::ChatCompletions,
    }
}

fn source_format_from_path(path: &str) -> prism_core::provider::Format {
    match endpoint_from_path(path) {
        RouteEndpoint::Messages => prism_core::provider::Format::Claude,
        RouteEndpoint::GenerateContent | RouteEndpoint::StreamGenerateContent => {
            prism_core::provider::Format::Gemini
        }
        _ => prism_core::provider::Format::OpenAI,
    }
}

fn traffic_inspector(record: Option<&RequestRecord>, query: &WorkspaceQuery) -> WorkspaceInspector {
    if let Some(record) = record {
        let (outcome, _) = request_result(record);
        return WorkspaceInspector {
            eyebrow: "SESSION / SELECTED".to_string(),
            title: record.request_id.clone(),
            summary: record.error.clone().unwrap_or_else(|| {
                format!(
                    "{} via {}",
                    outcome,
                    record
                        .provider
                        .clone()
                        .unwrap_or_else(|| "unknown-provider".to_string())
                )
            }),
            sections: vec![
                InspectorSection {
                    title: "Execution".to_string(),
                    rows: vec![
                        InspectorRow {
                            label: "Outcome".to_string(),
                            value: outcome,
                        },
                        InspectorRow {
                            label: "Latency".to_string(),
                            value: format!("{} ms", record.latency_ms),
                        },
                        InspectorRow {
                            label: "Attempts".to_string(),
                            value: record.total_attempts.to_string(),
                        },
                    ],
                },
                InspectorSection {
                    title: "Context".to_string(),
                    rows: vec![
                        InspectorRow {
                            label: "Provider".to_string(),
                            value: record
                                .provider
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                        },
                        InspectorRow {
                            label: "Tenant".to_string(),
                            value: record
                                .tenant_id
                                .clone()
                                .unwrap_or_else(|| "unscoped".to_string()),
                        },
                        InspectorRow {
                            label: "Source".to_string(),
                            value: query.source_mode.clone(),
                        },
                    ],
                },
            ],
            actions: vec![
                "Open raw log".to_string(),
                "Explain route".to_string(),
                "Compare current window".to_string(),
            ],
        };
    }

    WorkspaceInspector {
        eyebrow: "SESSION / EMPTY".to_string(),
        title: "No request session selected".to_string(),
        summary: "The selected time range does not currently contain request sessions.".to_string(),
        sections: vec![],
        actions: vec!["Refresh".to_string()],
    }
}

fn provider_row(state: &AppState, provider: &ProviderKeyEntry) -> ProviderAtlasRow {
    let profiles = provider
        .expanded_auth_profiles()
        .into_iter()
        .map(|profile| {
            state
                .auth_runtime
                .apply_runtime_state(&provider.name, &profile)
                .unwrap_or(profile)
        })
        .collect::<Vec<_>>();
    let auth = profiles
        .first()
        .map(auth_mode_label)
        .unwrap_or_else(|| "Static api_key".to_string());
    let rotation = provider_rotation_summary(&profiles);
    let (status, tone, _) = provider_runtime_status(state, provider);

    ProviderAtlasRow {
        provider: provider.name.clone(),
        format: provider.format.as_str().to_string(),
        auth,
        status: title_case(status),
        status_tone: tone.to_string(),
        rotation,
        region: provider
            .region
            .clone()
            .unwrap_or_else(|| "global".to_string()),
        wire_api: wire_api_label(provider.wire_api).to_string(),
        model_count: provider.models.len(),
    }
}

fn auth_mode_label(profile: &AuthProfileEntry) -> String {
    match profile.mode {
        AuthMode::ApiKey => "API key".to_string(),
        AuthMode::BearerToken => "Bearer token".to_string(),
        AuthMode::CodexOAuth => "Codex OAuth".to_string(),
        AuthMode::AnthropicClaudeSubscription => "Claude subscription".to_string(),
    }
}

fn provider_rotation_summary(profiles: &[AuthProfileEntry]) -> String {
    if profiles.is_empty() {
        return "No auth profile".to_string();
    }

    for profile in profiles {
        if profile.mode.is_managed() {
            if profile
                .refresh_token
                .as_deref()
                .unwrap_or_default()
                .is_empty()
                && profile
                    .access_token
                    .as_deref()
                    .unwrap_or_default()
                    .is_empty()
            {
                return "Disconnected".to_string();
            }
            if let Some(expires_at) = &profile.expires_at
                && let Ok(expiry) = DateTime::parse_from_rfc3339(expires_at)
            {
                let remaining = expiry.with_timezone(&Utc) - Utc::now();
                if remaining.num_seconds() <= 0 {
                    return "Expired".to_string();
                }
                if remaining.num_days() < 1 {
                    return format!("Renews in {}h", remaining.num_hours());
                }
                return format!("Renews in {}d", remaining.num_days());
            }
            return "Managed".to_string();
        }
    }

    "Static".to_string()
}

fn provider_runtime_status(
    state: &AppState,
    provider: &ProviderKeyEntry,
) -> (&'static str, &'static str, String) {
    if provider.disabled {
        return (
            "disabled",
            "neutral",
            "Provider is disabled in config.".to_string(),
        );
    }

    let profiles = provider
        .expanded_auth_profiles()
        .into_iter()
        .map(|profile| {
            state
                .auth_runtime
                .apply_runtime_state(&provider.name, &profile)
                .unwrap_or(profile)
        })
        .collect::<Vec<_>>();

    let disconnected = profiles.iter().any(|profile| {
        profile.mode.is_managed()
            && profile
                .refresh_token
                .as_deref()
                .unwrap_or_default()
                .is_empty()
            && profile
                .access_token
                .as_deref()
                .unwrap_or_default()
                .is_empty()
    });
    if disconnected {
        return (
            "degraded",
            "warning",
            "Managed auth is configured but not currently connected.".to_string(),
        );
    }

    if let Some(probe) = cached_probe_result(state, &provider.name) {
        match probe.status.as_str() {
            "failed" => {
                return (
                    "degraded",
                    "warning",
                    "Latest live capability probe failed.".to_string(),
                );
            }
            "verified" => {
                return (
                    "healthy",
                    "success",
                    "Latest live capability probe passed.".to_string(),
                );
            }
            _ => {
                return (
                    "watch",
                    "info",
                    "No successful live probe has been recorded yet.".to_string(),
                );
            }
        }
    }

    ("watch", "info", "No probe result recorded yet.".to_string())
}

fn provider_inspector(row: &ProviderAtlasRow, config: &Config) -> WorkspaceInspector {
    let linked_routes = config
        .routing
        .profiles
        .values()
        .filter(|profile| {
            profile
                .provider_policy
                .order
                .iter()
                .any(|entry| entry == &row.provider)
                || profile.provider_policy.weights.contains_key(&row.provider)
        })
        .count();
    WorkspaceInspector {
        eyebrow: "PROVIDER / PRIMARY".to_string(),
        title: row.provider.clone(),
        summary: format!("{} / {} / {}", row.auth, row.status, row.rotation),
        sections: vec![
            InspectorSection {
                title: "Identity".to_string(),
                rows: vec![
                    InspectorRow {
                        label: "Format".to_string(),
                        value: row.format.clone(),
                    },
                    InspectorRow {
                        label: "Region".to_string(),
                        value: row.region.clone(),
                    },
                    InspectorRow {
                        label: "Wire".to_string(),
                        value: row.wire_api.clone(),
                    },
                ],
            },
            InspectorSection {
                title: "Impact".to_string(),
                rows: vec![
                    InspectorRow {
                        label: "Models".to_string(),
                        value: row.model_count.to_string(),
                    },
                    InspectorRow {
                        label: "Linked route profiles".to_string(),
                        value: linked_routes.to_string(),
                    },
                ],
            },
        ],
        actions: vec![
            "Open provider config".to_string(),
            "Run live health check".to_string(),
            "Inspect auth profile".to_string(),
        ],
    }
}

fn default_provider_inspector() -> WorkspaceInspector {
    WorkspaceInspector {
        eyebrow: "PROVIDER / EMPTY".to_string(),
        title: "No providers configured".to_string(),
        summary: "Add at least one provider before using runtime control-plane workflows."
            .to_string(),
        sections: vec![],
        actions: vec!["Create provider".to_string()],
    }
}

async fn build_route_scenarios(
    state: &AppState,
    config: &Config,
    range: &str,
) -> Vec<RouteScenarioRow> {
    let from = range_start_timestamp(range);
    let recent = query_recent_logs(state, from, 8).await;
    let mut requests = recent
        .iter()
        .filter_map(|record| {
            let model = record
                .requested_model
                .clone()
                .or_else(|| record.model.clone())?;
            Some(RouteRequestFeatures {
                requested_model: model,
                endpoint: endpoint_from_path(&record.path),
                source_format: source_format_from_path(&record.path),
                tenant_id: record.tenant_id.clone(),
                api_key_id: record.api_key_id.clone(),
                region: record.client_region.clone(),
                stream: record.stream,
                headers: Default::default(),
                allowed_credentials: Vec::new(),
                required_capabilities: None,
            })
        })
        .collect::<Vec<_>>();

    if requests.is_empty() {
        requests = fallback_route_requests(config);
    }

    let inventory = state.catalog.snapshot();
    let health = state.health_manager.snapshot();
    let mut seen = HashSet::new();
    let mut rows = Vec::new();

    for request in requests {
        let key = format!(
            "{}:{}:{}",
            request
                .tenant_id
                .clone()
                .unwrap_or_else(|| "gateway".to_string()),
            request.requested_model,
            request.stream
        );
        if !seen.insert(key) {
            continue;
        }
        let explanation = explain(&RoutePlanner::plan(
            &request,
            &config.routing,
            &inventory,
            &health,
        ));
        let winner = explanation
            .selected
            .as_ref()
            .map(|selected| selected.provider.clone())
            .unwrap_or_else(|| "none".to_string());
        let blocked = explanation.selected.is_none();
        let decision = if blocked {
            "Blocked"
        } else if !explanation.rejections.is_empty() {
            "Fallback-ready"
        } else {
            "Routable"
        };
        let decision_tone = if blocked {
            "danger"
        } else if !explanation.rejections.is_empty() {
            "warning"
        } else {
            "success"
        };
        let delta = explanation
            .matched_rule
            .clone()
            .unwrap_or_else(|| format!("profile {}", explanation.profile));

        rows.push(RouteScenarioRow {
            scenario: format!(
                "{} / {}",
                request
                    .tenant_id
                    .clone()
                    .unwrap_or_else(|| "gateway".to_string()),
                request.requested_model
            ),
            winner,
            delta,
            decision: decision.to_string(),
            decision_tone: decision_tone.to_string(),
            endpoint: route_endpoint_label(&request.endpoint).to_string(),
            source_format: request.source_format.as_str().to_string(),
            stream: request.stream,
            model: request.requested_model.clone(),
            tenant_id: request.tenant_id.clone(),
            api_key_id: request.api_key_id.clone(),
            region: request.region.clone(),
        });

        if rows.len() >= 6 {
            break;
        }
    }

    rows
}

fn route_endpoint_label(endpoint: &RouteEndpoint) -> &'static str {
    match endpoint {
        RouteEndpoint::ChatCompletions => "chat-completions",
        RouteEndpoint::Messages => "messages",
        RouteEndpoint::Responses => "responses",
        RouteEndpoint::GenerateContent => "generate-content",
        RouteEndpoint::StreamGenerateContent => "stream-generate-content",
        RouteEndpoint::Models => "models",
    }
}

fn fallback_route_requests(config: &Config) -> Vec<RouteRequestFeatures> {
    config
        .providers
        .iter()
        .flat_map(|provider| {
            provider
                .models
                .iter()
                .take(2)
                .map(|model| RouteRequestFeatures {
                    requested_model: model.alias.clone().unwrap_or_else(|| model.id.clone()),
                    endpoint: RouteEndpoint::ChatCompletions,
                    source_format: prism_core::provider::Format::OpenAI,
                    tenant_id: None,
                    api_key_id: None,
                    region: provider.region.clone(),
                    stream: false,
                    headers: Default::default(),
                    allowed_credentials: Vec::new(),
                    required_capabilities: None,
                })
        })
        .collect()
}

fn wire_api_label(wire_api: prism_core::provider::WireApi) -> &'static str {
    match wire_api {
        prism_core::provider::WireApi::Chat => "chat",
        prism_core::provider::WireApi::Responses => "responses",
    }
}

fn total_model_resolution_steps(config: &Config) -> usize {
    config.routing.model_resolution.aliases.len()
        + config.routing.model_resolution.rewrites.len()
        + config.routing.model_resolution.fallbacks.len()
        + config.routing.model_resolution.provider_pins.len()
}

fn route_inspector(
    config: &Config,
    first_scenario: Option<&RouteScenarioRow>,
) -> WorkspaceInspector {
    let profile_names = config
        .routing
        .profiles
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    WorkspaceInspector {
        eyebrow: "ROUTE / CURRENT".to_string(),
        title: config.routing.default_profile.clone(),
        summary: format!("Profiles: {}", profile_names),
        sections: vec![
            InspectorSection {
                title: "Routing scope".to_string(),
                rows: vec![
                    InspectorRow {
                        label: "Default profile".to_string(),
                        value: config.routing.default_profile.clone(),
                    },
                    InspectorRow {
                        label: "Rules".to_string(),
                        value: config.routing.rules.len().to_string(),
                    },
                ],
            },
            InspectorSection {
                title: "Current sample".to_string(),
                rows: vec![
                    InspectorRow {
                        label: "Scenario".to_string(),
                        value: first_scenario
                            .map(|scenario| scenario.scenario.clone())
                            .unwrap_or_else(|| "none".to_string()),
                    },
                    InspectorRow {
                        label: "Decision".to_string(),
                        value: first_scenario
                            .map(|scenario| scenario.decision.clone())
                            .unwrap_or_else(|| "n/a".to_string()),
                    },
                ],
            },
        ],
        actions: vec![
            "Explain route".to_string(),
            "Open routing config".to_string(),
            "Patch profiles".to_string(),
        ],
    }
}

fn build_registry_rows(config: &Config) -> Vec<RegistryRow> {
    let explicit_auth_profiles = config
        .providers
        .iter()
        .map(|provider| provider.auth_profiles.len())
        .sum::<usize>();
    let tenants = config
        .auth_keys
        .iter()
        .filter_map(|entry| entry.tenant_id.clone())
        .collect::<BTreeSet<_>>();

    vec![
        RegistryRow {
            family: "providers".to_string(),
            record: format!("{} providers", config.providers.len()),
            state: if config.providers.is_empty() {
                "empty".to_string()
            } else {
                "configured".to_string()
            },
            state_tone: if config.providers.is_empty() {
                "warning".to_string()
            } else {
                "success".to_string()
            },
            dependents: format!("{} route profiles", config.routing.profiles.len()),
        },
        RegistryRow {
            family: "auth-profiles".to_string(),
            record: format!("{explicit_auth_profiles} explicit profiles"),
            state: if explicit_auth_profiles == 0 {
                "implicit-only".to_string()
            } else {
                "configured".to_string()
            },
            state_tone: if explicit_auth_profiles == 0 {
                "info".to_string()
            } else {
                "success".to_string()
            },
            dependents: format!("{} providers", config.providers.len()),
        },
        RegistryRow {
            family: "auth-keys".to_string(),
            record: format!("{} auth keys", config.auth_keys.len()),
            state: if config.auth_keys.is_empty() {
                "empty".to_string()
            } else {
                "configured".to_string()
            },
            state_tone: if config.auth_keys.is_empty() {
                "warning".to_string()
            } else {
                "success".to_string()
            },
            dependents: format!("{} tenants", tenants.len()),
        },
        RegistryRow {
            family: "route-profiles".to_string(),
            record: format!("{} profiles", config.routing.profiles.len()),
            state: "live".to_string(),
            state_tone: "success".to_string(),
            dependents: format!("{} rules", config.routing.rules.len()),
        },
        RegistryRow {
            family: "model-resolution".to_string(),
            record: format!("{} transforms", total_model_resolution_steps(config)),
            state: if total_model_resolution_steps(config) == 0 {
                "baseline".to_string()
            } else {
                "customized".to_string()
            },
            state_tone: if total_model_resolution_steps(config) == 0 {
                "info".to_string()
            } else {
                "success".to_string()
            },
            dependents: config.routing.default_profile.clone(),
        },
    ]
}

fn change_inspector(config: &Config, config_version: String) -> WorkspaceInspector {
    WorkspaceInspector {
        eyebrow: "CHANGE / CONFIG".to_string(),
        title: config_version,
        summary: "Greenfield Change Studio uses the config transaction path as the current source of truth until structured change objects land.".to_string(),
        sections: vec![
            InspectorSection {
                title: "Current shape".to_string(),
                rows: vec![
                    InspectorRow {
                        label: "Providers".to_string(),
                        value: config.providers.len().to_string(),
                    },
                    InspectorRow {
                        label: "Auth keys".to_string(),
                        value: config.auth_keys.len().to_string(),
                    },
                    InspectorRow {
                        label: "Route rules".to_string(),
                        value: config.routing.rules.len().to_string(),
                    },
                ],
            },
            InspectorSection {
                title: "Transaction path".to_string(),
                rows: vec![
                    InspectorRow {
                        label: "Validate".to_string(),
                        value: "available".to_string(),
                    },
                    InspectorRow {
                        label: "Apply".to_string(),
                        value: "available".to_string(),
                    },
                    InspectorRow {
                        label: "Reload".to_string(),
                        value: "available".to_string(),
                    },
                ],
            },
        ],
        actions: vec![
            "Open raw YAML".to_string(),
            "Validate current config".to_string(),
            "Reload runtime".to_string(),
        ],
    }
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}
