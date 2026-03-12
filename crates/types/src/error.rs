use serde_json::json;

/// Unified error type for all proxy operations.
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("no credentials available for provider {provider}, model {model}")]
    NoCredentials { provider: String, model: String },

    #[error("model {model} is in cooldown for {seconds}s")]
    ModelCooldown { model: String, seconds: u64 },

    #[error("upstream error (status {status}): {body}")]
    Upstream {
        status: u16,
        body: String,
        /// Parsed from upstream `Retry-After` header (seconds), if present.
        retry_after_secs: Option<u64>,
    },

    #[error("network error: {0}")]
    Network(String),

    #[error("translation error: {0}")]
    Translation(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("rate limit exceeded: {message}")]
    RateLimited {
        message: String,
        /// Seconds until the rate limit resets.
        retry_after_secs: u64,
    },

    #[error("model access denied: {0}")]
    ModelNotAllowed(String),

    #[error("API key expired")]
    KeyExpired,

    #[error("internal error: {0}")]
    Internal(String),
}

impl ProxyError {
    /// HTTP status code as u16 (framework-independent).
    pub fn status_code_u16(&self) -> u16 {
        match self {
            Self::Config(_) | Self::Internal(_) => 500,
            Self::Auth(_) | Self::KeyExpired => 401,
            Self::ModelNotAllowed(_) => 403,
            Self::NoCredentials { .. } => 503,
            Self::ModelCooldown { .. } | Self::RateLimited { .. } => 429,
            Self::Upstream { status, .. } => *status,
            Self::Network(_) => 502,
            Self::Translation(_) => 500,
            Self::BadRequest(_) => 400,
            Self::ModelNotFound(_) => 404,
        }
    }

    pub fn error_type(&self) -> &str {
        match self {
            Self::Auth(_) | Self::KeyExpired => "authentication_error",
            Self::ModelNotAllowed(_) => "permission_error",
            Self::NoCredentials { .. } => "insufficient_quota",
            Self::ModelCooldown { .. } | Self::RateLimited { .. } => "rate_limit_error",
            Self::BadRequest(_) => "invalid_request_error",
            Self::ModelNotFound(_) => "invalid_request_error",
            Self::Upstream { .. } => "upstream_error",
            _ => "server_error",
        }
    }

    pub fn error_code(&self) -> &str {
        match self {
            Self::Auth(_) | Self::KeyExpired => "invalid_api_key",
            Self::ModelNotAllowed(_) => "model_not_allowed",
            Self::NoCredentials { .. } => "insufficient_quota",
            Self::ModelCooldown { .. } | Self::RateLimited { .. } => "rate_limit_exceeded",
            Self::ModelNotFound(_) => "model_not_found",
            Self::BadRequest(_) => "invalid_request",
            _ => "internal_error",
        }
    }

    /// Build the standard error JSON response body.
    pub fn to_json_body(&self) -> String {
        // For upstream errors, try to pass through the original JSON body
        if let Self::Upstream { body, .. } = self
            && serde_json::from_str::<serde_json::Value>(body).is_ok()
        {
            return body.clone();
        }

        json!({
            "error": {
                "message": self.to_string(),
                "type": self.error_type(),
                "code": self.error_code(),
            }
        })
        .to_string()
    }

    /// Retry-After seconds, if applicable.
    pub fn retry_after_secs(&self) -> Option<u64> {
        match self {
            Self::RateLimited {
                retry_after_secs, ..
            } => Some(*retry_after_secs),
            Self::ModelCooldown { seconds, .. } => Some(*seconds),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for ProxyError {
    fn from(e: serde_json::Error) -> Self {
        Self::Translation(format!("JSON error: {e}"))
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for ProxyError {
    fn into_response(self) -> axum::response::Response {
        let status = axum::http::StatusCode::from_u16(self.status_code_u16())
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        let body = self.to_json_body();
        let retry_secs = self.retry_after_secs();

        let mut response = (status, [("content-type", "application/json")], body).into_response();

        if let Some(secs) = retry_secs
            && let Ok(val) = secs.to_string().parse()
        {
            response.headers_mut().insert("retry-after", val);
        }

        response
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for ProxyError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            Self::Network(format!("request timed out: {e}"))
        } else if e.is_connect() {
            Self::Network(format!("connection failed: {e}"))
        } else {
            Self::Network(e.to_string())
        }
    }
}
