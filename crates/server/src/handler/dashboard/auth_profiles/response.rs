use prism_core::auth_profile::{AuthHeaderKind, AuthMode};
use prism_core::presentation::UpstreamPresentationConfig;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub(super) struct AuthProfileListItem {
    pub provider: String,
    pub format: String,
    pub id: String,
    pub qualified_name: String,
    pub mode: AuthMode,
    pub header: AuthHeaderKind,
    pub connected: bool,
    pub secret_masked: Option<String>,
    pub access_token_masked: Option<String>,
    pub refresh_token_present: bool,
    pub id_token_present: bool,
    pub expires_at: Option<String>,
    pub account_id: Option<String>,
    pub email: Option<String>,
    pub last_refresh: Option<String>,
    pub headers: HashMap<String, String>,
    pub disabled: bool,
    pub weight: u32,
    pub region: Option<String>,
    pub prefix: Option<String>,
    pub upstream_presentation: UpstreamPresentationConfig,
}

#[derive(Debug, Serialize)]
pub(super) struct AuthProfilesListResponse {
    pub profiles: Vec<AuthProfileListItem>,
}

#[derive(Debug, Serialize)]
pub(super) struct AuthProfilesRuntimeResponse {
    pub storage_dir: Option<String>,
    pub codex_auth_file: Option<String>,
    pub proxy_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct AuthProfileMutationEnvelope {
    pub profile: AuthProfileListItem,
}

#[derive(Debug, Serialize)]
pub(super) struct AuthProfileDeletedResponse {
    pub deleted: bool,
}
