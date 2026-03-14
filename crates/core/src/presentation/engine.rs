use super::config::{ActivationMode, ProfileKind, UpstreamPresentationConfig};
use super::profile::{ProfileDefinition, resolve_profile};
use super::protected::is_protected;
use super::trace::{
    HeaderProvenance, HeaderSource, MutationKind, MutationRecord, PresentationTrace,
};
use crate::provider::Format;
use std::collections::HashMap;

/// Input context for presentation computation.
pub struct PresentationContext<'a> {
    pub target_format: Format,
    pub model: &'a str,
    pub user_agent: Option<&'a str>,
    pub api_key: &'a str,
}

/// Output of the presentation engine.
pub struct PresentationResult {
    pub headers: HashMap<String, String>,
    pub trace: PresentationTrace,
}

/// Apply upstream presentation to headers and payload body.
pub fn apply(
    config: &UpstreamPresentationConfig,
    context: &PresentationContext,
    payload: &mut serde_json::Value,
) -> PresentationResult {
    let profile_def = resolve_profile(&config.profile);
    let mut trace = PresentationTrace::new(&config.profile.to_string());

    // 1. Check activation
    if !should_activate(config, context, &profile_def) {
        trace.activated = false;
        // Still apply custom headers when profile is not activated
        let mut headers = config.custom_headers.clone();
        filter_protected_headers(&mut headers, &mut trace);
        return PresentationResult { headers, trace };
    }
    trace.activated = true;

    // 2. Merge headers: profile defaults → custom overrides
    let mut headers = profile_def.default_headers.clone();
    for (k, v) in &config.custom_headers {
        headers.insert(k.to_lowercase(), v.clone());
    }

    // 3. Filter protected headers
    filter_protected_headers(&mut headers, &mut trace);

    // 4. Record header provenance
    for (k, v) in &headers {
        let source = if config.custom_headers.contains_key(k) {
            HeaderSource::CustomOverride
        } else {
            HeaderSource::Profile
        };
        trace.headers.push(HeaderProvenance {
            name: k.clone(),
            value: v.clone(),
            source,
        });
    }

    // 5. Apply body mutations (if format is compatible)
    if profile_def
        .compatible_formats
        .contains(&context.target_format)
    {
        for mutation in &profile_def.body_mutations {
            apply_body_mutation(mutation, config, context, payload, &mut trace);
        }
    }

    PresentationResult { headers, trace }
}

fn filter_protected_headers(headers: &mut HashMap<String, String>, trace: &mut PresentationTrace) {
    headers.retain(|k, _| {
        if is_protected(k) {
            trace.protected_blocked.push(k.clone());
            false
        } else {
            true
        }
    });
}

fn should_activate(
    config: &UpstreamPresentationConfig,
    context: &PresentationContext,
    profile_def: &ProfileDefinition,
) -> bool {
    // Native profile always activates (it's a no-op)
    if config.profile == ProfileKind::Native {
        return true;
    }
    match config.mode {
        ActivationMode::Always => true,
        ActivationMode::Auto => {
            let ua = context.user_agent.unwrap_or("");
            !profile_def
                .auto_skip_ua_prefixes
                .iter()
                .any(|prefix| ua.starts_with(prefix))
        }
    }
}

fn apply_body_mutation(
    kind: &MutationKind,
    config: &UpstreamPresentationConfig,
    context: &PresentationContext,
    payload: &mut serde_json::Value,
    trace: &mut PresentationTrace,
) {
    match kind {
        MutationKind::SystemPromptInjection => {
            inject_system_prompt(payload, config.strict_mode);
            trace.body_mutations.push(MutationRecord {
                kind: MutationKind::SystemPromptInjection,
                applied: true,
                reason: None,
            });
        }
        MutationKind::UserIdGeneration => {
            inject_user_id(payload, context.api_key, config.cache_user_id);
            trace.body_mutations.push(MutationRecord {
                kind: MutationKind::UserIdGeneration,
                applied: true,
                reason: None,
            });
        }
        MutationKind::SensitiveWordObfuscation => {
            if config.sensitive_words.is_empty() {
                trace.body_mutations.push(MutationRecord {
                    kind: MutationKind::SensitiveWordObfuscation,
                    applied: false,
                    reason: Some("no sensitive words configured".into()),
                });
            } else {
                crate::cloak::obfuscate_sensitive_words(payload, &config.sensitive_words);
                trace.body_mutations.push(MutationRecord {
                    kind: MutationKind::SensitiveWordObfuscation,
                    applied: true,
                    reason: None,
                });
            }
        }
    }
}

/// Claude Code system prompt snippet.
const CLOAK_SYSTEM_PROMPT: &str = "You are Claude Code, Anthropic's official CLI for Claude. \
You are an interactive agent specialized in software engineering tasks. \
You help users with coding, debugging, and software development.";

fn inject_system_prompt(payload: &mut serde_json::Value, strict_mode: bool) {
    let obj = match payload.as_object_mut() {
        Some(o) => o,
        None => return,
    };

    if strict_mode {
        obj.insert(
            "system".to_string(),
            serde_json::Value::String(CLOAK_SYSTEM_PROMPT.to_string()),
        );
    } else {
        let existing = obj
            .get("system")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let combined = if existing.is_empty() {
            CLOAK_SYSTEM_PROMPT.to_string()
        } else {
            format!("{CLOAK_SYSTEM_PROMPT}\n\n{existing}")
        };
        obj.insert("system".to_string(), serde_json::Value::String(combined));
    }
}

fn inject_user_id(payload: &mut serde_json::Value, api_key: &str, cache: bool) {
    let obj = match payload.as_object_mut() {
        Some(o) => o,
        None => return,
    };

    let user_id = crate::cloak::generate_user_id(api_key, cache);
    let metadata = obj
        .entry("metadata")
        .or_insert_with(|| serde_json::json!({}));
    if let Some(meta_obj) = metadata.as_object_mut() {
        meta_obj.insert("user_id".to_string(), serde_json::Value::String(user_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_context<'a>(
        format: Format,
        ua: Option<&'a str>,
        api_key: &'a str,
    ) -> PresentationContext<'a> {
        PresentationContext {
            target_format: format,
            model: "test-model",
            user_agent: ua,
            api_key,
        }
    }

    // === Activation tests ===

    #[test]
    fn test_native_always_activates() {
        let config = UpstreamPresentationConfig::default();
        let ctx = make_context(Format::OpenAI, None, "key");
        let mut payload = json!({});
        let result = apply(&config, &ctx, &mut payload);
        assert!(result.trace.activated);
    }

    #[test]
    fn test_claude_code_always_mode() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            mode: ActivationMode::Always,
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, Some("claude-code/1.0"), "key");
        let mut payload = json!({"messages": []});
        let result = apply(&config, &ctx, &mut payload);
        assert!(result.trace.activated);
    }

    #[test]
    fn test_claude_code_auto_skips_real_client() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            mode: ActivationMode::Auto,
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, Some("claude-code/1.0.0"), "key");
        let mut payload = json!({"messages": []});
        let result = apply(&config, &ctx, &mut payload);
        assert!(!result.trace.activated);
    }

    #[test]
    fn test_claude_code_auto_applies_for_other_ua() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            mode: ActivationMode::Auto,
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, Some("python-requests/2.31.0"), "key");
        let mut payload = json!({"messages": []});
        let result = apply(&config, &ctx, &mut payload);
        assert!(result.trace.activated);
    }

    #[test]
    fn test_claude_code_auto_applies_for_no_ua() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            mode: ActivationMode::Auto,
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, None, "key");
        let mut payload = json!({"messages": []});
        let result = apply(&config, &ctx, &mut payload);
        assert!(result.trace.activated);
    }

    // === Header merge tests ===

    #[test]
    fn test_claude_code_profile_headers() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, None, "key");
        let mut payload = json!({"messages": []});
        let result = apply(&config, &ctx, &mut payload);
        assert_eq!(
            result.headers.get("user-agent").unwrap(),
            "claude-code/1.0.0"
        );
    }

    #[test]
    fn test_custom_headers_override_profile() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            custom_headers: HashMap::from([
                ("user-agent".into(), "my-agent/2.0".into()),
                ("x-extra".into(), "value".into()),
            ]),
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, None, "key");
        let mut payload = json!({"messages": []});
        let result = apply(&config, &ctx, &mut payload);
        assert_eq!(result.headers.get("user-agent").unwrap(), "my-agent/2.0");
        assert_eq!(result.headers.get("x-extra").unwrap(), "value");
    }

    #[test]
    fn test_protected_headers_blocked() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::Native,
            custom_headers: HashMap::from([
                ("authorization".into(), "Bearer evil".into()),
                ("x-api-key".into(), "evil-key".into()),
                ("x-custom".into(), "safe".into()),
            ]),
            ..Default::default()
        };
        let ctx = make_context(Format::OpenAI, None, "key");
        let mut payload = json!({});
        let result = apply(&config, &ctx, &mut payload);
        assert!(!result.headers.contains_key("authorization"));
        assert!(!result.headers.contains_key("x-api-key"));
        assert_eq!(result.headers.get("x-custom").unwrap(), "safe");
        assert!(
            result
                .trace
                .protected_blocked
                .contains(&"authorization".to_string())
        );
        assert!(
            result
                .trace
                .protected_blocked
                .contains(&"x-api-key".to_string())
        );
    }

    // === Body mutation tests ===

    #[test]
    fn test_system_prompt_prepend() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            strict_mode: false,
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, None, "key");
        let mut payload = json!({
            "model": "claude-sonnet-4-20250514",
            "messages": [{"role": "user", "content": "hello"}],
            "system": "You are a helpful assistant."
        });
        apply(&config, &ctx, &mut payload);
        let system = payload["system"].as_str().unwrap();
        assert!(system.starts_with("You are Claude Code"));
        assert!(system.contains("You are a helpful assistant."));
    }

    #[test]
    fn test_system_prompt_strict() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            strict_mode: true,
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, None, "key");
        let mut payload = json!({
            "model": "claude-sonnet-4-20250514",
            "messages": [{"role": "user", "content": "hello"}],
            "system": "You are a helpful assistant."
        });
        apply(&config, &ctx, &mut payload);
        let system = payload["system"].as_str().unwrap();
        assert!(system.starts_with("You are Claude Code"));
        assert!(!system.contains("You are a helpful assistant."));
    }

    #[test]
    fn test_user_id_generation() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, None, "test-key-for-presentation");
        let mut payload = json!({
            "model": "claude-sonnet-4-20250514",
            "messages": [{"role": "user", "content": "hello"}]
        });
        apply(&config, &ctx, &mut payload);
        let user_id = payload["metadata"]["user_id"].as_str().unwrap();
        assert!(user_id.starts_with("user_"));
        assert!(user_id.contains("_account__session_"));
    }

    #[test]
    fn test_sensitive_word_obfuscation() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            sensitive_words: vec!["proxy".into(), "prism".into()],
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, None, "key");
        let mut payload = json!({
            "model": "claude-sonnet-4-20250514",
            "messages": [{"role": "user", "content": "This proxy uses prism"}]
        });
        apply(&config, &ctx, &mut payload);
        let content = payload["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains('\u{200B}'));
        assert!(!content.contains("proxy"));
    }

    #[test]
    fn test_no_sensitive_words_skips_mutation() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, None, "key");
        let mut payload = json!({"messages": [{"role": "user", "content": "hello proxy"}]});
        let result = apply(&config, &ctx, &mut payload);
        let obfuscation = result
            .trace
            .body_mutations
            .iter()
            .find(|m| m.kind == MutationKind::SensitiveWordObfuscation)
            .unwrap();
        assert!(!obfuscation.applied);
    }

    #[test]
    fn test_body_mutations_skipped_for_incompatible_format() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            ..Default::default()
        };
        // claude-code profile targeting OpenAI format - headers applied, body mutations skipped
        let ctx = make_context(Format::OpenAI, None, "key");
        let mut payload = json!({"messages": [{"role": "user", "content": "hello"}]});
        let result = apply(&config, &ctx, &mut payload);
        assert!(result.trace.activated);
        assert!(result.headers.contains_key("user-agent"));
        assert!(result.trace.body_mutations.is_empty());
        // System prompt should NOT be injected
        assert!(payload.get("system").is_none());
    }

    // === Provenance tests ===

    #[test]
    fn test_header_provenance_tracking() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            custom_headers: HashMap::from([("x-extra".into(), "val".into())]),
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, None, "key");
        let mut payload = json!({"messages": []});
        let result = apply(&config, &ctx, &mut payload);

        let ua_prov = result
            .trace
            .headers
            .iter()
            .find(|h| h.name == "user-agent")
            .unwrap();
        assert_eq!(ua_prov.source, HeaderSource::Profile);

        let extra_prov = result
            .trace
            .headers
            .iter()
            .find(|h| h.name == "x-extra")
            .unwrap();
        assert_eq!(extra_prov.source, HeaderSource::CustomOverride);
    }

    // === Gemini/Codex profile tests ===

    #[test]
    fn test_gemini_cli_headers() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::GeminiCli,
            ..Default::default()
        };
        let ctx = make_context(Format::Gemini, None, "key");
        let mut payload = json!({});
        let result = apply(&config, &ctx, &mut payload);
        assert_eq!(
            result.headers.get("user-agent").unwrap(),
            "gemini-cli/0.1.0"
        );
        assert_eq!(
            result.headers.get("x-goog-api-client").unwrap(),
            "gemini-cli/0.1.0"
        );
    }

    #[test]
    fn test_codex_cli_headers() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::CodexCli,
            ..Default::default()
        };
        let ctx = make_context(Format::OpenAI, None, "key");
        let mut payload = json!({});
        let result = apply(&config, &ctx, &mut payload);
        assert_eq!(result.headers.get("user-agent").unwrap(), "codex-cli/1.0.0");
    }

    // === Debug header ===

    #[test]
    fn test_trace_debug_header() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, None, "key");
        let mut payload = json!({"messages": []});
        let result = apply(&config, &ctx, &mut payload);
        let header = result.trace.to_debug_header();
        assert!(header.starts_with("profile=claude-code"));
        assert!(header.contains("activated=true"));
    }

    // === Custom headers only when not activated ===

    #[test]
    fn test_custom_headers_still_apply_when_not_activated() {
        let config = UpstreamPresentationConfig {
            profile: ProfileKind::ClaudeCode,
            mode: ActivationMode::Auto,
            custom_headers: HashMap::from([("x-custom".into(), "value".into())]),
            ..Default::default()
        };
        let ctx = make_context(Format::Claude, Some("claude-code/1.0"), "key");
        let mut payload = json!({"messages": []});
        let result = apply(&config, &ctx, &mut payload);
        assert!(!result.trace.activated);
        assert_eq!(result.headers.get("x-custom").unwrap(), "value");
        // Profile headers should NOT be present
        assert!(!result.headers.contains_key("user-agent"));
    }
}
