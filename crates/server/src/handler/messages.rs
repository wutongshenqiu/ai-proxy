use crate::AppState;
use crate::dispatch::{DispatchRequest, dispatch};
use ai_proxy_core::context::RequestContext;
use ai_proxy_core::error::ProxyError;
use ai_proxy_core::provider::Format;
use axum::Extension;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use bytes::Bytes;

/// Claude Messages API passthrough (/v1/messages).
pub async fn messages(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ProxyError> {
    let parsed = super::parse_request(&headers, &body)?;

    dispatch(
        &state,
        DispatchRequest {
            source_format: Format::Claude,
            model: parsed.model,
            models: parsed.models,
            stream: parsed.stream,
            body,
            allowed_formats: Some(vec![Format::Claude]),
            user_agent: parsed.user_agent,
            debug: parsed.debug,
            api_key: ctx.auth_key.as_ref().map(|e| e.key.clone()),
            client_region: ctx.client_region,
        },
    )
    .await
}
