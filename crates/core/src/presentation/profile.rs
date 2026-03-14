use super::config::ProfileKind;
use super::trace::MutationKind;
use crate::provider::Format;
use std::collections::HashMap;

/// Definition of a built-in presentation profile.
pub struct ProfileDefinition {
    pub default_headers: HashMap<String, String>,
    pub body_mutations: Vec<MutationKind>,
    pub compatible_formats: Vec<Format>,
    pub auto_skip_ua_prefixes: Vec<String>,
}

/// Resolve a profile kind to its definition.
pub fn resolve_profile(kind: &ProfileKind) -> ProfileDefinition {
    match kind {
        ProfileKind::Native => native_profile(),
        ProfileKind::ClaudeCode => claude_code_profile(),
        ProfileKind::GeminiCli => gemini_cli_profile(),
        ProfileKind::CodexCli => codex_cli_profile(),
    }
}

fn native_profile() -> ProfileDefinition {
    ProfileDefinition {
        default_headers: HashMap::new(),
        body_mutations: vec![],
        compatible_formats: vec![Format::OpenAI, Format::Claude, Format::Gemini],
        auto_skip_ua_prefixes: vec![],
    }
}

fn claude_code_profile() -> ProfileDefinition {
    ProfileDefinition {
        default_headers: HashMap::from([("user-agent".into(), "claude-code/1.0.0".into())]),
        body_mutations: vec![
            MutationKind::SystemPromptInjection,
            MutationKind::UserIdGeneration,
            MutationKind::SensitiveWordObfuscation,
        ],
        compatible_formats: vec![Format::Claude],
        auto_skip_ua_prefixes: vec!["claude-cli".into(), "claude-code".into()],
    }
}

fn gemini_cli_profile() -> ProfileDefinition {
    ProfileDefinition {
        default_headers: HashMap::from([
            ("user-agent".into(), "gemini-cli/0.1.0".into()),
            ("x-goog-api-client".into(), "gemini-cli/0.1.0".into()),
        ]),
        body_mutations: vec![],
        compatible_formats: vec![Format::Gemini],
        auto_skip_ua_prefixes: vec!["gemini-cli".into()],
    }
}

fn codex_cli_profile() -> ProfileDefinition {
    ProfileDefinition {
        default_headers: HashMap::from([("user-agent".into(), "codex-cli/1.0.0".into())]),
        body_mutations: vec![],
        compatible_formats: vec![Format::OpenAI],
        auto_skip_ua_prefixes: vec!["codex".into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_profile() {
        let def = resolve_profile(&ProfileKind::Native);
        assert!(def.default_headers.is_empty());
        assert!(def.body_mutations.is_empty());
        assert_eq!(def.compatible_formats.len(), 3);
    }

    #[test]
    fn test_claude_code_profile() {
        let def = resolve_profile(&ProfileKind::ClaudeCode);
        assert_eq!(
            def.default_headers.get("user-agent").unwrap(),
            "claude-code/1.0.0"
        );
        assert_eq!(def.body_mutations.len(), 3);
        assert_eq!(def.compatible_formats, vec![Format::Claude]);
        assert_eq!(def.auto_skip_ua_prefixes, vec!["claude-cli", "claude-code"]);
    }

    #[test]
    fn test_gemini_cli_profile() {
        let def = resolve_profile(&ProfileKind::GeminiCli);
        assert_eq!(
            def.default_headers.get("user-agent").unwrap(),
            "gemini-cli/0.1.0"
        );
        assert_eq!(
            def.default_headers.get("x-goog-api-client").unwrap(),
            "gemini-cli/0.1.0"
        );
        assert!(def.body_mutations.is_empty());
        assert_eq!(def.compatible_formats, vec![Format::Gemini]);
    }

    #[test]
    fn test_codex_cli_profile() {
        let def = resolve_profile(&ProfileKind::CodexCli);
        assert_eq!(
            def.default_headers.get("user-agent").unwrap(),
            "codex-cli/1.0.0"
        );
        assert!(def.body_mutations.is_empty());
        assert_eq!(def.compatible_formats, vec![Format::OpenAI]);
    }
}
