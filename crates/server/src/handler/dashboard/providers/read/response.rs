use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub(super) struct ProviderListResponse {
    pub providers: Vec<ProviderSummary>,
}

#[derive(Debug, Serialize)]
pub(super) struct ProviderSummary {
    pub name: String,
    pub format: String,
    pub upstream: String,
    pub api_key_masked: String,
    pub base_url: Option<String>,
    pub models: Vec<prism_core::config::ModelMapping>,
    pub disabled: bool,
    pub wire_api: prism_core::provider::WireApi,
    pub upstream_presentation: prism_core::presentation::UpstreamPresentationConfig,
    pub auth_profiles: Vec<AuthProfileSummary>,
}

#[derive(Debug, Serialize)]
pub(super) struct ProviderDetailResponse {
    pub name: String,
    pub format: String,
    pub upstream: String,
    pub api_key_masked: String,
    pub base_url: Option<String>,
    pub proxy_url: Option<String>,
    pub prefix: Option<String>,
    pub models: Vec<prism_core::config::ModelMapping>,
    pub excluded_models: Vec<String>,
    pub headers: HashMap<String, String>,
    pub disabled: bool,
    pub wire_api: prism_core::provider::WireApi,
    pub weight: u32,
    pub region: Option<String>,
    pub upstream_presentation: prism_core::presentation::UpstreamPresentationConfig,
    pub vertex: bool,
    pub vertex_project: Option<String>,
    pub vertex_location: Option<String>,
    pub auth_profiles: Vec<AuthProfileSummary>,
}

#[derive(Debug, Serialize)]
pub(super) struct AuthProfileSummary {
    pub id: String,
    pub qualified_name: String,
    pub mode: prism_core::auth_profile::AuthMode,
    pub header: prism_core::auth_profile::AuthHeaderKind,
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
    pub upstream_presentation: prism_core::presentation::UpstreamPresentationConfig,
}
