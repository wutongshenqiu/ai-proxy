use ai_proxy_core::config::{Config, RoutingStrategy};
use ai_proxy_core::provider::{AuthRecord, Format, ModelEntry, ModelInfo};
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

pub struct CredentialRouter {
    credentials: RwLock<HashMap<Format, Vec<AuthRecord>>>,
    counters: RwLock<HashMap<String, AtomicUsize>>,
    strategy: RwLock<RoutingStrategy>,
}

impl CredentialRouter {
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            credentials: RwLock::new(HashMap::new()),
            counters: RwLock::new(HashMap::new()),
            strategy: RwLock::new(strategy),
        }
    }

    /// Pick the next available credential for the given provider and model.
    /// Skips credentials whose IDs are in `tried`.
    pub fn pick(&self, provider: Format, model: &str, tried: &[String]) -> Option<AuthRecord> {
        let creds = self.credentials.read().ok()?;
        let entries = creds.get(&provider)?;

        // Filter to available credentials that support the model and haven't been tried
        let candidates: Vec<&AuthRecord> = entries
            .iter()
            .filter(|a| a.is_available() && a.supports_model(model) && !tried.contains(&a.id))
            .collect();

        if candidates.is_empty() {
            return None;
        }

        let strategy = self.strategy.read().ok()?;
        match *strategy {
            RoutingStrategy::FillFirst => {
                // Always pick the first available credential
                candidates.first().cloned().cloned()
            }
            RoutingStrategy::RoundRobin => {
                let key = format!("{}:{}", provider.as_str(), model);
                let counters = self.counters.read().ok()?;
                let idx = if let Some(counter) = counters.get(&key) {
                    counter.fetch_add(1, Ordering::Relaxed)
                } else {
                    drop(counters);
                    let mut counters = self.counters.write().ok()?;
                    let counter = counters.entry(key).or_insert_with(|| AtomicUsize::new(0));
                    counter.fetch_add(1, Ordering::Relaxed)
                };
                let picked = candidates[idx % candidates.len()];
                Some(picked.clone())
            }
        }
    }

    /// Mark a credential as unavailable for a duration (cooldown).
    pub fn mark_unavailable(&self, auth_id: &str, duration: Duration) {
        if let Ok(mut creds) = self.credentials.write() {
            let until = Instant::now() + duration;
            for entries in creds.values_mut() {
                for auth in entries.iter_mut() {
                    if auth.id == auth_id {
                        auth.cooldown_until = Some(until);
                    }
                }
            }
        }
    }

    /// Rebuild credentials from config, preserving cooldown state from existing credentials.
    pub fn update_from_config(&self, config: &Config) {
        let mut map: HashMap<Format, Vec<AuthRecord>> = HashMap::new();

        // Claude credentials
        for entry in &config.claude_api_key {
            let auth = build_auth_record(entry, Format::Claude);
            map.entry(Format::Claude).or_default().push(auth);
        }

        // OpenAI credentials
        for entry in &config.openai_api_key {
            let auth = build_auth_record(entry, Format::OpenAI);
            map.entry(Format::OpenAI).or_default().push(auth);
        }

        // Gemini credentials
        for entry in &config.gemini_api_key {
            let auth = build_auth_record(entry, Format::Gemini);
            map.entry(Format::Gemini).or_default().push(auth);
        }

        // OpenAI-compatible credentials
        for entry in &config.openai_compatibility {
            let auth = build_auth_record(entry, Format::OpenAICompat);
            map.entry(Format::OpenAICompat).or_default().push(auth);
        }

        if let Ok(mut creds) = self.credentials.write() {
            // Preserve cooldown state from existing credentials (matched by api_key + format)
            for (format, new_entries) in map.iter_mut() {
                if let Some(old_entries) = creds.get(format) {
                    for new_auth in new_entries.iter_mut() {
                        if let Some(old_auth) =
                            old_entries.iter().find(|o| o.api_key == new_auth.api_key)
                        {
                            new_auth.cooldown_until = old_auth.cooldown_until;
                        }
                    }
                }
            }
            *creds = map;
        }

        // Update strategy
        if let Ok(mut strategy) = self.strategy.write() {
            *strategy = config.routing.strategy.clone();
        }
    }

    /// Get all available models across all providers.
    pub fn all_models(&self) -> Vec<ModelInfo> {
        let mut models = Vec::new();
        if let Ok(creds) = self.credentials.read() {
            for (format, entries) in creds.iter() {
                for auth in entries {
                    if !auth.is_available() {
                        continue;
                    }
                    for model_entry in &auth.models {
                        let model_id = if let Some(ref alias) = model_entry.alias {
                            alias.clone()
                        } else {
                            model_entry.id.clone()
                        };
                        // Avoid duplicates
                        if !models.iter().any(|m: &ModelInfo| m.id == model_id) {
                            models.push(ModelInfo {
                                id: model_id,
                                provider: format.as_str().to_string(),
                                owned_by: format.as_str().to_string(),
                            });
                        }
                    }
                }
            }
        }
        models
    }

    /// Check if the model name matches any credential that has a prefix configured.
    /// Used by `force_model_prefix` to reject unprefixed model requests.
    pub fn model_has_prefix(&self, model: &str) -> bool {
        if let Ok(creds) = self.credentials.read() {
            for entries in creds.values() {
                for auth in entries {
                    if auth.prefix.is_some() && auth.is_available() && auth.supports_model(model) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Resolve which provider(s) can handle a given model name.
    pub fn resolve_providers(&self, model: &str) -> Vec<Format> {
        let mut formats = Vec::new();
        if let Ok(creds) = self.credentials.read() {
            for (format, entries) in creds.iter() {
                for auth in entries {
                    if auth.is_available() && auth.supports_model(model) {
                        if !formats.contains(format) {
                            formats.push(*format);
                        }
                        break;
                    }
                }
            }
        }
        formats
    }
}

fn build_auth_record(
    entry: &ai_proxy_core::config::ProviderKeyEntry,
    format: Format,
) -> AuthRecord {
    let models = entry
        .models
        .iter()
        .map(|m| ModelEntry {
            id: m.id.clone(),
            alias: m.alias.clone(),
        })
        .collect();

    AuthRecord {
        id: uuid::Uuid::new_v4().to_string(),
        provider: format,
        api_key: entry.api_key.clone(),
        base_url: entry.base_url.clone(),
        proxy_url: entry.proxy_url.clone(),
        headers: entry.headers.clone(),
        models,
        excluded_models: entry.excluded_models.clone(),
        prefix: entry.prefix.clone(),
        disabled: entry.disabled,
        cooldown_until: None,
        cloak: if matches!(format, Format::Claude) {
            Some(entry.cloak.clone())
        } else {
            None
        },
        wire_api: entry.wire_api,
    }
}
