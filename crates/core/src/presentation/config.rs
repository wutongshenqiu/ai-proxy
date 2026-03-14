use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Client identity profile kind.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileKind {
    #[default]
    Native,
    ClaudeCode,
    GeminiCli,
    CodexCli,
}

impl std::fmt::Display for ProfileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native => write!(f, "native"),
            Self::ClaudeCode => write!(f, "claude-code"),
            Self::GeminiCli => write!(f, "gemini-cli"),
            Self::CodexCli => write!(f, "codex-cli"),
        }
    }
}

/// Profile activation mode.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActivationMode {
    #[default]
    Always,
    Auto,
}

/// Upstream presentation configuration per provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct UpstreamPresentationConfig {
    pub profile: ProfileKind,
    pub mode: ActivationMode,
    /// Replace (true) or prepend (false) user's system prompt (claude-code only).
    pub strict_mode: bool,
    /// Words to obfuscate with zero-width spaces (claude-code only).
    pub sensitive_words: Vec<String>,
    /// Cache generated user_id per API key (claude-code only).
    pub cache_user_id: bool,
    /// Raw header overrides (advanced escape hatch).
    pub custom_headers: HashMap<String, String>,
}

impl Default for UpstreamPresentationConfig {
    fn default() -> Self {
        Self {
            profile: ProfileKind::Native,
            mode: ActivationMode::Always,
            strict_mode: false,
            sensitive_words: Vec::new(),
            cache_user_id: false,
            custom_headers: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UpstreamPresentationConfig::default();
        assert_eq!(config.profile, ProfileKind::Native);
        assert_eq!(config.mode, ActivationMode::Always);
        assert!(!config.strict_mode);
        assert!(config.sensitive_words.is_empty());
        assert!(!config.cache_user_id);
        assert!(config.custom_headers.is_empty());
    }

    #[test]
    fn test_profile_kind_display() {
        assert_eq!(ProfileKind::Native.to_string(), "native");
        assert_eq!(ProfileKind::ClaudeCode.to_string(), "claude-code");
        assert_eq!(ProfileKind::GeminiCli.to_string(), "gemini-cli");
        assert_eq!(ProfileKind::CodexCli.to_string(), "codex-cli");
    }

    #[test]
    fn test_yaml_round_trip() {
        let yaml = r#"
profile: claude-code
mode: auto
strict-mode: true
sensitive-words:
  - proxy
  - prism
cache-user-id: true
custom-headers:
  x-custom: value
"#;
        let config: UpstreamPresentationConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.profile, ProfileKind::ClaudeCode);
        assert_eq!(config.mode, ActivationMode::Auto);
        assert!(config.strict_mode);
        assert_eq!(config.sensitive_words, vec!["proxy", "prism"]);
        assert!(config.cache_user_id);
        assert_eq!(config.custom_headers.get("x-custom").unwrap(), "value");
    }

    #[test]
    fn test_yaml_defaults() {
        let yaml = "{}";
        let config: UpstreamPresentationConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.profile, ProfileKind::Native);
        assert_eq!(config.mode, ActivationMode::Always);
    }
}
