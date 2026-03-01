use std::collections::HashMap;
use std::sync::{Mutex, RwLock};
use std::time::Instant;

use crate::config::RateLimitConfig;

/// Sliding window rate limiter using in-memory timestamp tracking.
pub struct RateLimiter {
    /// Global request timestamps (sliding window).
    global: Mutex<SlidingWindow>,
    /// Per-key request timestamps (sliding window per key).
    per_key: RwLock<HashMap<String, Mutex<SlidingWindow>>>,
    /// Current configuration.
    config: RwLock<RateLimitConfig>,
}

struct SlidingWindow {
    timestamps: Vec<Instant>,
}

impl SlidingWindow {
    fn new() -> Self {
        Self {
            timestamps: Vec::new(),
        }
    }

    /// Remove timestamps older than 60 seconds and return current count.
    fn count_and_prune(&mut self, now: Instant) -> u32 {
        let cutoff = now - std::time::Duration::from_secs(60);
        self.timestamps.retain(|&t| t > cutoff);
        self.timestamps.len() as u32
    }

    /// Record a new request timestamp.
    fn record(&mut self, now: Instant) {
        self.timestamps.push(now);
    }
}

/// Result of a rate limit check.
pub struct RateLimitInfo {
    /// Whether the request is allowed.
    pub allowed: bool,
    /// Requests remaining in the current window.
    pub remaining: u32,
    /// The rate limit for this window.
    pub limit: u32,
    /// Seconds until the window resets (approximate).
    pub reset_secs: u64,
}

impl RateLimiter {
    pub fn new(config: &RateLimitConfig) -> Self {
        Self {
            global: Mutex::new(SlidingWindow::new()),
            per_key: RwLock::new(HashMap::new()),
            config: RwLock::new(config.clone()),
        }
    }

    /// Update configuration (called on hot-reload).
    pub fn update_config(&self, config: &RateLimitConfig) {
        if let Ok(mut cfg) = self.config.write() {
            *cfg = config.clone();
        }
    }

    /// Check rate limits. Returns info about the most restrictive limit.
    /// `api_key` is None for unauthenticated requests.
    pub fn check(&self, api_key: Option<&str>) -> RateLimitInfo {
        let config = self.config.read().unwrap();

        if !config.enabled {
            return RateLimitInfo {
                allowed: true,
                remaining: u32::MAX,
                limit: 0,
                reset_secs: 0,
            };
        }

        let now = Instant::now();
        let mut most_restrictive = RateLimitInfo {
            allowed: true,
            remaining: u32::MAX,
            limit: 0,
            reset_secs: 60,
        };

        // Check global RPM
        if config.global_rpm > 0 {
            let mut global = self.global.lock().unwrap();
            let count = global.count_and_prune(now);
            let remaining = config.global_rpm.saturating_sub(count);
            if count >= config.global_rpm {
                return RateLimitInfo {
                    allowed: false,
                    remaining: 0,
                    limit: config.global_rpm,
                    reset_secs: self.estimate_reset(&global, now),
                };
            }
            if remaining < most_restrictive.remaining {
                most_restrictive = RateLimitInfo {
                    allowed: true,
                    remaining,
                    limit: config.global_rpm,
                    reset_secs: 60,
                };
            }
        }

        // Check per-key RPM
        if config.per_key_rpm > 0
            && let Some(key) = api_key
        {
            let info = self.check_per_key(key, config.per_key_rpm, now);
            if !info.allowed {
                return info;
            }
            if info.remaining < most_restrictive.remaining {
                most_restrictive = info;
            }
        }

        most_restrictive
    }

    /// Record a request. Call after check() returns allowed=true.
    pub fn record(&self, api_key: Option<&str>) {
        let config = self.config.read().unwrap();
        if !config.enabled {
            return;
        }

        let now = Instant::now();

        if config.global_rpm > 0 {
            let mut global = self.global.lock().unwrap();
            global.record(now);
        }

        if config.per_key_rpm > 0
            && let Some(key) = api_key
        {
            self.record_per_key(key, now);
        }
    }

    fn check_per_key(&self, key: &str, limit: u32, now: Instant) -> RateLimitInfo {
        let per_key = self.per_key.read().unwrap();
        if let Some(window) = per_key.get(key) {
            let mut window = window.lock().unwrap();
            let count = window.count_and_prune(now);
            let remaining = limit.saturating_sub(count);
            RateLimitInfo {
                allowed: count < limit,
                remaining,
                limit,
                reset_secs: if count >= limit {
                    self.estimate_reset(&window, now)
                } else {
                    60
                },
            }
        } else {
            RateLimitInfo {
                allowed: true,
                remaining: limit,
                limit,
                reset_secs: 60,
            }
        }
    }

    fn record_per_key(&self, key: &str, now: Instant) {
        // Fast path: read lock
        {
            let per_key = self.per_key.read().unwrap();
            if let Some(window) = per_key.get(key) {
                let mut window = window.lock().unwrap();
                window.record(now);
                return;
            }
        }
        // Slow path: write lock to insert
        {
            let mut per_key = self.per_key.write().unwrap();
            let window = per_key
                .entry(key.to_string())
                .or_insert_with(|| Mutex::new(SlidingWindow::new()));
            let window = window.get_mut().unwrap();
            window.record(now);
        }
    }

    fn estimate_reset(&self, window: &SlidingWindow, now: Instant) -> u64 {
        if let Some(&oldest) = window.timestamps.first() {
            let age = now.duration_since(oldest);
            60u64.saturating_sub(age.as_secs())
        } else {
            60
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_allows_all() {
        let config = RateLimitConfig {
            enabled: false,
            global_rpm: 10,
            per_key_rpm: 5,
        };
        let limiter = RateLimiter::new(&config);
        let info = limiter.check(Some("key1"));
        assert!(info.allowed);
    }

    #[test]
    fn test_global_rpm_limit() {
        let config = RateLimitConfig {
            enabled: true,
            global_rpm: 3,
            per_key_rpm: 0,
        };
        let limiter = RateLimiter::new(&config);

        for _ in 0..3 {
            let info = limiter.check(None);
            assert!(info.allowed);
            limiter.record(None);
        }

        let info = limiter.check(None);
        assert!(!info.allowed);
        assert_eq!(info.remaining, 0);
        assert_eq!(info.limit, 3);
    }

    #[test]
    fn test_per_key_rpm_limit() {
        let config = RateLimitConfig {
            enabled: true,
            global_rpm: 0,
            per_key_rpm: 2,
        };
        let limiter = RateLimiter::new(&config);

        // key1 uses 2 requests
        for _ in 0..2 {
            let info = limiter.check(Some("key1"));
            assert!(info.allowed);
            limiter.record(Some("key1"));
        }

        // key1 is now rate limited
        let info = limiter.check(Some("key1"));
        assert!(!info.allowed);

        // key2 still has quota
        let info = limiter.check(Some("key2"));
        assert!(info.allowed);
    }

    #[test]
    fn test_remaining_count() {
        let config = RateLimitConfig {
            enabled: true,
            global_rpm: 5,
            per_key_rpm: 0,
        };
        let limiter = RateLimiter::new(&config);

        limiter.record(None);
        limiter.record(None);

        let info = limiter.check(None);
        assert!(info.allowed);
        assert_eq!(info.remaining, 3);
        assert_eq!(info.limit, 5);
    }

    #[test]
    fn test_update_config() {
        let config = RateLimitConfig {
            enabled: true,
            global_rpm: 2,
            per_key_rpm: 0,
        };
        let limiter = RateLimiter::new(&config);

        limiter.record(None);
        limiter.record(None);
        assert!(!limiter.check(None).allowed);

        // Increase limit
        limiter.update_config(&RateLimitConfig {
            enabled: true,
            global_rpm: 5,
            per_key_rpm: 0,
        });

        assert!(limiter.check(None).allowed);
    }
}
