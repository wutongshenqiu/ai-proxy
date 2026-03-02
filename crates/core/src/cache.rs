use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Trait: pluggable cache backend (exact/semantic/Redis etc.).
#[async_trait]
pub trait ResponseCacheBackend: Send + Sync {
    async fn get(&self, key: &CacheKey) -> Option<CachedResponse>;
    async fn insert(&self, key: CacheKey, response: CachedResponse);
    async fn invalidate(&self, key: &CacheKey);
    async fn clear(&self);
    fn stats(&self) -> CacheStats;
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CacheKey([u8; 32]);

#[derive(Clone)]
pub struct CachedResponse {
    pub payload: Bytes,
    pub provider: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: u64,
    pub hit_rate: f64,
}

impl CacheKey {
    /// Build a cache key from model name and request body.
    /// Returns None for non-cacheable requests (streaming, non-zero temperature).
    pub fn build(model: &str, body: &serde_json::Value) -> Option<Self> {
        // Don't cache streaming requests
        if body
            .get("stream")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return None;
        }

        // Don't cache when temperature is non-zero
        if let Some(temp) = body.get("temperature").and_then(|v| v.as_f64())
            && temp != 0.0
        {
            return None;
        }

        let mut hasher = sha2::Sha256::new();
        hasher.update(model.as_bytes());

        // Canonicalize relevant fields
        let fields = [
            "messages",
            "temperature",
            "top_p",
            "max_tokens",
            "tools",
            "response_format",
        ];
        for field in &fields {
            if let Some(val) = body.get(*field) {
                hasher.update(field.as_bytes());
                hasher.update(val.to_string().as_bytes());
            }
        }

        let hash: [u8; 32] = hasher.finalize().into();
        Some(CacheKey(hash))
    }
}

/// Default implementation: Moka LRU cache.
pub struct MokaCache {
    inner: moka::future::Cache<CacheKey, CachedResponse>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl MokaCache {
    pub fn new(config: &CacheConfig) -> Self {
        let cache = moka::future::Cache::builder()
            .max_capacity(config.max_entries)
            .time_to_live(Duration::from_secs(config.ttl_secs))
            .build();
        Self {
            inner: cache,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl ResponseCacheBackend for MokaCache {
    async fn get(&self, key: &CacheKey) -> Option<CachedResponse> {
        match self.inner.get(key).await {
            Some(resp) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(resp)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    async fn insert(&self, key: CacheKey, response: CachedResponse) {
        self.inner.insert(key, response).await;
    }

    async fn invalidate(&self, key: &CacheKey) {
        self.inner.invalidate(key).await;
    }

    async fn clear(&self) {
        self.inner.invalidate_all();
    }

    fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        CacheStats {
            hits,
            misses,
            entries: self.inner.entry_count(),
            hit_rate: if total > 0 {
                hits as f64 / total as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_entries: u64,
    pub ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_entries: 10_000,
            ttl_secs: 3600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_deterministic() {
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "hello"}],
            "temperature": 0,
            "stream": false,
        });
        let k1 = CacheKey::build("gpt-4", &body);
        let k2 = CacheKey::build("gpt-4", &body);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_cache_key_different_models() {
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "hello"}],
            "temperature": 0,
        });
        let k1 = CacheKey::build("gpt-4", &body);
        let k2 = CacheKey::build("gpt-3.5", &body);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_cache_key_none_for_stream() {
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "hello"}],
            "stream": true,
        });
        assert!(CacheKey::build("gpt-4", &body).is_none());
    }

    #[test]
    fn test_cache_key_none_for_nonzero_temperature() {
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "hello"}],
            "temperature": 0.7,
        });
        assert!(CacheKey::build("gpt-4", &body).is_none());
    }

    #[test]
    fn test_cache_key_allows_zero_temperature() {
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "hello"}],
            "temperature": 0.0,
        });
        assert!(CacheKey::build("gpt-4", &body).is_some());
    }

    #[tokio::test]
    async fn test_moka_cache_basic() {
        let config = CacheConfig {
            enabled: true,
            max_entries: 100,
            ttl_secs: 3600,
        };
        let cache = MokaCache::new(&config);

        let key = CacheKey([0u8; 32]);
        let response = CachedResponse {
            payload: Bytes::from("test"),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input_tokens: 10,
            output_tokens: 20,
        };

        assert!(cache.get(&key).await.is_none());
        assert_eq!(cache.stats().misses, 1);

        cache.insert(key.clone(), response).await;
        let cached = cache.get(&key).await;
        assert!(cached.is_some());
        assert_eq!(cache.stats().hits, 1);
    }
}
