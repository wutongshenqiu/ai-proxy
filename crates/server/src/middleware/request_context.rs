use ai_proxy_core::context::RequestContext;
use axum::{extract::Request, middleware::Next, response::Response};

/// Middleware that injects a `RequestContext` as an axum Extension.
pub async fn request_context_middleware(mut request: Request, next: Next) -> Response {
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        });

    let ctx = RequestContext::new(client_ip);
    request.extensions_mut().insert(ctx);
    next.run(request).await
}
