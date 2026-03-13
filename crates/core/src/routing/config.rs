use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Top-level routing config ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct RoutingConfig {
    /// Default profile name (must exist in profiles map).
    pub default_profile: String,
    /// Named routing profiles.
    pub profiles: HashMap<String, RouteProfile>,
    /// Request-to-profile matching rules (evaluated by specificity then order).
    #[serde(default)]
    pub rules: Vec<RouteRule>,
    /// Model resolution config (aliases, rewrites, fallbacks, provider pins).
    #[serde(default)]
    pub model_resolution: ModelResolution,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            default_profile: "balanced".to_string(),
            profiles: Self::default_profiles(),
            rules: Vec::new(),
            model_resolution: ModelResolution::default(),
        }
    }
}

impl RoutingConfig {
    /// Build the 4 preset profiles.
    pub fn default_profiles() -> HashMap<String, RouteProfile> {
        let mut profiles = HashMap::new();
        profiles.insert("balanced".to_string(), RouteProfile::balanced());
        profiles.insert("stable".to_string(), RouteProfile::stable());
        profiles.insert("lowest-latency".to_string(), RouteProfile::lowest_latency());
        profiles.insert("lowest-cost".to_string(), RouteProfile::lowest_cost());
        profiles
    }

    /// Validate the config for internal consistency.
    pub fn validate(&self) -> Result<(), String> {
        if self.profiles.is_empty() {
            return Err("profiles must not be empty".to_string());
        }
        if !self.profiles.contains_key(&self.default_profile) {
            return Err(format!(
                "default-profile '{}' not found in profiles",
                self.default_profile
            ));
        }
        for rule in &self.rules {
            if !self.profiles.contains_key(&rule.use_profile) {
                return Err(format!(
                    "rule '{}' references non-existent profile '{}'",
                    rule.name, rule.use_profile
                ));
            }
        }
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .map_err(|e| format!("profile '{}': {}", name, e))?;
        }
        Ok(())
    }
}

// ─── Route profile ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct RouteProfile {
    pub provider_policy: ProviderPolicy,
    pub credential_policy: CredentialPolicy,
    pub health: HealthConfig,
    pub failover: FailoverConfig,
}

impl Default for RouteProfile {
    fn default() -> Self {
        Self::balanced()
    }
}

impl RouteProfile {
    pub fn balanced() -> Self {
        Self {
            provider_policy: ProviderPolicy {
                strategy: ProviderStrategy::WeightedRoundRobin,
                ..Default::default()
            },
            credential_policy: CredentialPolicy {
                strategy: CredentialStrategy::PriorityWeightedRR,
            },
            health: HealthConfig::default(),
            failover: FailoverConfig {
                credential_attempts: 2,
                provider_attempts: 2,
                model_attempts: 2,
                ..Default::default()
            },
        }
    }

    pub fn stable() -> Self {
        Self {
            provider_policy: ProviderPolicy {
                strategy: ProviderStrategy::OrderedFallback,
                ..Default::default()
            },
            credential_policy: CredentialPolicy {
                strategy: CredentialStrategy::FillFirst,
            },
            health: HealthConfig::default(),
            failover: FailoverConfig {
                credential_attempts: 1,
                provider_attempts: 1,
                model_attempts: 1,
                ..Default::default()
            },
        }
    }

    pub fn lowest_latency() -> Self {
        Self {
            provider_policy: ProviderPolicy {
                strategy: ProviderStrategy::EwmaLatency,
                ..Default::default()
            },
            credential_policy: CredentialPolicy {
                strategy: CredentialStrategy::LeastInflight,
            },
            health: HealthConfig::default(),
            failover: FailoverConfig {
                credential_attempts: 2,
                provider_attempts: 2,
                model_attempts: 1,
                ..Default::default()
            },
        }
    }

    pub fn lowest_cost() -> Self {
        Self {
            provider_policy: ProviderPolicy {
                strategy: ProviderStrategy::LowestEstimatedCost,
                ..Default::default()
            },
            credential_policy: CredentialPolicy {
                strategy: CredentialStrategy::PriorityWeightedRR,
            },
            health: HealthConfig::default(),
            failover: FailoverConfig {
                credential_attempts: 1,
                provider_attempts: 2,
                model_attempts: 1,
                ..Default::default()
            },
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.provider_policy.validate()?;
        Ok(())
    }
}

// ─── Provider policy ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct ProviderPolicy {
    pub strategy: ProviderStrategy,
    /// Sticky key expression (e.g. "tenant-id").
    pub sticky_key: Option<String>,
    /// Provider weights (provider name -> weight).
    #[serde(default)]
    pub weights: HashMap<String, u32>,
    /// Explicit provider ordering (for ordered-fallback).
    #[serde(default)]
    pub order: Vec<String>,
}

impl Default for ProviderPolicy {
    fn default() -> Self {
        Self {
            strategy: ProviderStrategy::WeightedRoundRobin,
            sticky_key: None,
            weights: HashMap::new(),
            order: Vec::new(),
        }
    }
}

impl ProviderPolicy {
    pub fn validate(&self) -> Result<(), String> {
        match self.strategy {
            ProviderStrategy::OrderedFallback => {
                // Empty order is valid — means "all providers in config order"
            }
            ProviderStrategy::StickyHash => {
                if self.sticky_key.is_none() {
                    return Err("sticky-hash strategy requires 'sticky-key' to be set".to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderStrategy {
    OrderedFallback,
    #[default]
    WeightedRoundRobin,
    EwmaLatency,
    LowestEstimatedCost,
    StickyHash,
}

// ─── Credential policy ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct CredentialPolicy {
    pub strategy: CredentialStrategy,
}

impl Default for CredentialPolicy {
    fn default() -> Self {
        Self {
            strategy: CredentialStrategy::PriorityWeightedRR,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CredentialStrategy {
    #[default]
    #[serde(rename = "priority-weighted-rr")]
    PriorityWeightedRR,
    FillFirst,
    LeastInflight,
    EwmaLatency,
    StickyHash,
    RandomTwoChoices,
}

// ─── Health config ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct HealthConfig {
    pub circuit_breaker: CircuitBreakerHealthConfig,
    pub outlier_detection: OutlierDetectionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct CircuitBreakerHealthConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub cooldown_seconds: u64,
}

impl Default for CircuitBreakerHealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 5,
            cooldown_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct OutlierDetectionConfig {
    pub consecutive_5xx: u32,
    pub consecutive_local_failures: u32,
    pub base_eject_seconds: u64,
    pub max_eject_seconds: u64,
}

impl Default for OutlierDetectionConfig {
    fn default() -> Self {
        Self {
            consecutive_5xx: 3,
            consecutive_local_failures: 2,
            base_eject_seconds: 30,
            max_eject_seconds: 300,
        }
    }
}

// ─── Failover config ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct FailoverConfig {
    pub credential_attempts: u32,
    pub provider_attempts: u32,
    pub model_attempts: u32,
    pub retry_budget: RetryBudgetConfig,
    #[serde(default)]
    pub retry_on: Vec<RetryCondition>,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            credential_attempts: 2,
            provider_attempts: 2,
            model_attempts: 2,
            retry_budget: RetryBudgetConfig::default(),
            retry_on: vec![
                RetryCondition::Network,
                RetryCondition::RateLimit,
                RetryCondition::ServerError,
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct RetryBudgetConfig {
    /// Max ratio of retries to total requests.
    pub ratio: f64,
    /// Minimum retries per second regardless of ratio.
    pub min_retries_per_second: u32,
}

impl Default for RetryBudgetConfig {
    fn default() -> Self {
        Self {
            ratio: 0.2,
            min_retries_per_second: 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetryCondition {
    Network,
    #[serde(alias = "429")]
    RateLimit,
    #[serde(alias = "5xx")]
    ServerError,
}

// ─── Route rules ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RouteRule {
    pub name: String,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(rename = "match")]
    pub match_conditions: RouteMatch,
    pub use_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct RouteMatch {
    /// Glob patterns for model names.
    #[serde(default)]
    pub models: Vec<String>,
    /// Glob patterns for tenant IDs.
    #[serde(default)]
    pub tenants: Vec<String>,
    /// Endpoint names (e.g. "chat-completions", "messages").
    #[serde(default)]
    pub endpoints: Vec<String>,
    /// Region identifiers.
    #[serde(default)]
    pub regions: Vec<String>,
    /// Stream mode filter.
    #[serde(default)]
    pub stream: Option<bool>,
    /// Header match conditions.
    #[serde(default)]
    pub headers: HashMap<String, Vec<String>>,
}

// ─── Model resolution ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct ModelResolution {
    #[serde(default)]
    pub aliases: Vec<ModelAlias>,
    #[serde(default)]
    pub rewrites: Vec<ModelRewrite>,
    #[serde(default)]
    pub fallbacks: Vec<ModelFallback>,
    #[serde(default)]
    pub provider_pins: Vec<ProviderPin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModelAlias {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModelRewrite {
    /// Glob pattern to match incoming model names.
    pub pattern: String,
    /// Target model name to rewrite to.
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModelFallback {
    /// Glob pattern to match the primary model.
    pub pattern: String,
    /// Ordered fallback model names.
    pub to: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProviderPin {
    /// Glob pattern to match model names.
    pub pattern: String,
    /// Only these providers can serve the matched models.
    pub providers: Vec<String>,
}

// ─── Convenience methods (bridge for existing dispatch code) ─────────────────

impl RoutingConfig {
    /// Resolve server-side fallback models for a given model.
    /// Uses the model-resolution fallback config.
    pub fn resolve_fallbacks(&self, model: &str) -> Vec<String> {
        for fb in &self.model_resolution.fallbacks {
            if crate::glob::glob_match(&fb.pattern, model) {
                return fb.to.clone();
            }
        }
        Vec::new()
    }

    /// Apply model rewrite rules. Returns the target model name if a rule matches.
    pub fn resolve_model_rewrite(&self, model: &str) -> Option<&str> {
        // Check aliases first (exact match)
        for alias in &self.model_resolution.aliases {
            if alias.from == model {
                return Some(&alias.to);
            }
        }
        // Then rewrites (glob match)
        for rewrite in &self.model_resolution.rewrites {
            if crate::glob::glob_match(&rewrite.pattern, model) {
                return Some(&rewrite.to);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RoutingConfig::default();
        assert_eq!(config.default_profile, "balanced");
        assert_eq!(config.profiles.len(), 4);
        assert!(config.profiles.contains_key("balanced"));
        assert!(config.profiles.contains_key("stable"));
        assert!(config.profiles.contains_key("lowest-latency"));
        assert!(config.profiles.contains_key("lowest-cost"));
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_preset_balanced() {
        let profile = RouteProfile::balanced();
        assert_eq!(
            profile.provider_policy.strategy,
            ProviderStrategy::WeightedRoundRobin
        );
        assert_eq!(
            profile.credential_policy.strategy,
            CredentialStrategy::PriorityWeightedRR
        );
        assert_eq!(profile.failover.credential_attempts, 2);
        assert_eq!(profile.failover.provider_attempts, 2);
        assert_eq!(profile.failover.model_attempts, 2);
    }

    #[test]
    fn test_preset_stable() {
        let profile = RouteProfile::stable();
        assert_eq!(
            profile.provider_policy.strategy,
            ProviderStrategy::OrderedFallback
        );
        assert_eq!(
            profile.credential_policy.strategy,
            CredentialStrategy::FillFirst
        );
        assert_eq!(profile.failover.credential_attempts, 1);
    }

    #[test]
    fn test_preset_lowest_latency() {
        let profile = RouteProfile::lowest_latency();
        assert_eq!(
            profile.provider_policy.strategy,
            ProviderStrategy::EwmaLatency
        );
        assert_eq!(
            profile.credential_policy.strategy,
            CredentialStrategy::LeastInflight
        );
    }

    #[test]
    fn test_preset_lowest_cost() {
        let profile = RouteProfile::lowest_cost();
        assert_eq!(
            profile.provider_policy.strategy,
            ProviderStrategy::LowestEstimatedCost
        );
    }

    #[test]
    fn test_yaml_round_trip() {
        let config = RoutingConfig::default();
        let yaml = serde_yaml_ng::to_string(&config).unwrap();
        let parsed: RoutingConfig = serde_yaml_ng::from_str(&yaml).unwrap();
        assert_eq!(parsed.default_profile, config.default_profile);
        assert_eq!(parsed.profiles.len(), config.profiles.len());
    }

    #[test]
    fn test_yaml_deserialization_full() {
        let yaml = r#"
default-profile: balanced
profiles:
  balanced:
    provider-policy:
      strategy: weighted-round-robin
      weights:
        openai: 100
        claude: 100
    credential-policy:
      strategy: priority-weighted-rr
    health:
      circuit-breaker:
        enabled: true
        failure-threshold: 5
        cooldown-seconds: 30
      outlier-detection:
        consecutive-5xx: 3
        consecutive-local-failures: 2
        base-eject-seconds: 30
        max-eject-seconds: 300
    failover:
      credential-attempts: 2
      provider-attempts: 2
      model-attempts: 2
      retry-budget:
        ratio: 0.2
        min-retries-per-second: 5
      retry-on:
        - network
        - 429
        - 5xx
rules:
  - name: enterprise-latency
    match:
      tenants: ["enterprise-*"]
      models: ["gpt-*", "claude-*"]
    use-profile: balanced
model-resolution:
  aliases:
    - from: gpt-5-default
      to: gpt-5
  rewrites:
    - pattern: "claude-opus-*"
      to: "claude-sonnet-4-5"
  fallbacks:
    - pattern: "gpt-5"
      to: ["gpt-5-mini", "claude-sonnet-4-5"]
  provider-pins:
    - pattern: "gemini-*"
      providers: [gemini]
"#;
        let config: RoutingConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.default_profile, "balanced");
        assert_eq!(config.profiles.len(), 1);

        let balanced = &config.profiles["balanced"];
        assert_eq!(
            balanced.provider_policy.strategy,
            ProviderStrategy::WeightedRoundRobin
        );
        assert_eq!(
            *balanced.provider_policy.weights.get("openai").unwrap(),
            100
        );
        assert_eq!(balanced.failover.retry_on.len(), 3);

        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].name, "enterprise-latency");
        assert_eq!(config.rules[0].use_profile, "balanced");
        assert_eq!(
            config.rules[0].match_conditions.tenants,
            vec!["enterprise-*"]
        );

        assert_eq!(config.model_resolution.aliases.len(), 1);
        assert_eq!(config.model_resolution.aliases[0].from, "gpt-5-default");
        assert_eq!(config.model_resolution.rewrites.len(), 1);
        assert_eq!(config.model_resolution.fallbacks.len(), 1);
        assert_eq!(config.model_resolution.provider_pins.len(), 1);
    }

    #[test]
    fn test_yaml_minimal_defaults() {
        let yaml = "{}";
        let config: RoutingConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.default_profile, "balanced");
        assert_eq!(config.profiles.len(), 4);
    }

    #[test]
    fn test_validate_valid() {
        let config = RoutingConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_missing_default_profile() {
        let config = RoutingConfig {
            default_profile: "nonexistent".to_string(),
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert!(err.contains("default-profile"));
    }

    #[test]
    fn test_validate_empty_profiles() {
        let config = RoutingConfig {
            profiles: HashMap::new(),
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert!(err.contains("empty"));
    }

    #[test]
    fn test_validate_rule_references_missing_profile() {
        let mut config = RoutingConfig::default();
        config.rules.push(RouteRule {
            name: "bad-rule".to_string(),
            priority: None,
            match_conditions: RouteMatch::default(),
            use_profile: "nonexistent".to_string(),
        });
        let err = config.validate().unwrap_err();
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn test_validate_ordered_fallback_empty_order_ok() {
        let mut config = RoutingConfig::default();
        config.profiles.insert(
            "ordered".to_string(),
            RouteProfile {
                provider_policy: ProviderPolicy {
                    strategy: ProviderStrategy::OrderedFallback,
                    order: vec![], // empty is valid — means all providers in config order
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_sticky_hash_requires_key() {
        let mut config = RoutingConfig::default();
        config.profiles.insert(
            "bad".to_string(),
            RouteProfile {
                provider_policy: ProviderPolicy {
                    strategy: ProviderStrategy::StickyHash,
                    sticky_key: None, // missing!
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        let err = config.validate().unwrap_err();
        assert!(err.contains("sticky-key"));
    }

    #[test]
    fn test_resolve_model_rewrite_alias() {
        let config = RoutingConfig {
            model_resolution: ModelResolution {
                aliases: vec![ModelAlias {
                    from: "gpt-5-default".to_string(),
                    to: "gpt-5".to_string(),
                }],
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(config.resolve_model_rewrite("gpt-5-default"), Some("gpt-5"));
        assert_eq!(config.resolve_model_rewrite("gpt-5"), None);
    }

    #[test]
    fn test_resolve_model_rewrite_glob() {
        let config = RoutingConfig {
            model_resolution: ModelResolution {
                rewrites: vec![ModelRewrite {
                    pattern: "claude-opus-*".to_string(),
                    to: "claude-sonnet-4-5".to_string(),
                }],
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            config.resolve_model_rewrite("claude-opus-2025"),
            Some("claude-sonnet-4-5")
        );
        assert_eq!(config.resolve_model_rewrite("claude-sonnet-4-5"), None);
    }

    #[test]
    fn test_resolve_fallbacks() {
        let config = RoutingConfig {
            model_resolution: ModelResolution {
                fallbacks: vec![ModelFallback {
                    pattern: "gpt-5".to_string(),
                    to: vec!["gpt-5-mini".to_string(), "claude-sonnet".to_string()],
                }],
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            config.resolve_fallbacks("gpt-5"),
            vec!["gpt-5-mini", "claude-sonnet"]
        );
        assert!(config.resolve_fallbacks("nonexistent").is_empty());
    }

    #[test]
    fn test_retry_condition_serde() {
        let yaml = r#"["network", "429", "5xx"]"#;
        let conditions: Vec<RetryCondition> = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(conditions.len(), 3);
        assert_eq!(conditions[0], RetryCondition::Network);
        assert_eq!(conditions[1], RetryCondition::RateLimit);
        assert_eq!(conditions[2], RetryCondition::ServerError);
    }

    #[test]
    fn test_provider_strategy_serde() {
        let yaml = r#""ewma-latency""#;
        let s: ProviderStrategy = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(s, ProviderStrategy::EwmaLatency);

        let yaml = r#""lowest-estimated-cost""#;
        let s: ProviderStrategy = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(s, ProviderStrategy::LowestEstimatedCost);
    }

    #[test]
    fn test_credential_strategy_serde() {
        let yaml = r#""least-inflight""#;
        let s: CredentialStrategy = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(s, CredentialStrategy::LeastInflight);

        let yaml = r#""random-two-choices""#;
        let s: CredentialStrategy = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(s, CredentialStrategy::RandomTwoChoices);
    }

    #[test]
    fn test_default_failover_has_all_retry_conditions() {
        let config = FailoverConfig::default();
        assert_eq!(config.retry_on.len(), 3);
        assert!(config.retry_on.contains(&RetryCondition::Network));
        assert!(config.retry_on.contains(&RetryCondition::RateLimit));
        assert!(config.retry_on.contains(&RetryCondition::ServerError));
    }

    #[test]
    fn test_health_config_defaults() {
        let config = HealthConfig::default();
        assert!(config.circuit_breaker.enabled);
        assert_eq!(config.circuit_breaker.failure_threshold, 5);
        assert_eq!(config.circuit_breaker.cooldown_seconds, 30);
        assert_eq!(config.outlier_detection.consecutive_5xx, 3);
        assert_eq!(config.outlier_detection.consecutive_local_failures, 2);
        assert_eq!(config.outlier_detection.base_eject_seconds, 30);
        assert_eq!(config.outlier_detection.max_eject_seconds, 300);
    }
}
