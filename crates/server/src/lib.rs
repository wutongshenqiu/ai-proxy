pub mod auth;
pub mod dispatch;
pub mod handler;
pub mod middleware;
pub mod streaming;

use ai_proxy_core::config::Config;
use ai_proxy_core::metrics::Metrics;
use ai_proxy_provider::routing::CredentialRouter;
use ai_proxy_provider::ExecutorRegistry;
use ai_proxy_translator::TranslatorRegistry;
use arc_swap::ArcSwap;
use axum::{middleware as axum_mw, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ArcSwap<Config>>,
    pub router: Arc<CredentialRouter>,
    pub executors: Arc<ExecutorRegistry>,
    pub translators: Arc<TranslatorRegistry>,
    pub metrics: Arc<Metrics>,
}

pub fn build_router(state: AppState) -> Router {
    let body_limit_bytes = state.config.load().body_limit_mb * 1024 * 1024;

    // Public routes — no auth required
    let public_routes = Router::new()
        .route("/health", axum::routing::get(handler::health::health))
        .route(
            "/metrics",
            axum::routing::get(handler::health::metrics),
        );

    // Admin routes — no auth required (read-only)
    let admin_routes = Router::new()
        .route("/admin/config", axum::routing::get(handler::admin::admin_config))
        .route("/admin/metrics", axum::routing::get(handler::admin::admin_metrics))
        .route("/admin/models", axum::routing::get(handler::admin::admin_models));

    // API routes — auth required, with body size limit
    let api_routes = Router::new()
        .route(
            "/v1/models",
            axum::routing::get(handler::models::list_models),
        )
        .route(
            "/v1/chat/completions",
            axum::routing::post(handler::chat_completions::chat_completions),
        )
        .route(
            "/v1/messages",
            axum::routing::post(handler::messages::messages),
        )
        .route(
            "/v1/responses",
            axum::routing::post(handler::responses::responses),
        )
        .layer(RequestBodyLimitLayer::new(body_limit_bytes))
        .layer(axum_mw::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    // Compose: public + admin + api, then global middleware layers (outer → inner)
    Router::new()
        .merge(public_routes)
        .merge(admin_routes)
        .merge(api_routes)
        .layer(axum_mw::from_fn(
            middleware::request_logging::request_logging_middleware,
        ))
        .layer(axum_mw::from_fn(
            middleware::request_context::request_context_middleware,
        ))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
