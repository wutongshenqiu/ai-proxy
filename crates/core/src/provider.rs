use crate::error::ProxyError;
use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use tokio_stream::Stream;

/// Supported provider/API format identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    OpenAI,
    Claude,
    Gemini,
    OpenAICompat,
}

impl Format {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Claude => "claude",
            Self::Gemini => "gemini",
            Self::OpenAICompat => "openai-compat",
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openai" => Ok(Self::OpenAI),
            "claude" => Ok(Self::Claude),
            "gemini" => Ok(Self::Gemini),
            "openai-compat" | "openai_compat" => Ok(Self::OpenAICompat),
            _ => Err(format!("unknown format: {s}")),
        }
    }
}

/// Wire API format for OpenAI-compatible providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WireApi {
    #[default]
    Chat,
    Responses,
}

/// Credentials for executing a request against a specific provider.
#[derive(Debug, Clone)]
pub struct AuthRecord {
    pub id: String,
    pub provider: Format,
    pub api_key: String,
    pub base_url: Option<String>,
    pub proxy_url: Option<String>,
    pub headers: HashMap<String, String>,
    pub models: Vec<ModelEntry>,
    pub excluded_models: Vec<String>,
    pub prefix: Option<String>,
    pub disabled: bool,
    pub cooldown_until: Option<std::time::Instant>,
    pub cloak: Option<crate::cloak::CloakConfig>,
    /// Wire API format for OpenAI-compatible providers.
    pub wire_api: WireApi,
    /// Human-readable name for this credential.
    pub credential_name: Option<String>,
    /// Weight for weighted round-robin routing (default: 1).
    pub weight: u32,
}

#[derive(Debug, Clone)]
pub struct ModelEntry {
    pub id: String,
    pub alias: Option<String>,
}

impl AuthRecord {
    /// Return the base URL or the provider's default.
    pub fn base_url_or_default(&self, default: &str) -> String {
        self.base_url
            .as_deref()
            .unwrap_or(default)
            .trim_end_matches('/')
            .to_string()
    }

    /// Resolve the effective proxy URL (entry-level → global fallback).
    pub fn effective_proxy<'a>(&'a self, global_proxy: Option<&'a str>) -> Option<&'a str> {
        crate::proxy::resolve_proxy_url(self.proxy_url.as_deref(), global_proxy)
    }

    /// Check whether this auth record supports the given model name.
    /// If a prefix is set, the model name must start with the prefix,
    /// and matching is done against the name after stripping the prefix.
    /// Model IDs support glob patterns (e.g., "gemini-*", "*flash*").
    pub fn supports_model(&self, model: &str) -> bool {
        let effective_model = self.strip_prefix(model);

        // If no explicit model list, support everything not excluded
        if self.models.is_empty() {
            return !self.is_model_excluded(effective_model);
        }
        let found = self.models.iter().any(|m| {
            crate::glob::glob_match(&m.id, effective_model)
                || m.alias
                    .as_deref()
                    .is_some_and(|a| crate::glob::glob_match(a, effective_model))
        });
        found && !self.is_model_excluded(effective_model)
    }

    /// Resolve the actual model ID from a possibly-aliased model name.
    /// Strips prefix, then checks if the name matches an alias and returns the real ID.
    pub fn resolve_model_id(&self, model: &str) -> String {
        let effective = self.strip_prefix(model);
        for m in &self.models {
            if m.alias.as_deref() == Some(effective) {
                return m.id.clone();
            }
            if m.id == effective {
                return m.id.clone();
            }
        }
        effective.to_string()
    }

    /// Strip the prefix from a model name. If the model doesn't have the prefix,
    /// returns the original name (for backward compatibility with no-prefix entries).
    pub fn strip_prefix<'a>(&self, model: &'a str) -> &'a str {
        if let Some(ref prefix) = self.prefix {
            model.strip_prefix(prefix.as_str()).unwrap_or(model)
        } else {
            model
        }
    }

    /// Get the prefixed model name for display/routing.
    pub fn prefixed_model_id(&self, model_id: &str) -> String {
        if let Some(ref prefix) = self.prefix {
            format!("{prefix}{model_id}")
        } else {
            model_id.to_string()
        }
    }

    /// Check if a model is in the exclusion list (supports glob wildcard matching).
    pub fn is_model_excluded(&self, model: &str) -> bool {
        self.excluded_models
            .iter()
            .any(|pattern| crate::glob::glob_match(pattern, model))
    }

    /// Get human-readable name for this credential.
    pub fn name(&self) -> Option<&str> {
        self.credential_name.as_deref()
    }

    /// Check if this credential is currently available.
    pub fn is_available(&self) -> bool {
        if self.disabled {
            return false;
        }
        if let Some(until) = self.cooldown_until
            && std::time::Instant::now() < until
        {
            return false;
        }
        true
    }
}

/// A request to be executed by a provider.
#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub model: String,
    pub payload: Bytes,
    pub source_format: Format,
    pub stream: bool,
    pub headers: HashMap<String, String>,
    pub original_request: Option<Bytes>,
}

/// A non-streaming response from a provider.
#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub payload: Bytes,
    pub headers: HashMap<String, String>,
}

/// A single chunk in a streaming response.
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// SSE event type (e.g. "message_start" for Claude).
    pub event_type: Option<String>,
    /// The JSON data payload.
    pub data: String,
}

/// The result of a streaming provider execution.
pub struct StreamResult {
    pub headers: HashMap<String, String>,
    pub stream: Pin<Box<dyn Stream<Item = Result<StreamChunk, ProxyError>> + Send>>,
}

/// Model info exposed via /v1/models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
    pub owned_by: String,
}

/// Trait for provider executors that handle forwarding requests to upstream APIs.
#[async_trait]
pub trait ProviderExecutor: Send + Sync {
    /// Unique identifier for this provider (e.g., "claude", "openai", "gemini").
    fn identifier(&self) -> &str;

    /// The native format of this provider.
    fn native_format(&self) -> Format;

    /// Default base URL for this provider.
    fn default_base_url(&self) -> &str;

    /// Execute a non-streaming request.
    async fn execute(
        &self,
        auth: &AuthRecord,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProxyError>;

    /// Execute a streaming request.
    async fn execute_stream(
        &self,
        auth: &AuthRecord,
        request: ProviderRequest,
    ) -> Result<StreamResult, ProxyError>;

    /// Return the list of models supported by this provider (based on auth records).
    fn supported_models(&self, auth: &AuthRecord) -> Vec<ModelInfo>;
}
