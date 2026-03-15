use crate::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

/// POST /api/dashboard/config/validate — dry-run config validation.
/// Accepts either `{"yaml": "..."}` (YAML string) or a raw JSON config object.
///
/// Performs two validation phases:
/// 1. Structural parsing (YAML/JSON → Config)
/// 2. Full resolution including secrets (env://, file://)
pub async fn validate_config(
    State(_state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let yaml_str;
    let parse_result = if let Some(s) = body.get("yaml").and_then(|v| v.as_str()) {
        yaml_str = s.to_string();
        // Phase 1: raw parse — catches structural/schema errors
        prism_core::config::Config::from_yaml_raw(&yaml_str)
    } else {
        match serde_json::from_value::<prism_core::config::Config>(body) {
            Ok(_cfg) => {
                return (StatusCode::OK, Json(json!({"valid": true, "errors": []})));
            }
            Err(e) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({"valid": false, "errors": [e.to_string()]})),
                );
            }
        }
    };

    let mut errors = Vec::new();

    match parse_result {
        Ok(raw_cfg) => {
            // Phase 1 passed. Phase 2: full resolution (secrets, validation)
            // Check for auth fields that reference unresolvable secrets
            for (i, p) in raw_cfg.providers.iter().enumerate() {
                if (p.api_key.starts_with("env://") || p.api_key.starts_with("file://"))
                    && let Err(e) = prism_core::secret::resolve(&p.api_key)
                {
                    errors.push(format!(
                        "providers[{}] '{}': api_key secret resolution failed: {}",
                        i, p.name, e
                    ));
                }
            }
            for (i, ak) in raw_cfg.auth_keys.iter().enumerate() {
                if (ak.key.starts_with("env://") || ak.key.starts_with("file://"))
                    && let Err(e) = prism_core::secret::resolve(&ak.key)
                {
                    errors.push(format!(
                        "auth-keys[{}]: key secret resolution failed: {}",
                        i, e
                    ));
                }
            }

            if !errors.is_empty() {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({"valid": false, "errors": errors})),
                );
            }

            // Full validation with resolution
            match prism_core::config::Config::load_from_str(&raw_cfg.to_yaml().unwrap_or_default())
            {
                Ok(_) => (StatusCode::OK, Json(json!({"valid": true, "errors": []}))),
                Err(e) => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({"valid": false, "errors": [e.to_string()]})),
                ),
            }
        }
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"valid": false, "errors": [e.to_string()]})),
        ),
    }
}

/// POST /api/dashboard/config/reload — trigger hot-reload.
pub async fn reload_config(State(state): State<AppState>) -> impl IntoResponse {
    let config_path = match state.config_path.lock() {
        Ok(path) => path.clone(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "lock_failed", "message": e.to_string()})),
            );
        }
    };

    match prism_core::config::Config::load(&config_path) {
        Ok(new_cfg) => {
            state.router.update_from_config(&new_cfg);
            state
                .catalog
                .update_from_credentials(&state.router.credential_map());
            state.rate_limiter.update_config(&new_cfg.rate_limit);
            state.cost_calculator.update_prices(&new_cfg.model_prices);
            state.http_client_pool.clear();
            state.config.store(std::sync::Arc::new(new_cfg));
            tracing::info!(path = %config_path, "Configuration reloaded via dashboard API");
            (
                StatusCode::OK,
                Json(json!({"message": "Configuration reloaded successfully"})),
            )
        }
        Err(e) => {
            tracing::error!(path = %config_path, error = %e, "Configuration reload failed");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": "reload_failed", "message": e.to_string()})),
            )
        }
    }
}

/// PUT /api/dashboard/config/apply — validate, persist, and reload config.
/// Accepts `{"yaml": "..."}` with the full YAML config to apply.
pub async fn apply_config(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let yaml_str = match body.get("yaml").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": "validation_failed", "message": "Missing 'yaml' field"})),
            );
        }
    };

    // Step 1: Validate
    let runtime_config = match prism_core::config::Config::load_from_str(&yaml_str) {
        Ok(cfg) => cfg,
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": "validation_failed", "message": e.to_string()})),
            );
        }
    };

    // Step 2: Persist atomically
    let config_path = match state.config_path.lock() {
        Ok(path) => path.clone(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "lock_failed", "message": e.to_string()})),
            );
        }
    };

    let dir = std::path::Path::new(&config_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let tmp_path = dir.join(format!(".config.yaml.tmp.{}", std::process::id()));
    if let Err(e) = std::fs::write(&tmp_path, &yaml_str) {
        let _ = std::fs::remove_file(&tmp_path);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "write_failed", "message": e.to_string()})),
        );
    }
    if let Err(e) = std::fs::rename(&tmp_path, &config_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "write_failed", "message": e.to_string()})),
        );
    }

    // Step 3: Reload runtime
    state.router.update_from_config(&runtime_config);
    state
        .catalog
        .update_from_credentials(&state.router.credential_map());
    state.rate_limiter.update_config(&runtime_config.rate_limit);
    state
        .cost_calculator
        .update_prices(&runtime_config.model_prices);
    state.http_client_pool.clear();
    state.config.store(std::sync::Arc::new(runtime_config));

    tracing::info!(path = %config_path, "Configuration applied via dashboard API");
    (
        StatusCode::OK,
        Json(json!({"message": "Configuration applied successfully"})),
    )
}

/// GET /api/dashboard/config/raw — get raw YAML config file contents.
pub async fn get_raw_config(State(state): State<AppState>) -> impl IntoResponse {
    let config_path = match state.config_path.lock() {
        Ok(path) => path.clone(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "lock_failed", "message": e.to_string()})),
            );
        }
    };

    match std::fs::read_to_string(&config_path) {
        Ok(content) => (
            StatusCode::OK,
            Json(json!({"content": content, "path": config_path})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "read_failed", "message": e.to_string()})),
        ),
    }
}

/// GET /api/dashboard/config/current — get full sanitized config.
pub async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.load();
    let sanitized = json!({
        "host": config.host,
        "port": config.port,
        "tls": { "enable": config.tls.enable },
        "auth_keys_count": config.auth_keys.len(),
        "routing": config.routing,
        "retry": config.retry,
        "body_limit_mb": config.body_limit_mb,
        "streaming": config.streaming,
        "connect_timeout": config.connect_timeout,
        "request_timeout": config.request_timeout,
        "dashboard": {
            "enabled": config.dashboard.enabled,
            "username": config.dashboard.username,
            "jwt_ttl_secs": config.dashboard.jwt_ttl_secs,
            "log_store_capacity": config.log_store.capacity,
        },
        "providers": {
            "total": config.providers.len(),
        },
    });
    (StatusCode::OK, Json(sanitized))
}
