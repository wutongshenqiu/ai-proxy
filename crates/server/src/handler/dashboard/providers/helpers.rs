use axum::Json;
use axum::http::StatusCode;
use serde_json::json;

pub(super) fn validation_error(
    message: impl Into<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({"error": "validation_failed", "message": message.into()})),
    )
}

pub(super) fn config_tx_error_response(
    error: crate::handler::dashboard::config_tx::ConfigTxError,
) -> (StatusCode, Json<serde_json::Value>) {
    match error {
        crate::handler::dashboard::config_tx::ConfigTxError::Conflict { current_version } => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "conflict",
                "message": "config version conflict",
                "current_version": current_version
            })),
        ),
        crate::handler::dashboard::config_tx::ConfigTxError::Validation(message) => {
            validation_error(message)
        }
        crate::handler::dashboard::config_tx::ConfigTxError::Internal(message) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "write_failed", "message": message})),
        ),
    }
}

pub(super) fn is_valid_format(format_str: &str) -> bool {
    matches!(format_str, "openai" | "claude" | "gemini")
}

pub(super) fn parse_upstream_kind(
    format: prism_core::provider::Format,
    upstream: Option<&str>,
) -> Result<prism_core::provider::UpstreamKind, (StatusCode, Json<serde_json::Value>)> {
    let Some(raw) = upstream.filter(|value| !value.trim().is_empty()) else {
        return Ok(format.into());
    };
    raw.parse().map_err(validation_error)
}
