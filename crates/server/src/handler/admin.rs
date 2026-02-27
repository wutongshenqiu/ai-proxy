use crate::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

/// GET /admin/config — returns sanitized (no API keys) configuration.
pub async fn admin_config(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.load();
    let sanitized = serde_json::json!({
        "host": config.host,
        "port": config.port,
        "tls": { "enable": config.tls.enable },
        "api_keys_count": config.api_keys.len(),
        "routing": config.routing,
        "retry": config.retry,
        "body_limit_mb": config.body_limit_mb,
        "streaming": config.streaming,
        "connect_timeout": config.connect_timeout,
        "request_timeout": config.request_timeout,
        "claude_keys_count": config.claude_api_key.len(),
        "openai_keys_count": config.openai_api_key.len(),
        "gemini_keys_count": config.gemini_api_key.len(),
        "compat_keys_count": config.openai_compatibility.len(),
    });
    Json(sanitized)
}

/// GET /admin/metrics — same as /metrics, full metrics snapshot.
pub async fn admin_metrics(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.metrics.snapshot())
}

/// GET /admin/models — list all available models.
pub async fn admin_models(State(state): State<AppState>) -> impl IntoResponse {
    let models = state.router.all_models();
    Json(serde_json::json!({ "models": models }))
}
