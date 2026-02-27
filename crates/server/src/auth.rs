use crate::AppState;
use ai_proxy_core::error::ProxyError;
use axum::{extract::State, http::Request, middleware::Next, response::Response};

pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, ProxyError> {
    let config = state.config.load();

    // If no API keys configured, skip auth
    if config.api_keys.is_empty() {
        return Ok(next.run(request).await);
    }

    // Extract token from Authorization: Bearer or x-api-key header
    let token = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .or_else(|| {
            request
                .headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
        });

    match token {
        Some(t) if config.api_keys_set.contains(t) => Ok(next.run(request).await),
        _ => Err(ProxyError::Auth("Invalid API key".to_string())),
    }
}
