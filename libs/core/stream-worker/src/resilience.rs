//! Resilience patterns for stream workers.
//!
//! This module provides:
//! - **Circuit Breaker**: Prevents cascading failures by stopping requests when error rate is high
//! - **Rate Limiter**: Controls the rate of outbound requests to external services
//!
//! ## Circuit Breaker States
//!
//! ```text
//! ┌─────────┐  failures >= threshold  ┌────────┐
//! │ CLOSED  │ ──────────────────────> │  OPEN  │
//! └─────────┘                         └────────┘
//!      ^                                   │
//!      │                                   │ timeout elapsed
//!      │                                   v
//!      │      success            ┌─────────────┐
//!      └──────────────────────── │ HALF-OPEN   │
//!                                └─────────────┘
//!                                      │
//!                       failure        │
//!                       ───────────────┘
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use stream_worker::resilience::{CircuitBreaker, CircuitBreakerConfig};
//!
//! let breaker = CircuitBreaker::new(CircuitBreakerConfig {
//!     failure_threshold: 5,
//!     success_threshold: 2,
//!     timeout_secs: 30,
//! });
//!
//! // Check if we can proceed
//! if breaker.can_execute() {
//!     match do_work().await {
//!         Ok(_) => breaker.record_success(),
//!         Err(_) => breaker.record_failure(),
//!     }
//! } else {
//!     // Circuit is open, fail fast
//! }
//! ```

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally.
    Closed,
    /// Circuit is open, requests are rejected immediately.
    Open,
    /// Circuit is half-open, testing if the service has recovered.
    HalfOpen,
}

/// Configuration for the circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit.
    pub failure_threshold: u32,
    /// Number of consecutive successes in half-open state before closing.
    pub success_threshold: u32,
    /// How long to wait in open state before transitioning to half-open.
    pub timeout_secs: u64,
    /// Percentage of errors (0-100) that triggers circuit open (alternative to consecutive failures).
    /// If set, uses sliding window error rate instead of consecutive failures.
    pub error_rate_threshold: Option<u32>,
    /// Size of the sliding window for error rate calculation.
    pub window_size: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout_secs: 30,
            error_rate_threshold: None,
            window_size: 100,
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a new config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the failure threshold.
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the success threshold for half-open state.
    pub fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set the timeout in seconds.
    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Set error rate threshold (0-100 percentage).
    pub fn with_error_rate_threshold(mut self, rate: u32) -> Self {
        self.error_rate_threshold = Some(rate.min(100));
        self
    }
}

/// Thread-safe circuit breaker implementation.
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: RwLock<CircuitState>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: RwLock<Option<Instant>>,
    // Sliding window counters
    window_total: AtomicU32,
    window_failures: AtomicU32,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration.
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: RwLock::new(None),
            window_total: AtomicU32::new(0),
            window_failures: AtomicU32::new(0),
        }
    }

    /// Create a circuit breaker with default configuration.
    pub fn default_config() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Get the current state of the circuit breaker.
    pub fn state(&self) -> CircuitState {
        *self.state.read().unwrap()
    }

    /// Check if a request can be executed.
    ///
    /// Returns `true` if the circuit is closed or half-open.
    /// Returns `false` if the circuit is open (fail fast).
    pub fn can_execute(&self) -> bool {
        let current_state = *self.state.read().unwrap();

        match current_state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                if self.should_attempt_reset() {
                    self.transition_to_half_open();
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Record a successful operation.
    pub fn record_success(&self) {
        self.update_sliding_window(false);

        let current_state = *self.state.read().unwrap();

        match current_state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if successes >= self.config.success_threshold {
                    self.transition_to_closed();
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset anyway
                self.transition_to_closed();
            }
        }
    }

    /// Record a failed operation.
    pub fn record_failure(&self) {
        self.update_sliding_window(true);

        // Update last failure time
        *self.last_failure_time.write().unwrap() = Some(Instant::now());

        let current_state = *self.state.read().unwrap();

        match current_state {
            CircuitState::Closed => {
                // Check if we should open based on error rate
                if let Some(threshold) = self.config.error_rate_threshold {
                    if self.error_rate() >= threshold {
                        self.transition_to_open();
                        return;
                    }
                }

                // Check consecutive failures
                let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if failures >= self.config.failure_threshold {
                    self.transition_to_open();
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state reopens the circuit
                self.transition_to_open();
            }
            CircuitState::Open => {
                // Already open, nothing to do
            }
        }
    }

    /// Get the current error rate (0-100).
    pub fn error_rate(&self) -> u32 {
        let total = self.window_total.load(Ordering::SeqCst);
        if total == 0 {
            return 0;
        }
        let failures = self.window_failures.load(Ordering::SeqCst);
        ((failures as f64 / total as f64) * 100.0) as u32
    }

    /// Get the failure count.
    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::SeqCst)
    }

    /// Reset the circuit breaker to closed state.
    pub fn reset(&self) {
        self.transition_to_closed();
        self.window_total.store(0, Ordering::SeqCst);
        self.window_failures.store(0, Ordering::SeqCst);
    }

    // Internal methods

    fn should_attempt_reset(&self) -> bool {
        let last_failure = self.last_failure_time.read().unwrap();
        match *last_failure {
            Some(time) => time.elapsed() >= Duration::from_secs(self.config.timeout_secs),
            None => true,
        }
    }

    fn transition_to_open(&self) {
        let mut state = self.state.write().unwrap();
        *state = CircuitState::Open;
        self.success_count.store(0, Ordering::SeqCst);
        tracing::warn!("Circuit breaker OPENED");
    }

    fn transition_to_half_open(&self) {
        let mut state = self.state.write().unwrap();
        *state = CircuitState::HalfOpen;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        tracing::info!("Circuit breaker HALF-OPEN (testing recovery)");
    }

    fn transition_to_closed(&self) {
        let mut state = self.state.write().unwrap();
        *state = CircuitState::Closed;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        tracing::info!("Circuit breaker CLOSED (recovered)");
    }

    fn update_sliding_window(&self, is_failure: bool) {
        let total = self.window_total.fetch_add(1, Ordering::SeqCst) + 1;

        if is_failure {
            self.window_failures.fetch_add(1, Ordering::SeqCst);
        }

        // Reset window if it exceeds the configured size
        if total >= self.config.window_size {
            self.window_total.store(0, Ordering::SeqCst);
            self.window_failures.store(0, Ordering::SeqCst);
        }
    }
}

/// Simple token bucket rate limiter.
///
/// Controls the rate of operations by limiting to a maximum number
/// of operations per time window.
pub struct RateLimiter {
    /// Maximum tokens (operations) allowed per window.
    max_tokens: u32,
    /// Current available tokens.
    tokens: AtomicU32,
    /// Window duration in milliseconds.
    window_ms: u64,
    /// Last refill time.
    last_refill: RwLock<Instant>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// # Arguments
    ///
    /// * `max_per_second` - Maximum operations per second
    pub fn new(max_per_second: u32) -> Self {
        Self {
            max_tokens: max_per_second,
            tokens: AtomicU32::new(max_per_second),
            window_ms: 1000,
            last_refill: RwLock::new(Instant::now()),
        }
    }

    /// Create a rate limiter with custom window.
    ///
    /// # Arguments
    ///
    /// * `max_tokens` - Maximum operations per window
    /// * `window_ms` - Window duration in milliseconds
    pub fn with_window(max_tokens: u32, window_ms: u64) -> Self {
        Self {
            max_tokens,
            tokens: AtomicU32::new(max_tokens),
            window_ms,
            last_refill: RwLock::new(Instant::now()),
        }
    }

    /// Try to acquire a token.
    ///
    /// Returns `true` if a token was acquired, `false` if rate limited.
    pub fn try_acquire(&self) -> bool {
        self.refill_if_needed();

        loop {
            let current = self.tokens.load(Ordering::SeqCst);
            if current == 0 {
                return false;
            }

            if self
                .tokens
                .compare_exchange(current, current - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
        }
    }

    /// Get the number of available tokens.
    pub fn available_tokens(&self) -> u32 {
        self.refill_if_needed();
        self.tokens.load(Ordering::SeqCst)
    }

    /// Check if rate limited (without consuming a token).
    pub fn is_rate_limited(&self) -> bool {
        self.refill_if_needed();
        self.tokens.load(Ordering::SeqCst) == 0
    }

    fn refill_if_needed(&self) {
        let mut last_refill = self.last_refill.write().unwrap();
        let elapsed = last_refill.elapsed().as_millis() as u64;

        if elapsed >= self.window_ms {
            // Refill tokens
            self.tokens.store(self.max_tokens, Ordering::SeqCst);
            *last_refill = Instant::now();
        }
    }
}

/// Combined resilience wrapper for processors.
///
/// Provides both circuit breaker and rate limiting in one convenient struct.
pub struct ResilienceLayer {
    /// Circuit breaker for failure protection.
    pub circuit_breaker: CircuitBreaker,
    /// Optional rate limiter.
    pub rate_limiter: Option<RateLimiter>,
}

impl ResilienceLayer {
    /// Create a new resilience layer with circuit breaker only.
    pub fn new(breaker_config: CircuitBreakerConfig) -> Self {
        Self {
            circuit_breaker: CircuitBreaker::new(breaker_config),
            rate_limiter: None,
        }
    }

    /// Create a resilience layer with both circuit breaker and rate limiter.
    pub fn with_rate_limit(breaker_config: CircuitBreakerConfig, max_per_second: u32) -> Self {
        Self {
            circuit_breaker: CircuitBreaker::new(breaker_config),
            rate_limiter: Some(RateLimiter::new(max_per_second)),
        }
    }

    /// Check if an operation can proceed.
    ///
    /// Returns `Ok(())` if allowed, `Err` with reason if blocked.
    pub fn check(&self) -> Result<(), ResilienceError> {
        // Check rate limiter first (cheaper check)
        if let Some(ref limiter) = self.rate_limiter {
            if !limiter.try_acquire() {
                return Err(ResilienceError::RateLimited);
            }
        }

        // Check circuit breaker
        if !self.circuit_breaker.can_execute() {
            return Err(ResilienceError::CircuitOpen);
        }

        Ok(())
    }

    /// Record a successful operation.
    pub fn record_success(&self) {
        self.circuit_breaker.record_success();
    }

    /// Record a failed operation.
    pub fn record_failure(&self) {
        self.circuit_breaker.record_failure();
    }

    /// Get circuit breaker state.
    pub fn circuit_state(&self) -> CircuitState {
        self.circuit_breaker.state()
    }
}

/// Error returned when resilience checks fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResilienceError {
    /// Circuit breaker is open.
    CircuitOpen,
    /// Rate limit exceeded.
    RateLimited,
}

impl std::fmt::Display for ResilienceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CircuitOpen => write!(f, "circuit breaker is open"),
            Self::RateLimited => write!(f, "rate limit exceeded"),
        }
    }
}

impl std::error::Error for ResilienceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_initial_state() {
        let breaker = CircuitBreaker::default_config();
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(breaker.can_execute());
    }

    #[test]
    fn test_circuit_breaker_opens_on_failures() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        });

        assert_eq!(breaker.state(), CircuitState::Closed);

        // Record failures
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.can_execute());
    }

    #[test]
    fn test_circuit_breaker_success_resets_failures() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        });

        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.failure_count(), 2);

        breaker.record_success();
        assert_eq!(breaker.failure_count(), 0);
    }

    #[test]
    fn test_circuit_breaker_half_open_closes_on_success() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout_secs: 0, // Immediate timeout for testing
            ..Default::default()
        });

        // Open the circuit
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);

        // Trigger half-open by attempting execute
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(breaker.can_execute());
        assert_eq!(breaker.state(), CircuitState::HalfOpen);

        // Successes close the circuit
        breaker.record_success();
        assert_eq!(breaker.state(), CircuitState::HalfOpen);

        breaker.record_success();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_reopens_on_failure() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            timeout_secs: 0,
            ..Default::default()
        });

        // Open the circuit
        breaker.record_failure();

        // Trigger half-open
        std::thread::sleep(std::time::Duration::from_millis(10));
        breaker.can_execute();
        assert_eq!(breaker.state(), CircuitState::HalfOpen);

        // Failure reopens
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[test]
    fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(3);

        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire()); // Rate limited
    }

    #[test]
    fn test_rate_limiter_refills() {
        let limiter = RateLimiter::with_window(2, 50); // 2 per 50ms

        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire());

        // Wait for refill
        std::thread::sleep(std::time::Duration::from_millis(60));

        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
    }

    #[test]
    fn test_resilience_layer() {
        let layer = ResilienceLayer::with_rate_limit(
            CircuitBreakerConfig {
                failure_threshold: 2,
                ..Default::default()
            },
            10,
        );

        // Should allow execution
        assert!(layer.check().is_ok());
        layer.record_success();

        // Open circuit breaker
        layer.record_failure();
        layer.record_failure();

        // Should be blocked by circuit breaker
        assert_eq!(layer.check(), Err(ResilienceError::CircuitOpen));
    }

    #[test]
    fn test_error_rate_threshold() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 100, // High threshold, won't trigger
            error_rate_threshold: Some(50), // 50% error rate triggers
            window_size: 10,
            ..Default::default()
        });

        // 3 successes, 2 failures = 40% error rate
        breaker.record_success();
        breaker.record_success();
        breaker.record_success();
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Closed);

        // Add more failures to exceed 50%
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);
    }
}
