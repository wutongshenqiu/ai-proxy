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

    // Extract client region from CDN / custom headers
    let client_region = request
        .headers()
        .get("x-client-region")
        .or_else(|| request.headers().get("cf-ipcountry"))
        .or_else(|| request.headers().get("x-vercel-ip-country"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut ctx = RequestContext::new(client_ip);
    ctx.client_region = client_region;
    request.extensions_mut().insert(ctx);
    next.run(request).await
}
