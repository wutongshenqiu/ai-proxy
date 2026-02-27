use crate::dispatch::{dispatch, DispatchRequest};
use crate::AppState;
use ai_proxy_core::error::ProxyError;
use ai_proxy_core::provider::Format;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use bytes::Bytes;

pub async fn chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ProxyError> {
    let parsed = super::parse_request(&headers, &body)?;

    dispatch(
        &state,
        DispatchRequest {
            source_format: Format::OpenAI,
            model: parsed.model,
            stream: parsed.stream,
            body,
            allowed_formats: None,
            user_agent: parsed.user_agent,
        },
    )
    .await
}
