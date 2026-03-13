use prism_core::routing::config::HealthConfig;
use prism_core::routing::planner::{CredentialHealth, HealthSnapshot};
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

// ─── Sliding window counter ───────────────────────────────────────────────

/// A simple fixed-window counter for rate tracking.
pub struct SlidingWindowCounter {
    window: Duration,
    count: AtomicU64,
    window_start: RwLock<Instant>,
}

impl SlidingWindowCounter {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            count: AtomicU64::new(0),
            window_start: RwLock::new(Instant::now()),
        }
    }

    pub fn increment(&self) {
        self.maybe_reset();
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.maybe_reset();
        self.count.load(Ordering::Relaxed)
    }

    pub fn rate_per_second(&self) -> f64 {
        let count = self.get();
        let elapsed = self
            .window_start
            .read()
            .map(|s| s.elapsed().as_secs_f64())
            .unwrap_or(1.0);
        if elapsed < 0.001 {
            return 0.0;
        }
        count as f64 / elapsed
    }

    fn maybe_reset(&self) {
        if let Ok(start) = self.window_start.read()
            && start.elapsed() >= self.window
        {
            drop(start);
            if let Ok(mut start) = self.window_start.write()
                && start.elapsed() >= self.window
            {
                *start = Instant::now();
                self.count.store(0, Ordering::Relaxed);
            }
        }
    }
}

// ─── Credential health state ──────────────────────────────────────────────

pub struct CredentialHealthState {
    // Outlier detection
    consecutive_5xx: u32,
    consecutive_local_failures: u32,
    // Ejection
    ejected: bool,
    eject_until: Option<Instant>,
    eject_count: u32,
    // Inflight
    inflight: AtomicU64,
    // EWMA
    ewma_latency_ms: f64,
    ewma_cost_micro_usd: f64,
    ewma_alpha: f64,
    // Rate tracking
    recent_429: SlidingWindowCounter,
    recent_5xx: SlidingWindowCounter,
    recent_total: SlidingWindowCounter,
    // Cooldown
    cooldown_until: Option<Instant>,
    // Circuit breaker (simple open/closed)
    circuit_open: bool,
    failure_count: u32,
    failure_threshold: u32,
    cooldown_seconds: u64,
    last_failure_time: Option<Instant>,
}

impl CredentialHealthState {
    fn new(config: &HealthConfig) -> Self {
        let window = Duration::from_secs(60);
        Self {
            consecutive_5xx: 0,
            consecutive_local_failures: 0,
            ejected: false,
            eject_until: None,
            eject_count: 0,
            inflight: AtomicU64::new(0),
            ewma_latency_ms: 0.0,
            ewma_cost_micro_usd: 0.0,
            ewma_alpha: 0.3,
            recent_429: SlidingWindowCounter::new(window),
            recent_5xx: SlidingWindowCounter::new(window),
            recent_total: SlidingWindowCounter::new(window),
            cooldown_until: None,
            circuit_open: false,
            failure_count: 0,
            failure_threshold: config.circuit_breaker.failure_threshold,
            cooldown_seconds: config.circuit_breaker.cooldown_seconds,
            last_failure_time: None,
        }
    }

    fn to_snapshot(&self) -> CredentialHealth {
        // Check ejection expiry
        let ejected = self.ejected
            && self
                .eject_until
                .map(|t| Instant::now() < t)
                .unwrap_or(false);

        // Check cooldown expiry
        let cooldown_active = self
            .cooldown_until
            .map(|t| Instant::now() < t)
            .unwrap_or(false);

        // Check circuit breaker cooldown
        let circuit_open = if self.circuit_open {
            // Check if cooldown has elapsed → allow half-open probe
            if let Some(t) = self.last_failure_time {
                t.elapsed() < Duration::from_secs(self.cooldown_seconds)
            } else {
                true
            }
        } else {
            false
        };

        CredentialHealth {
            circuit_open,
            ejected,
            inflight: self.inflight.load(Ordering::Relaxed),
            ewma_latency_ms: self.ewma_latency_ms,
            ewma_cost_micro_usd: self.ewma_cost_micro_usd,
            cooldown_active,
        }
    }
}

// ─── Attempt result ───────────────────────────────────────────────────────

pub struct AttemptResult {
    pub latency_ms: f64,
    pub cost_micro_usd: Option<u64>,
    pub status: AttemptStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptStatus {
    Success,
    RateLimit,
    ServerError,
    NetworkError,
    Timeout,
    ClientError,
}

// ─── Health manager ───────────────────────────────────────────────────────

pub struct HealthManager {
    states: RwLock<HashMap<String, CredentialHealthState>>,
    config: RwLock<HealthConfig>,
}

impl HealthManager {
    pub fn new(config: HealthConfig) -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            config: RwLock::new(config),
        }
    }

    /// Create a read-only snapshot for the planner.
    pub fn snapshot(&self) -> HealthSnapshot {
        let states = self.states.read().unwrap_or_else(|e| e.into_inner());
        let credentials = states
            .iter()
            .map(|(id, state)| (id.clone(), state.to_snapshot()))
            .collect();
        HealthSnapshot { credentials }
    }

    /// Register a credential for health tracking.
    pub fn register_credential(&self, credential_id: &str) {
        let config = self.config.read().unwrap_or_else(|e| e.into_inner());
        let mut states = self.states.write().unwrap_or_else(|e| e.into_inner());
        states
            .entry(credential_id.to_string())
            .or_insert_with(|| CredentialHealthState::new(&config));
    }

    /// Remove a credential from health tracking.
    pub fn unregister_credential(&self, credential_id: &str) {
        let mut states = self.states.write().unwrap_or_else(|e| e.into_inner());
        states.remove(credential_id);
    }

    /// Record that an attempt has started (increment inflight counter).
    pub fn record_attempt_start(&self, credential_id: &str) {
        let states = self.states.read().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = states.get(credential_id) {
            state.inflight.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record the result of an attempt.
    pub fn record_attempt_result(&self, credential_id: &str, result: &AttemptResult) {
        let mut states = self.states.write().unwrap_or_else(|e| e.into_inner());
        let Some(state) = states.get_mut(credential_id) else {
            return;
        };

        // Decrement inflight
        let prev = state.inflight.fetch_sub(1, Ordering::Relaxed);
        if prev == 0 {
            // Prevent underflow
            state.inflight.store(0, Ordering::Relaxed);
        }

        // Track total
        state.recent_total.increment();

        // Update EWMA latency
        if state.ewma_latency_ms == 0.0 {
            state.ewma_latency_ms = result.latency_ms;
        } else {
            state.ewma_latency_ms = state.ewma_alpha * result.latency_ms
                + (1.0 - state.ewma_alpha) * state.ewma_latency_ms;
        }

        // Update EWMA cost
        if let Some(cost) = result.cost_micro_usd {
            let cost_f = cost as f64;
            if state.ewma_cost_micro_usd == 0.0 {
                state.ewma_cost_micro_usd = cost_f;
            } else {
                state.ewma_cost_micro_usd = state.ewma_alpha * cost_f
                    + (1.0 - state.ewma_alpha) * state.ewma_cost_micro_usd;
            }
        }

        let config = self.config.read().unwrap_or_else(|e| e.into_inner());

        match result.status {
            AttemptStatus::Success => {
                // Reset failure counters
                state.consecutive_5xx = 0;
                state.consecutive_local_failures = 0;
                // Clear ejection
                if state.ejected {
                    state.ejected = false;
                    state.eject_until = None;
                    state.eject_count = 0;
                }
                // Close circuit breaker
                if state.circuit_open {
                    state.circuit_open = false;
                    state.failure_count = 0;
                }
            }
            AttemptStatus::ServerError => {
                state.consecutive_5xx += 1;
                state.recent_5xx.increment();
                state.failure_count += 1;
                state.last_failure_time = Some(Instant::now());
                self.check_ejection(state, &config);
                self.check_circuit_breaker(state);
            }
            AttemptStatus::NetworkError | AttemptStatus::Timeout => {
                state.consecutive_local_failures += 1;
                state.failure_count += 1;
                state.last_failure_time = Some(Instant::now());
                self.check_ejection(state, &config);
                self.check_circuit_breaker(state);
            }
            AttemptStatus::RateLimit => {
                state.recent_429.increment();
                // Set cooldown
                state.cooldown_until = Some(
                    Instant::now() + Duration::from_secs(config.circuit_breaker.cooldown_seconds),
                );
            }
            AttemptStatus::ClientError => {
                // Client errors don't affect health
            }
        }
    }

    /// Update health config (on config reload).
    pub fn update_config(&self, config: &HealthConfig) {
        let mut cfg = self.config.write().unwrap_or_else(|e| e.into_inner());
        *cfg = config.clone();
    }

    fn check_ejection(&self, state: &mut CredentialHealthState, config: &HealthConfig) {
        let od = &config.outlier_detection;
        let should_eject = state.consecutive_5xx >= od.consecutive_5xx
            || state.consecutive_local_failures >= od.consecutive_local_failures;

        if should_eject && !state.ejected {
            state.ejected = true;
            let base = Duration::from_secs(od.base_eject_seconds);
            let max = Duration::from_secs(od.max_eject_seconds);
            let duration = std::cmp::min(base * 2u32.saturating_pow(state.eject_count), max);
            state.eject_until = Some(Instant::now() + duration);
            state.eject_count = state.eject_count.saturating_add(1);
        }
    }

    fn check_circuit_breaker(&self, state: &mut CredentialHealthState) {
        if !state.circuit_open && state.failure_count >= state.failure_threshold {
            state.circuit_open = true;
        }
    }
}

// ─── Retry budget ─────────────────────────────────────────────────────────

pub struct RetryBudgetState {
    ratio: f64,
    min_retries_per_second: u32,
    recent_requests: SlidingWindowCounter,
    recent_retries: SlidingWindowCounter,
}

impl RetryBudgetState {
    pub fn new(ratio: f64, min_retries_per_second: u32) -> Self {
        let window = Duration::from_secs(10);
        Self {
            ratio,
            min_retries_per_second,
            recent_requests: SlidingWindowCounter::new(window),
            recent_retries: SlidingWindowCounter::new(window),
        }
    }

    pub fn record_request(&self) {
        self.recent_requests.increment();
    }

    pub fn allows_retry(&self) -> bool {
        let retries = self.recent_retries.get();
        let requests = self.recent_requests.get();

        // Always allow if under min-retries-per-second floor
        let retry_rate = self.recent_retries.rate_per_second();
        if retry_rate < self.min_retries_per_second as f64 {
            return true;
        }

        // Allow if under ratio
        if requests == 0 {
            return true;
        }
        (retries as f64 / requests as f64) < self.ratio
    }

    pub fn record_retry(&self) {
        self.recent_retries.increment();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_health_config() -> HealthConfig {
        HealthConfig::default()
    }

    #[test]
    fn test_sliding_window_counter_basic() {
        let counter = SlidingWindowCounter::new(Duration::from_secs(60));
        assert_eq!(counter.get(), 0);
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn test_health_manager_register_unregister() {
        let hm = HealthManager::new(default_health_config());
        hm.register_credential("cred-1");
        let snapshot = hm.snapshot();
        assert!(snapshot.credentials.contains_key("cred-1"));

        hm.unregister_credential("cred-1");
        let snapshot = hm.snapshot();
        assert!(!snapshot.credentials.contains_key("cred-1"));
    }

    #[test]
    fn test_health_manager_snapshot_default() {
        let hm = HealthManager::new(default_health_config());
        hm.register_credential("cred-1");
        let snapshot = hm.snapshot();
        let ch = &snapshot.credentials["cred-1"];
        assert!(!ch.circuit_open);
        assert!(!ch.ejected);
        assert_eq!(ch.inflight, 0);
        assert_eq!(ch.ewma_latency_ms, 0.0);
    }

    #[test]
    fn test_health_manager_inflight_tracking() {
        let hm = HealthManager::new(default_health_config());
        hm.register_credential("cred-1");

        hm.record_attempt_start("cred-1");
        hm.record_attempt_start("cred-1");
        let snapshot = hm.snapshot();
        assert_eq!(snapshot.credentials["cred-1"].inflight, 2);

        hm.record_attempt_result(
            "cred-1",
            &AttemptResult {
                latency_ms: 100.0,
                cost_micro_usd: None,
                status: AttemptStatus::Success,
            },
        );
        let snapshot = hm.snapshot();
        assert_eq!(snapshot.credentials["cred-1"].inflight, 1);
    }

    #[test]
    fn test_health_manager_ewma_latency() {
        let hm = HealthManager::new(default_health_config());
        hm.register_credential("cred-1");

        // First result sets initial value
        hm.record_attempt_result(
            "cred-1",
            &AttemptResult {
                latency_ms: 100.0,
                cost_micro_usd: None,
                status: AttemptStatus::Success,
            },
        );
        let snapshot = hm.snapshot();
        assert!((snapshot.credentials["cred-1"].ewma_latency_ms - 100.0).abs() < 0.001);

        // Second result applies EWMA (alpha=0.3)
        hm.record_attempt_result(
            "cred-1",
            &AttemptResult {
                latency_ms: 200.0,
                cost_micro_usd: None,
                status: AttemptStatus::Success,
            },
        );
        let snapshot = hm.snapshot();
        // 0.3 * 200 + 0.7 * 100 = 130
        assert!((snapshot.credentials["cred-1"].ewma_latency_ms - 130.0).abs() < 0.001);
    }

    #[test]
    fn test_health_manager_circuit_breaker() {
        let mut config = default_health_config();
        config.circuit_breaker.failure_threshold = 3;
        config.circuit_breaker.cooldown_seconds = 60;
        let hm = HealthManager::new(config);
        hm.register_credential("cred-1");

        // Record failures
        for _ in 0..3 {
            hm.record_attempt_result(
                "cred-1",
                &AttemptResult {
                    latency_ms: 100.0,
                    cost_micro_usd: None,
                    status: AttemptStatus::ServerError,
                },
            );
        }

        let snapshot = hm.snapshot();
        assert!(snapshot.credentials["cred-1"].circuit_open);

        // Success closes circuit
        hm.record_attempt_result(
            "cred-1",
            &AttemptResult {
                latency_ms: 100.0,
                cost_micro_usd: None,
                status: AttemptStatus::Success,
            },
        );
        let snapshot = hm.snapshot();
        assert!(!snapshot.credentials["cred-1"].circuit_open);
    }

    #[test]
    fn test_health_manager_outlier_ejection() {
        let mut config = default_health_config();
        config.outlier_detection.consecutive_5xx = 3;
        config.outlier_detection.base_eject_seconds = 1;
        config.outlier_detection.max_eject_seconds = 60;
        let hm = HealthManager::new(config);
        hm.register_credential("cred-1");

        // Record 3 consecutive 5xx
        for _ in 0..3 {
            hm.record_attempt_result(
                "cred-1",
                &AttemptResult {
                    latency_ms: 100.0,
                    cost_micro_usd: None,
                    status: AttemptStatus::ServerError,
                },
            );
        }

        let snapshot = hm.snapshot();
        assert!(snapshot.credentials["cred-1"].ejected);
    }

    #[test]
    fn test_health_manager_success_clears_ejection() {
        let mut config = default_health_config();
        config.outlier_detection.consecutive_5xx = 1;
        config.outlier_detection.base_eject_seconds = 3600; // long eject
        let hm = HealthManager::new(config);
        hm.register_credential("cred-1");

        // Trigger ejection
        hm.record_attempt_result(
            "cred-1",
            &AttemptResult {
                latency_ms: 100.0,
                cost_micro_usd: None,
                status: AttemptStatus::ServerError,
            },
        );
        assert!(hm.snapshot().credentials["cred-1"].ejected);

        // Success clears ejection
        hm.record_attempt_result(
            "cred-1",
            &AttemptResult {
                latency_ms: 100.0,
                cost_micro_usd: None,
                status: AttemptStatus::Success,
            },
        );
        assert!(!hm.snapshot().credentials["cred-1"].ejected);
    }

    #[test]
    fn test_health_manager_ewma_cost() {
        let hm = HealthManager::new(default_health_config());
        hm.register_credential("cred-1");

        hm.record_attempt_result(
            "cred-1",
            &AttemptResult {
                latency_ms: 100.0,
                cost_micro_usd: Some(1000),
                status: AttemptStatus::Success,
            },
        );
        let snapshot = hm.snapshot();
        assert!((snapshot.credentials["cred-1"].ewma_cost_micro_usd - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_health_manager_rate_limit_cooldown() {
        let mut config = default_health_config();
        config.circuit_breaker.cooldown_seconds = 5;
        let hm = HealthManager::new(config);
        hm.register_credential("cred-1");

        hm.record_attempt_result(
            "cred-1",
            &AttemptResult {
                latency_ms: 100.0,
                cost_micro_usd: None,
                status: AttemptStatus::RateLimit,
            },
        );
        let snapshot = hm.snapshot();
        assert!(snapshot.credentials["cred-1"].cooldown_active);
    }

    #[test]
    fn test_retry_budget_allows_when_under_ratio() {
        let budget = RetryBudgetState::new(0.2, 0);
        for _ in 0..10 {
            budget.record_request();
        }
        budget.record_retry();
        assert!(budget.allows_retry()); // 1/10 = 0.1 < 0.2
    }

    #[test]
    fn test_retry_budget_min_floor() {
        let budget = RetryBudgetState::new(0.0, 100); // ratio=0 but high min floor
        budget.record_request();
        assert!(budget.allows_retry()); // under min floor
    }

    #[test]
    fn test_retry_budget_no_requests() {
        let budget = RetryBudgetState::new(0.2, 0);
        assert!(budget.allows_retry()); // no requests = allow
    }

    #[test]
    fn test_concurrent_health_manager() {
        use std::sync::Arc;
        use std::thread;

        let hm = Arc::new(HealthManager::new(default_health_config()));
        hm.register_credential("cred-1");

        let mut handles = vec![];
        for i in 0..10 {
            let hm = hm.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    hm.record_attempt_start("cred-1");
                    let status = if i % 2 == 0 {
                        AttemptStatus::Success
                    } else {
                        AttemptStatus::ServerError
                    };
                    hm.record_attempt_result(
                        "cred-1",
                        &AttemptResult {
                            latency_ms: 100.0,
                            cost_micro_usd: Some(500),
                            status,
                        },
                    );
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Should not panic, snapshot should be valid
        let snapshot = hm.snapshot();
        assert!(snapshot.credentials.contains_key("cred-1"));
    }
}
