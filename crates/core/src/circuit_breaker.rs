use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;

/// Trait: pluggable circuit breaker policy.
pub trait CircuitBreakerPolicy: Send + Sync {
    fn can_execute(&self) -> bool;
    fn record_success(&self);
    fn record_failure(&self);
    fn state(&self) -> CircuitState;
    fn reset(&self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitStateInner {
    state: CircuitState,
    failure_count: u32,
    last_failure: Option<Instant>,
    opened_at: Option<Instant>,
    half_open_probes: u32,
}

/// Default implementation: three-state circuit breaker (Closed → Open → HalfOpen → Closed).
pub struct ThreeStateCircuitBreaker {
    inner: Mutex<CircuitStateInner>,
    config: CircuitBreakerConfig,
}

impl ThreeStateCircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            inner: Mutex::new(CircuitStateInner {
                state: CircuitState::Closed,
                failure_count: 0,
                last_failure: None,
                opened_at: None,
                half_open_probes: 0,
            }),
            config,
        }
    }
}

impl CircuitBreakerPolicy for ThreeStateCircuitBreaker {
    fn can_execute(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        match inner.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(opened_at) = inner.opened_at {
                    if opened_at.elapsed().as_secs() >= self.config.cooldown_secs {
                        inner.state = CircuitState::HalfOpen;
                        inner.half_open_probes = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => inner.half_open_probes < self.config.half_open_max_probes,
        }
    }

    fn record_success(&self) {
        let mut inner = self.inner.lock().unwrap();
        match inner.state {
            CircuitState::HalfOpen => {
                // Recovery confirmed — close the circuit
                inner.state = CircuitState::Closed;
                inner.failure_count = 0;
                inner.last_failure = None;
                inner.opened_at = None;
                inner.half_open_probes = 0;
            }
            CircuitState::Closed => {
                inner.failure_count = inner.failure_count.saturating_sub(1);
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&self) {
        let mut inner = self.inner.lock().unwrap();
        let now = Instant::now();
        match inner.state {
            CircuitState::Closed => {
                // Expire old failures outside the rolling window
                if let Some(last) = inner.last_failure
                    && last.elapsed().as_secs() > self.config.rolling_window_secs
                {
                    inner.failure_count = 0;
                }
                inner.failure_count += 1;
                inner.last_failure = Some(now);
                if inner.failure_count >= self.config.failure_threshold {
                    inner.state = CircuitState::Open;
                    inner.opened_at = Some(now);
                }
            }
            CircuitState::HalfOpen => {
                // Probe failed — re-open the circuit
                inner.state = CircuitState::Open;
                inner.opened_at = Some(now);
                inner.half_open_probes = 0;
            }
            CircuitState::Open => {}
        }
    }

    fn state(&self) -> CircuitState {
        self.inner.lock().unwrap().state
    }

    fn reset(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.state = CircuitState::Closed;
        inner.failure_count = 0;
        inner.last_failure = None;
        inner.opened_at = None;
        inner.half_open_probes = 0;
    }
}

/// Noop implementation — always allows execution. Used when circuit breaking is disabled.
pub struct NoopCircuitBreaker;

impl CircuitBreakerPolicy for NoopCircuitBreaker {
    fn can_execute(&self) -> bool {
        true
    }
    fn record_success(&self) {}
    fn record_failure(&self) {}
    fn state(&self) -> CircuitState {
        CircuitState::Closed
    }
    fn reset(&self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub cooldown_secs: u64,
    pub half_open_max_probes: u32,
    pub rolling_window_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 5,
            cooldown_secs: 30,
            half_open_max_probes: 1,
            rolling_window_secs: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closed_allows_execution() {
        let cb = ThreeStateCircuitBreaker::new(CircuitBreakerConfig::default());
        assert!(cb.can_execute());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_opens_after_threshold_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = ThreeStateCircuitBreaker::new(config);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.can_execute());
    }

    #[test]
    fn test_success_decrements_failure_count() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = ThreeStateCircuitBreaker::new(config);

        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        // failure_count should be 1 now, one more failure shouldn't open
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_noop_always_allows() {
        let cb = NoopCircuitBreaker;
        assert!(cb.can_execute());
        cb.record_failure();
        assert!(cb.can_execute());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_reset_returns_to_closed() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            ..Default::default()
        };
        let cb = ThreeStateCircuitBreaker::new(config);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_execute());
    }

    #[test]
    fn test_half_open_success_closes() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            cooldown_secs: 0, // Instant cooldown for testing
            half_open_max_probes: 1,
            ..Default::default()
        };
        let cb = ThreeStateCircuitBreaker::new(config);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Cooldown elapsed (0 seconds), should transition to HalfOpen
        assert!(cb.can_execute());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_failure_reopens() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            cooldown_secs: 0,
            half_open_max_probes: 1,
            ..Default::default()
        };
        let cb = ThreeStateCircuitBreaker::new(config);

        cb.record_failure();
        assert!(cb.can_execute()); // transitions to half-open
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }
}
