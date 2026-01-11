//! Resilience patterns: Circuit Breaker and Rate Limiter
//!
//! These patterns help protect downstream services from overload.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests pass through
    Closed,
    /// Failures detected - requests blocked
    Open,
    /// Testing if service recovered - limited requests
    HalfOpen,
}

/// Circuit breaker for protecting downstream services
///
/// Opens when failure threshold is reached, preventing further calls.
/// After a timeout, allows limited requests to test recovery.
pub struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: RwLock<Option<Instant>>,

    /// Number of failures before opening
    failure_threshold: u32,

    /// Time to wait before testing recovery
    recovery_timeout: Duration,

    /// Number of successes in half-open to close
    success_threshold: u32,
}

impl CircuitBreaker {
    /// Create a new CircuitBreaker
    pub fn new(failure_threshold: u32, recovery_timeout: Duration, success_threshold: u32) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: RwLock::new(None),
            failure_threshold,
            recovery_timeout,
            success_threshold,
        }
    }

    /// Create with default settings (5 failures, 30s recovery, 3 successes)
    pub fn default_settings() -> Self {
        Self::new(5, Duration::from_secs(30), 3)
    }

    /// Get the current state
    pub async fn state(&self) -> CircuitState {
        let mut state = *self.state.read().await;

        // Check if we should transition from Open to HalfOpen
        if state == CircuitState::Open
            && let Some(last_failure) = *self.last_failure_time.read().await
            && last_failure.elapsed() >= self.recovery_timeout
        {
            state = CircuitState::HalfOpen;
            *self.state.write().await = state;
            self.success_count.store(0, Ordering::SeqCst);
            info!("Circuit breaker transitioned to HalfOpen");
        }

        state
    }

    /// Check if a request should be allowed
    pub async fn allow_request(&self) -> bool {
        match self.state().await {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => true, // Allow limited requests
        }
    }

    /// Record a successful request
    pub async fn record_success(&self) {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if success_count >= self.success_threshold {
                    *self.state.write().await = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    info!("Circuit breaker closed (service recovered)");
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self) {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if failure_count >= self.failure_threshold {
                    *self.state.write().await = CircuitState::Open;
                    *self.last_failure_time.write().await = Some(Instant::now());
                    warn!(
                        failures = failure_count,
                        threshold = self.failure_threshold,
                        "Circuit breaker opened"
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open goes back to open
                *self.state.write().await = CircuitState::Open;
                *self.last_failure_time.write().await = Some(Instant::now());
                self.success_count.store(0, Ordering::SeqCst);
                warn!("Circuit breaker re-opened (test request failed)");
            }
            CircuitState::Open => {}
        }
    }

    /// Reset the circuit breaker to closed state
    pub async fn reset(&self) {
        *self.state.write().await = CircuitState::Closed;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        *self.last_failure_time.write().await = None;
        info!("Circuit breaker reset");
    }
}

/// Token bucket rate limiter
///
/// Limits requests to a specified rate (requests per second).
pub struct RateLimiter {
    /// Maximum tokens (burst capacity)
    capacity: u32,

    /// Tokens added per second
    refill_rate: f64,

    /// Current token count (multiplied by 1000 for precision)
    tokens: AtomicU64,

    /// Last refill time
    last_refill: RwLock<Instant>,
}

impl RateLimiter {
    /// Create a new RateLimiter
    ///
    /// - `capacity`: Maximum burst size
    /// - `rate`: Requests per second
    pub fn new(capacity: u32, rate: f64) -> Self {
        Self {
            capacity,
            refill_rate: rate,
            tokens: AtomicU64::new((capacity as u64) * 1000),
            last_refill: RwLock::new(Instant::now()),
        }
    }

    /// Create with default settings (100 capacity, 100 rps)
    pub fn default_settings() -> Self {
        Self::new(100, 100.0)
    }

    /// Try to acquire a token
    ///
    /// Returns `true` if a token was acquired, `false` if rate limited.
    pub async fn try_acquire(&self) -> bool {
        self.refill().await;

        let tokens = self.tokens.load(Ordering::SeqCst);
        if tokens >= 1000 {
            self.tokens.fetch_sub(1000, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    /// Get current token count
    pub fn available_tokens(&self) -> f64 {
        self.tokens.load(Ordering::SeqCst) as f64 / 1000.0
    }

    /// Refill tokens based on elapsed time
    async fn refill(&self) {
        let now = Instant::now();
        let mut last_refill = self.last_refill.write().await;
        let elapsed = now.duration_since(*last_refill);

        if elapsed.as_millis() > 0 {
            let tokens_to_add = (elapsed.as_secs_f64() * self.refill_rate * 1000.0) as u64;
            if tokens_to_add > 0 {
                let current = self.tokens.load(Ordering::SeqCst);
                let max = (self.capacity as u64) * 1000;
                let new_tokens = (current + tokens_to_add).min(max);
                self.tokens.store(new_tokens, Ordering::SeqCst);
                *last_refill = now;
            }
        }
    }
}

/// Combined resilience helper
pub struct Resilience {
    pub circuit_breaker: Option<Arc<CircuitBreaker>>,
    pub rate_limiter: Option<Arc<RateLimiter>>,
}

impl Resilience {
    /// Create empty resilience (no protection)
    pub fn none() -> Self {
        Self {
            circuit_breaker: None,
            rate_limiter: None,
        }
    }

    /// Create with circuit breaker only
    pub fn with_circuit_breaker(cb: CircuitBreaker) -> Self {
        Self {
            circuit_breaker: Some(Arc::new(cb)),
            rate_limiter: None,
        }
    }

    /// Create with rate limiter only
    pub fn with_rate_limiter(rl: RateLimiter) -> Self {
        Self {
            circuit_breaker: None,
            rate_limiter: Some(Arc::new(rl)),
        }
    }

    /// Create with both protections
    pub fn with_both(cb: CircuitBreaker, rl: RateLimiter) -> Self {
        Self {
            circuit_breaker: Some(Arc::new(cb)),
            rate_limiter: Some(Arc::new(rl)),
        }
    }

    /// Check if request is allowed
    #[allow(dead_code)]
    pub async fn allow_request(&self) -> bool {
        // Check circuit breaker first
        if let Some(cb) = &self.circuit_breaker
            && !cb.allow_request().await
        {
            return false;
        }

        // Then check rate limiter
        if let Some(rl) = &self.rate_limiter
            && !rl.try_acquire().await
        {
            return false;
        }

        true
    }

    /// Record success
    pub async fn record_success(&self) {
        if let Some(cb) = &self.circuit_breaker {
            cb.record_success().await;
        }
    }

    /// Record failure
    pub async fn record_failure(&self) {
        if let Some(cb) = &self.circuit_breaker {
            cb.record_failure().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_opens() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(100), 2);

        assert_eq!(cb.state().await, CircuitState::Closed);
        assert!(cb.allow_request().await);

        // Record failures
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.state().await, CircuitState::Closed);

        cb.record_failure().await;
        assert_eq!(cb.state().await, CircuitState::Open);
        assert!(!cb.allow_request().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50), 2);

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.state().await, CircuitState::Open);

        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should be half-open now
        assert_eq!(cb.state().await, CircuitState::HalfOpen);
        assert!(cb.allow_request().await);

        // Record successes to close
        cb.record_success().await;
        cb.record_success().await;
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let rl = RateLimiter::new(2, 10.0);

        // Should allow burst
        assert!(rl.try_acquire().await);
        assert!(rl.try_acquire().await);

        // Should be rate limited
        assert!(!rl.try_acquire().await);

        // Wait for refill
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should allow one more
        assert!(rl.try_acquire().await);
    }
}
