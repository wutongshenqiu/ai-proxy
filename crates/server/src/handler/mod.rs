pub mod admin;
pub mod chat_completions;
pub mod dashboard;
pub mod health;
pub mod messages;
pub mod models;
pub mod responses;

use ai_proxy_core::error::ProxyError;
use axum::http::HeaderMap;
use bytes::Bytes;

pub(crate) struct ParsedRequest {
    pub model: String,
    /// Fallback model chain: try models in order until one succeeds.
    pub models: Option<Vec<String>>,
    pub stream: bool,
    pub user_agent: Option<String>,
    /// Debug mode: return routing details in response headers.
    pub debug: bool,
}

pub(crate) fn parse_request(
    headers: &HeaderMap,
    body: &Bytes,
) -> Result<ParsedRequest, ProxyError> {
    let req_value: serde_json::Value =
        serde_json::from_slice(body).map_err(|e| ProxyError::BadRequest(e.to_string()))?;

    let model = req_value
        .get("model")
        .and_then(|m| m.as_str())
        .ok_or_else(|| ProxyError::BadRequest("missing model field".into()))?
        .to_string();

    // Parse `models` array for fallback chain
    let models = req_value.get("models").and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(|m| m.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
    });

    let stream = req_value
        .get("stream")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Check x-debug header
    let debug = headers
        .get("x-debug")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == "true" || v == "1");

    Ok(ParsedRequest {
        model,
        models,
        stream,
        user_agent,
        debug,
    })
}
