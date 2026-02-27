use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// Lightweight in-memory metrics using atomic counters.
pub struct Metrics {
    pub total_requests: AtomicU64,
    pub total_errors: AtomicU64,
    pub total_input_tokens: AtomicU64,
    pub total_output_tokens: AtomicU64,
    /// Per-model request counts.
    model_counts: RwLock<HashMap<String, AtomicU64>>,
    /// Per-provider request counts.
    provider_counts: RwLock<HashMap<String, AtomicU64>>,
    /// Latency histogram buckets (ms): <100, <500, <1000, <5000, <30000, >=30000.
    pub latency_buckets: [AtomicU64; 6],
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_input_tokens: AtomicU64::new(0),
            total_output_tokens: AtomicU64::new(0),
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
    }

    pub fn record_tokens(&self, input: u64, output: u64) {
        self.total_input_tokens.fetch_add(input, Ordering::Relaxed);
        self.total_output_tokens
            .fetch_add(output, Ordering::Relaxed);
    }

    /// Snapshot current metrics as a JSON-serializable value.
    pub fn snapshot(&self) -> serde_json::Value {
        let model_counts = snapshot_map(&self.model_counts);
        let provider_counts = snapshot_map(&self.provider_counts);

        serde_json::json!({
            "total_requests": self.total_requests.load(Ordering::Relaxed),
            "total_errors": self.total_errors.load(Ordering::Relaxed),
            "total_input_tokens": self.total_input_tokens.load(Ordering::Relaxed),
            "total_output_tokens": self.total_output_tokens.load(Ordering::Relaxed),
            "latency_ms": {
                "<100": self.latency_buckets[0].load(Ordering::Relaxed),
                "100-499": self.latency_buckets[1].load(Ordering::Relaxed),
                "500-999": self.latency_buckets[2].load(Ordering::Relaxed),
                "1000-4999": self.latency_buckets[3].load(Ordering::Relaxed),
                "5000-29999": self.latency_buckets[4].load(Ordering::Relaxed),
                ">=30000": self.latency_buckets[5].load(Ordering::Relaxed),
            },
            "by_model": model_counts,
            "by_provider": provider_counts,
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
}
