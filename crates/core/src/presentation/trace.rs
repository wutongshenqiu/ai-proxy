use serde::{Deserialize, Serialize};

/// Provenance tracking for debugging/preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationTrace {
    pub profile: String,
    pub activated: bool,
    pub headers: Vec<HeaderProvenance>,
    pub body_mutations: Vec<MutationRecord>,
    pub protected_blocked: Vec<String>,
}

impl PresentationTrace {
    pub fn new(profile: &str) -> Self {
        Self {
            profile: profile.to_string(),
            activated: false,
            headers: Vec::new(),
            body_mutations: Vec::new(),
            protected_blocked: Vec::new(),
        }
    }

    /// Format as a compact debug header value.
    pub fn to_debug_header(&self) -> String {
        format!(
            "profile={},activated={},headers={},mutations={},blocked={}",
            self.profile,
            self.activated,
            self.headers.len(),
            self.body_mutations.len(),
            self.protected_blocked.len(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderProvenance {
    pub name: String,
    pub value: String,
    pub source: HeaderSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HeaderSource {
    Profile,
    CustomOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationRecord {
    pub kind: MutationKind,
    pub applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MutationKind {
    SystemPromptInjection,
    UserIdGeneration,
    SensitiveWordObfuscation,
}
