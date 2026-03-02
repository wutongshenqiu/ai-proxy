use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Lightweight in-memory metrics using atomic counters.
pub struct Metrics {
    pub total_requests: AtomicU64,
    pub total_errors: AtomicU64,
    pub total_input_tokens: AtomicU64,
    pub total_output_tokens: AtomicU64,
    /// Total cost in USD (stored as millionths of a cent for atomic precision).
    total_cost_micro: AtomicU64,
    /// Per-model cost tracking.
    model_costs: Mutex<HashMap<String, f64>>,
    /// Per-model request counts.
    model_counts: RwLock<HashMap<String, AtomicU64>>,
    /// Per-provider request counts.
    provider_counts: RwLock<HashMap<String, AtomicU64>>,
    /// Latency histogram buckets (ms): <100, <500, <1000, <5000, <30000, >=30000.
    pub latency_buckets: [AtomicU64; 6],
    /// Total latency sum in ms (for computing average).
    total_latency_ms: AtomicU64,
    /// TTFT histogram buckets (ms): <50, <100, <500, <1000, <5000, >=5000.
    ttft_buckets: [AtomicU64; 6],
    /// Per-tenant request counts.
    tenant_request_counts: RwLock<HashMap<String, AtomicU64>>,
    /// Per-tenant token counts (input + output).
    tenant_token_counts: RwLock<HashMap<String, AtomicU64>>,
    /// Per-tenant cost tracking (micro-USD).
    tenant_cost_micro: RwLock<HashMap<String, AtomicU64>>,
    /// Cache hit/miss counters.
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    /// When the metrics instance was created (for uptime).
    created_at: Instant,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_input_tokens: AtomicU64::new(0),
            total_output_tokens: AtomicU64::new(0),
            total_cost_micro: AtomicU64::new(0),
            model_costs: Mutex::new(HashMap::new()),
            model_counts: RwLock::new(HashMap::new()),
            provider_counts: RwLock::new(HashMap::new()),
            latency_buckets: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
            total_latency_ms: AtomicU64::new(0),
            ttft_buckets: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
            tenant_request_counts: RwLock::new(HashMap::new()),
            tenant_token_counts: RwLock::new(HashMap::new()),
            tenant_cost_micro: RwLock::new(HashMap::new()),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            created_at: Instant::now(),
        }
    }

    pub fn record_request(&self, model: &str, provider: &str) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        increment_map(&self.model_counts, model);
        increment_map(&self.provider_counts, provider);
    }

    pub fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_latency_ms(&self, ms: u128) {
        let bucket = match ms {
            0..=99 => 0,
            100..=499 => 1,
            500..=999 => 2,
            1000..=4999 => 3,
            5000..=29999 => 4,
            _ => 5,
        };
        self.latency_buckets[bucket].fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms
            .fetch_add(ms as u64, Ordering::Relaxed);
    }

    pub fn record_tokens(&self, input: u64, output: u64) {
        self.total_input_tokens.fetch_add(input, Ordering::Relaxed);
        self.total_output_tokens
            .fetch_add(output, Ordering::Relaxed);
    }

    /// Record cost in USD for a request.
    pub fn record_cost(&self, model: &str, cost: f64) {
        // Store as micro-USD (millionths) for atomic precision
        let micro = (cost * 1_000_000.0) as u64;
        self.total_cost_micro.fetch_add(micro, Ordering::Relaxed);
        if let Ok(mut costs) = self.model_costs.lock() {
            *costs.entry(model.to_string()).or_insert(0.0) += cost;
        }
    }

    /// Record Time-To-First-Token in milliseconds.
    pub fn record_ttft_ms(&self, ms: u64) {
        let bucket = match ms {
            0..=49 => 0,
            50..=99 => 1,
            100..=499 => 2,
            500..=999 => 3,
            1000..=4999 => 4,
            _ => 5,
        };
        self.ttft_buckets[bucket].fetch_add(1, Ordering::Relaxed);
    }

    /// Record per-tenant request.
    pub fn record_tenant_request(&self, tenant_id: &str) {
        increment_map(&self.tenant_request_counts, tenant_id);
    }

    /// Record per-tenant tokens.
    pub fn record_tenant_tokens(&self, tenant_id: &str, tokens: u64) {
        increment_map_by(&self.tenant_token_counts, tenant_id, tokens);
    }

    /// Record per-tenant cost.
    pub fn record_tenant_cost(&self, tenant_id: &str, cost: f64) {
        let micro = (cost * 1_000_000.0) as u64;
        increment_map_by(&self.tenant_cost_micro, tenant_id, micro);
    }

    /// Record a cache hit.
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss.
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Raw latency bucket values for Prometheus rendering.
    pub fn latency_bucket_values(&self) -> [u64; 6] {
        [
            self.latency_buckets[0].load(Ordering::Relaxed),
            self.latency_buckets[1].load(Ordering::Relaxed),
            self.latency_buckets[2].load(Ordering::Relaxed),
            self.latency_buckets[3].load(Ordering::Relaxed),
            self.latency_buckets[4].load(Ordering::Relaxed),
            self.latency_buckets[5].load(Ordering::Relaxed),
        ]
    }

    /// Raw TTFT bucket values for Prometheus rendering.
    pub fn ttft_bucket_values(&self) -> [u64; 6] {
        [
            self.ttft_buckets[0].load(Ordering::Relaxed),
            self.ttft_buckets[1].load(Ordering::Relaxed),
            self.ttft_buckets[2].load(Ordering::Relaxed),
            self.ttft_buckets[3].load(Ordering::Relaxed),
            self.ttft_buckets[4].load(Ordering::Relaxed),
            self.ttft_buckets[5].load(Ordering::Relaxed),
        ]
    }

    /// Per-tenant metrics snapshot.
    pub fn tenant_snapshot(&self) -> serde_json::Value {
        let mut tenants = serde_json::Map::new();
        if let Ok(req_counts) = self.tenant_request_counts.read() {
            for (tenant, count) in req_counts.iter() {
                let tokens = self
                    .tenant_token_counts
                    .read()
                    .ok()
                    .and_then(|m| m.get(tenant).map(|v| v.load(Ordering::Relaxed)))
                    .unwrap_or(0);
                let cost_micro = self
                    .tenant_cost_micro
                    .read()
                    .ok()
                    .and_then(|m| m.get(tenant).map(|v| v.load(Ordering::Relaxed)))
                    .unwrap_or(0);
                tenants.insert(
                    tenant.clone(),
                    serde_json::json!({
                        "requests": count.load(Ordering::Relaxed),
                        "tokens": tokens,
                        "cost_usd": cost_micro as f64 / 1_000_000.0,
                    }),
                );
            }
        }
        serde_json::Value::Object(tenants)
    }

    /// Snapshot current metrics as a JSON-serializable value.
    pub fn snapshot(&self) -> serde_json::Value {
        let model_counts = snapshot_map(&self.model_counts);
        let provider_counts = snapshot_map(&self.provider_counts);
        let total_cost = self.total_cost_micro.load(Ordering::Relaxed) as f64 / 1_000_000.0;
        let model_costs = if let Ok(costs) = self.model_costs.lock() {
            let mut map = serde_json::Map::new();
            for (k, v) in costs.iter() {
                map.insert(k.clone(), serde_json::json!(v));
            }
            serde_json::Value::Object(map)
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        let total_reqs = self.total_requests.load(Ordering::Relaxed);
        let total_errs = self.total_errors.load(Ordering::Relaxed);
        let uptime_secs = self.created_at.elapsed().as_secs();

        // Computed fields for dashboard frontend
        let error_rate = if total_reqs > 0 {
            total_errs as f64 / total_reqs as f64
        } else {
            0.0
        };
        let avg_latency = if total_reqs > 0 {
            self.total_latency_ms.load(Ordering::Relaxed) as f64 / total_reqs as f64
        } else {
            0.0
        };
        let rpm = if uptime_secs > 0 {
            (total_reqs as f64 / uptime_secs as f64) * 60.0
        } else {
            0.0
        };
        let total_tokens = self.total_input_tokens.load(Ordering::Relaxed)
            + self.total_output_tokens.load(Ordering::Relaxed);
        let active_providers = if let Ok(m) = self.provider_counts.read() {
            m.len() as u64
        } else {
            0
        };

        serde_json::json!({
            "total_requests": total_reqs,
            "total_errors": total_errs,
            "total_input_tokens": self.total_input_tokens.load(Ordering::Relaxed),
            "total_output_tokens": self.total_output_tokens.load(Ordering::Relaxed),
            "total_cost_usd": total_cost,
            "latency_ms": {
                "<100": self.latency_buckets[0].load(Ordering::Relaxed),
                "100-499": self.latency_buckets[1].load(Ordering::Relaxed),
                "500-999": self.latency_buckets[2].load(Ordering::Relaxed),
                "1000-4999": self.latency_buckets[3].load(Ordering::Relaxed),
                "5000-29999": self.latency_buckets[4].load(Ordering::Relaxed),
                ">=30000": self.latency_buckets[5].load(Ordering::Relaxed),
            },
            "ttft_ms": {
                "<50": self.ttft_buckets[0].load(Ordering::Relaxed),
                "50-99": self.ttft_buckets[1].load(Ordering::Relaxed),
                "100-499": self.ttft_buckets[2].load(Ordering::Relaxed),
                "500-999": self.ttft_buckets[3].load(Ordering::Relaxed),
                "1000-4999": self.ttft_buckets[4].load(Ordering::Relaxed),
                ">=5000": self.ttft_buckets[5].load(Ordering::Relaxed),
            },
            "cache": {
                "hits": self.cache_hits.load(Ordering::Relaxed),
                "misses": self.cache_misses.load(Ordering::Relaxed),
            },
            "by_model": model_counts,
            "by_provider": provider_counts,
            "cost_by_model": model_costs,
            "by_tenant": self.tenant_snapshot(),
            // Computed fields for dashboard frontend
            "total_tokens": total_tokens,
            "active_providers": active_providers,
            "requests_per_minute": rpm,
            "avg_latency_ms": avg_latency,
            "error_rate": error_rate,
            "uptime_seconds": uptime_secs,
        })
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

fn increment_map(map: &RwLock<HashMap<String, AtomicU64>>, key: &str) {
    // Fast path: read lock
    if let Ok(m) = map.read()
        && let Some(counter) = m.get(key)
    {
        counter.fetch_add(1, Ordering::Relaxed);
        return;
    }
    // Slow path: write lock to insert
    if let Ok(mut m) = map.write() {
        m.entry(key.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }
}

fn increment_map_by(map: &RwLock<HashMap<String, AtomicU64>>, key: &str, amount: u64) {
    if amount == 0 {
        return;
    }
    // Fast path: read lock
    if let Ok(m) = map.read()
        && let Some(counter) = m.get(key)
    {
        counter.fetch_add(amount, Ordering::Relaxed);
        return;
    }
    // Slow path: write lock to insert
    if let Ok(mut m) = map.write() {
        m.entry(key.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(amount, Ordering::Relaxed);
    }
}

fn snapshot_map(map: &RwLock<HashMap<String, AtomicU64>>) -> serde_json::Value {
    let mut result = serde_json::Map::new();
    if let Ok(m) = map.read() {
        for (k, v) in m.iter() {
            result.insert(
                k.clone(),
                serde_json::Value::Number(v.load(Ordering::Relaxed).into()),
            );
        }
    }
    serde_json::Value::Object(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_metrics() {
        let m = Metrics::new();
        m.record_request("gpt-4", "openai");
        m.record_request("gpt-4", "openai");
        m.record_request("claude-3", "claude");
        m.record_error();
        m.record_latency_ms(50);
        m.record_latency_ms(250);
        m.record_latency_ms(5000);

        let snap = m.snapshot();
        assert_eq!(snap["total_requests"], 3);
        assert_eq!(snap["total_errors"], 1);
        assert_eq!(snap["by_model"]["gpt-4"], 2);
        assert_eq!(snap["by_model"]["claude-3"], 1);
        assert_eq!(snap["by_provider"]["openai"], 2);
        assert_eq!(snap["latency_ms"]["<100"], 1);
        assert_eq!(snap["latency_ms"]["100-499"], 1);
        assert_eq!(snap["latency_ms"]["5000-29999"], 1);
    }

    #[test]
    fn test_ttft_recording() {
        let m = Metrics::new();
        m.record_ttft_ms(30);
        m.record_ttft_ms(80);
        m.record_ttft_ms(200);
        m.record_ttft_ms(800);
        m.record_ttft_ms(2000);
        m.record_ttft_ms(10000);

        let snap = m.snapshot();
        assert_eq!(snap["ttft_ms"]["<50"], 1);
        assert_eq!(snap["ttft_ms"]["50-99"], 1);
        assert_eq!(snap["ttft_ms"]["100-499"], 1);
        assert_eq!(snap["ttft_ms"]["500-999"], 1);
        assert_eq!(snap["ttft_ms"]["1000-4999"], 1);
        assert_eq!(snap["ttft_ms"][">=5000"], 1);
    }

    #[test]
    fn test_tenant_metrics() {
        let m = Metrics::new();
        m.record_tenant_request("alpha");
        m.record_tenant_request("alpha");
        m.record_tenant_request("beta");
        m.record_tenant_tokens("alpha", 1000);
        m.record_tenant_cost("alpha", 0.05);

        let snap = m.snapshot();
        assert_eq!(snap["by_tenant"]["alpha"]["requests"], 2);
        assert_eq!(snap["by_tenant"]["alpha"]["tokens"], 1000);
        assert_eq!(snap["by_tenant"]["beta"]["requests"], 1);
    }

    #[test]
    fn test_cache_counters() {
        let m = Metrics::new();
        m.record_cache_hit();
        m.record_cache_hit();
        m.record_cache_miss();

        let snap = m.snapshot();
        assert_eq!(snap["cache"]["hits"], 2);
        assert_eq!(snap["cache"]["misses"], 1);
    }
}
