use ai_proxy_core::context::RequestContext;
use axum::{extract::Request, middleware::Next, response::Response};

/// Middleware that logs request/response with request context info.
pub async fn request_logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().path().to_string();

    let ctx = request.extensions().get::<RequestContext>().cloned();
    let request_id = ctx
        .as_ref()
        .map(|c| c.request_id.clone())
        .unwrap_or_default();
    let client_ip = ctx
        .as_ref()
        .and_then(|c| c.client_ip.clone())
        .unwrap_or_else(|| "-".to_string());

    tracing::info!(
        request_id = %request_id,
        client_ip = %client_ip,
        method = %method,
        path = %uri,
        "Request received"
    );

    let response = next.run(request).await;

    let elapsed = ctx.map(|c| c.elapsed_ms()).unwrap_or(0);
    let status = response.status().as_u16();

    tracing::info!(
        request_id = %request_id,
        status = status,
        elapsed_ms = elapsed,
        "Request completed"
    );

    response
}
