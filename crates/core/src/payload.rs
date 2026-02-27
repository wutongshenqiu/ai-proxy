use crate::glob::glob_match;
use serde_json::Value;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PayloadConfig {
    #[serde(default)]
    pub default: Vec<PayloadRule>,
    #[serde(default)]
    pub r#override: Vec<PayloadRule>,
    #[serde(default)]
    pub filter: Vec<FilterRule>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModelMatcher {
    pub name: String,
    #[serde(default)]
    pub protocol: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PayloadRule {
    pub models: Vec<ModelMatcher>,
    pub params: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FilterRule {
    pub models: Vec<ModelMatcher>,
    pub params: Vec<String>,
}

/// Check if a rule matches the given model and protocol.
fn matches_rule(matchers: &[ModelMatcher], model: &str, protocol: Option<&str>) -> bool {
    matchers.iter().any(|m| {
        let name_match = glob_match(&m.name, model);
        let protocol_match = m
            .protocol
            .as_ref()
            .is_none_or(|p| protocol.is_some_and(|actual| actual.eq_ignore_ascii_case(p)));
        name_match && protocol_match
    })
}

/// Set a value at a dot-separated path, creating intermediate objects as needed.
/// Returns true if the value was actually set.
fn set_nested(root: &mut Value, path: &str, value: Value, only_if_missing: bool) -> bool {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            if let Some(obj) = current.as_object_mut() {
                if only_if_missing && obj.contains_key(*part) {
                    return false;
                }
                obj.insert(part.to_string(), value);
                return true;
            }
            return false;
        } else {
            // Intermediate part - ensure object exists
            if !current.is_object() {
                return false;
            }
            let obj = current.as_object_mut().unwrap();
            if !obj.contains_key(*part) {
                obj.insert(part.to_string(), Value::Object(serde_json::Map::new()));
            }
            current = obj.get_mut(*part).unwrap();
        }
    }
    false
}

/// Remove a value at a dot-separated path.
fn remove_nested(root: &mut Value, path: &str) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let Some(obj) = current.as_object_mut() {
                obj.remove(*part);
            }
        } else {
            match current.as_object_mut().and_then(|obj| obj.get_mut(*part)) {
                Some(next) => current = next,
                None => return,
            }
        }
    }
}

/// Apply all payload rules to a JSON body.
/// `model` is the actual model name, `protocol` is the target format (e.g., "openai", "claude", "gemini").
pub fn apply_payload_rules(
    body: &mut Value,
    config: &PayloadConfig,
    model: &str,
    protocol: Option<&str>,
) {
    // 1. Apply defaults (only if field is missing)
    for rule in &config.default {
        if matches_rule(&rule.models, model, protocol) {
            for (path, value) in &rule.params {
                set_nested(body, path, value.clone(), true);
            }
        }
    }

    // 2. Apply overrides (always set)
    for rule in &config.r#override {
        if matches_rule(&rule.models, model, protocol) {
            for (path, value) in &rule.params {
                set_nested(body, path, value.clone(), false);
            }
        }
    }

    // 3. Apply filters (delete fields)
    for rule in &config.filter {
        if matches_rule(&rule.models, model, protocol) {
            for path in &rule.params {
                remove_nested(body, path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_sets_missing() {
        let mut body = json!({"model": "gemini-2.5-pro"});
        let config = PayloadConfig {
            default: vec![PayloadRule {
                models: vec![ModelMatcher {
                    name: "gemini-*".into(),
                    protocol: None,
                }],
                params: {
                    let mut m = serde_json::Map::new();
                    m.insert(
                        "generationConfig.thinkingConfig.thinkingBudget".into(),
                        json!(32768),
                    );
                    m
                },
            }],
            ..Default::default()
        };
        apply_payload_rules(&mut body, &config, "gemini-2.5-pro", Some("gemini"));
        assert_eq!(
            body["generationConfig"]["thinkingConfig"]["thinkingBudget"],
            32768
        );
    }

    #[test]
    fn test_default_does_not_overwrite() {
        let mut body = json!({"temperature": 0.5});
        let config = PayloadConfig {
            default: vec![PayloadRule {
                models: vec![ModelMatcher {
                    name: "*".into(),
                    protocol: None,
                }],
                params: {
                    let mut m = serde_json::Map::new();
                    m.insert("temperature".into(), json!(1.0));
                    m
                },
            }],
            ..Default::default()
        };
        apply_payload_rules(&mut body, &config, "any-model", None);
        assert_eq!(body["temperature"], 0.5);
    }

    #[test]
    fn test_override_always_sets() {
        let mut body = json!({"reasoning": {"effort": "low"}});
        let config = PayloadConfig {
            r#override: vec![PayloadRule {
                models: vec![ModelMatcher {
                    name: "gpt-*".into(),
                    protocol: Some("openai".into()),
                }],
                params: {
                    let mut m = serde_json::Map::new();
                    m.insert("reasoning.effort".into(), json!("high"));
                    m
                },
            }],
            ..Default::default()
        };
        apply_payload_rules(&mut body, &config, "gpt-4o", Some("openai"));
        assert_eq!(body["reasoning"]["effort"], "high");
    }

    #[test]
    fn test_filter_removes_fields() {
        let mut body = json!({
            "generationConfig": {
                "responseJsonSchema": {"type": "object"},
                "temperature": 0.7
            }
        });
        let config = PayloadConfig {
            filter: vec![FilterRule {
                models: vec![ModelMatcher {
                    name: "gemini-*".into(),
                    protocol: None,
                }],
                params: vec!["generationConfig.responseJsonSchema".into()],
            }],
            ..Default::default()
        };
        apply_payload_rules(&mut body, &config, "gemini-2.0-flash", Some("gemini"));
        assert!(body["generationConfig"].get("responseJsonSchema").is_none());
        assert_eq!(body["generationConfig"]["temperature"], 0.7);
    }

    #[test]
    fn test_protocol_filter() {
        let mut body = json!({});
        let config = PayloadConfig {
            r#override: vec![PayloadRule {
                models: vec![ModelMatcher {
                    name: "*".into(),
                    protocol: Some("openai".into()),
                }],
                params: {
                    let mut m = serde_json::Map::new();
                    m.insert("stream_options.include_usage".into(), json!(true));
                    m
                },
            }],
            ..Default::default()
        };
        // Should NOT match because protocol is "claude"
        apply_payload_rules(&mut body, &config, "any-model", Some("claude"));
        assert!(body.get("stream_options").is_none());

        // Should match because protocol is "openai"
        apply_payload_rules(&mut body, &config, "any-model", Some("openai"));
        assert_eq!(body["stream_options"]["include_usage"], true);
    }
}
