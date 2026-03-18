use crate::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use super::FetchModelsRequest;
use super::common::{build_reqwest_client, client_error_response, normalize_base_url};
use crate::handler::dashboard::providers::helpers::{is_valid_format, parse_upstream_kind};

fn default_base_url(upstream: prism_core::provider::UpstreamKind) -> &'static str {
    upstream.default_base_url()
}

fn unsupported_model_discovery_response(
    message: impl Into<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "models": [],
            "supported": false,
            "message": message.into(),
        })),
    )
}

fn default_fetch_models_response(models: Vec<String>) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "models": models,
            "supported": true,
        })),
    )
}

fn is_dashscope_base_url(base_url: &str) -> bool {
    let base = normalize_base_url(base_url);
    base.contains("dashscope.aliyuncs.com")
}

fn model_discovery_unsupported_reason(
    provider_type: &str,
    api_key: &str,
    base_url: &str,
    upstream: prism_core::provider::UpstreamKind,
) -> Option<String> {
    if upstream == prism_core::provider::UpstreamKind::Codex {
        return Some(
            "Codex upstream does not support model discovery; configure models manually"
                .to_string(),
        );
    }

    if provider_type == "openai"
        && (api_key.starts_with("sk-sp-") || is_dashscope_base_url(base_url))
    {
        return Some(
            "DashScope coding endpoints do not expose model discovery; configure models manually"
                .to_string(),
        );
    }

    None
}

pub(super) fn build_models_request(
    client: &reqwest::Client,
    provider_type: &str,
    api_key: &str,
    base_url: &str,
    extra_headers: Option<&std::collections::HashMap<String, String>>,
) -> Result<reqwest::RequestBuilder, String> {
    let base = normalize_base_url(base_url);
    let mut req = match provider_type {
        "openai" => client
            .get(format!("{base}/v1/models"))
            .header("Authorization", format!("Bearer {api_key}")),
        "claude" => client
            .get(format!("{base}/v1/models"))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01"),
        "gemini" => client
            .get(format!("{base}/v1beta/models"))
            .header("x-goog-api-key", api_key),
        _ => return Err(format!("Unsupported provider_type: {provider_type}")),
    };
    if let Some(headers) = extra_headers {
        for (k, v) in headers {
            req = req.header(k.as_str(), v.as_str());
        }
    }
    Ok(req)
}

fn extract_model_ids(provider_type: &str, body: &serde_json::Value) -> Vec<String> {
    match provider_type {
        "openai" | "claude" => body
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("id").and_then(|v| v.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        "gemini" => body
            .get("models")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        item.get("name")
                            .and_then(|v| v.as_str())
                            .map(|s| s.strip_prefix("models/").unwrap_or(s).to_string())
                    })
                    .collect()
            })
            .unwrap_or_default(),
        _ => vec![],
    }
}

pub async fn fetch_models(
    State(state): State<AppState>,
    Json(body): Json<FetchModelsRequest>,
) -> impl IntoResponse {
    let format = body.format.as_str();

    if !is_valid_format(format) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(
                json!({"error": "validation_failed", "message": "Invalid format. Must be one of: openai, claude, gemini"}),
            ),
        );
    }
    let parsed_format: prism_core::provider::Format = match format.parse() {
        Ok(value) => value,
        Err(_) => prism_core::provider::Format::OpenAI,
    };
    let upstream = match parse_upstream_kind(parsed_format, body.upstream.as_deref()) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let base_url = match body.base_url.as_deref().filter(|s| !s.is_empty()) {
        Some(url) => url.to_string(),
        None => default_base_url(upstream).to_string(),
    };

    if let Some(message) =
        model_discovery_unsupported_reason(format, &body.api_key, &base_url, upstream)
    {
        return unsupported_model_discovery_response(message);
    }

    let global_proxy = state.config.load().proxy_url.clone();
    let client = match build_reqwest_client(&state.http_client_pool, global_proxy.as_deref(), 15) {
        Ok(client) => client,
        Err(error) => return client_error_response(error),
    };

    let request = match build_models_request(&client, format, &body.api_key, &base_url, None) {
        Ok(request) => request,
        Err(error) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": "validation_failed", "message": error})),
            );
        }
    };

    let response: reqwest::Response = match request.send().await {
        Ok(response) => response,
        Err(error) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(
                    json!({"error": "upstream_error", "message": format!("Failed to reach upstream: {error}")}),
                ),
            );
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        if provider_type_supports_optional_model_discovery(format)
            && matches!(status.as_u16(), 404 | 405 | 501)
        {
            return unsupported_model_discovery_response(format!(
                "Upstream does not expose model discovery for this provider; configure models manually ({status})"
            ));
        }
        return (
            StatusCode::BAD_GATEWAY,
            Json(
                json!({"error": "upstream_error", "message": format!("Upstream returned {status}: {body_text}")}),
            ),
        );
    }

    let body_json: serde_json::Value = match response.json().await {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(
                    json!({"error": "upstream_error", "message": format!("Failed to parse upstream response: {error}")}),
                ),
            );
        }
    };

    let models = extract_model_ids(format, &body_json);
    default_fetch_models_response(models)
}

fn provider_type_supports_optional_model_discovery(provider_type: &str) -> bool {
    matches!(provider_type, "openai" | "claude" | "gemini")
}
