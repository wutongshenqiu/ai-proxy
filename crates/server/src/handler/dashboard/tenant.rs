use crate::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use serde_json::json;

/// GET /api/dashboard/tenants — list all tenants with summary metrics.
pub async fn list_tenants(State(state): State<AppState>) -> impl IntoResponse {
    let snap = state.metrics.tenant_snapshot();
    let tenants: Vec<serde_json::Value> = if let Some(obj) = snap.as_object() {
        obj.iter()
            .map(|(id, metrics)| {
                json!({
                    "id": id,
                    "requests": metrics["requests"],
                    "tokens": metrics["tokens"],
                    "cost_usd": metrics["cost_usd"],
                })
            })
            .collect()
    } else {
        vec![]
    };
    Json(json!({ "tenants": tenants }))
}

/// GET /api/dashboard/tenants/:id/metrics — detailed metrics for a tenant.
pub async fn tenant_metrics(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> impl IntoResponse {
    let snap = state.metrics.tenant_snapshot();
    if let Some(metrics) = snap.get(&tenant_id) {
        Json(json!({
            "tenant_id": tenant_id,
            "metrics": metrics,
        }))
    } else {
        Json(json!({
            "tenant_id": tenant_id,
            "metrics": null,
        }))
    }
}
