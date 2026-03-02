use crate::auth_key::AuthKeyEntry;
use std::time::Instant;

/// Per-request context carrying metadata for logging, metrics, and audit.
/// Injected as an axum `Extension` by the `RequestContextLayer`.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique request identifier (UUID v4).
    pub request_id: String,
    /// When the request was received.
    pub start_time: Instant,
    /// Client IP address, if available.
    pub client_ip: Option<String>,
    /// Masked API key (first 4 + last 4 chars).
    pub api_key_id: Option<String>,
    /// Tenant ID from auth key entry.
    pub tenant_id: Option<String>,
    /// Full auth key entry (for model ACL, rate limits, etc.).
    pub auth_key: Option<AuthKeyEntry>,
    /// Client region from X-Client-Region / CDN headers.
    pub client_region: Option<String>,
}

impl RequestContext {
    pub fn new(client_ip: Option<String>) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            start_time: Instant::now(),
            client_ip,
            api_key_id: None,
            tenant_id: None,
            auth_key: None,
            client_region: None,
        }
    }

    /// Returns elapsed time since request start.
    pub fn elapsed_ms(&self) -> u128 {
        self.start_time.elapsed().as_millis()
    }
}
