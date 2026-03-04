//! # GCRA - Generic Cell Rate Algorithm
//!
//! A Rust implementation of the Generic Cell Rate Algorithm (GCRA) for rate limiting.
//!
//! GCRA is a traffic shaping and rate limiting algorithm based on the leaky bucket algorithm.
//! It provides precise control over request rates while allowing for burst traffic.
//!
//! ## Core Concepts
//!
//! - **T (Tick)**: The minimum time interval between two consecutive requests
//! - **τ (Tau)**: The tolerance, representing the maximum burst time that can be accumulated
//! - **TAT (Theoretical Arrival Time)**: The theoretical arrival time of the next request
//!
//! ## Usage Example
//!
//! ```rust
//! use gcra::RateLimiter;
//! use std::time::Duration;
//!
//! // Create a rate limiter allowing 10 requests per second with a burst of 5
//! let mut limiter = RateLimiter::new(10, Duration::from_secs(1), 5);
//!
//! if limiter.acquire().allowed {
//!     println!("Request allowed");
//! } else {
//!     println!("Request denied");
//! }
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Result of a rate limit acquisition attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcquireResult {
    /// Whether the request was allowed.
    pub allowed: bool,
    /// Estimated time to wait before the next request can be made (if denied).
    pub retry_after: Option<Duration>,
}

impl AcquireResult {
    /// Creates a new allowed result.
    #[inline]
    pub const fn allowed() -> Self {
        Self {
            allowed: true,
            retry_after: None,
        }
    }

    /// Creates a new denied result with optional retry time.
    #[inline]
    pub const fn denied(retry_after: Option<Duration>) -> Self {
        Self {
            allowed: false,
            retry_after,
        }
    }

    /// Returns `true` if the request was allowed.
    #[inline]
    pub const fn is_allowed(self) -> bool {
        self.allowed
    }

    /// Returns `true` if the request was denied.
    #[inline]
    pub const fn is_denied(self) -> bool {
        !self.allowed
    }
}

/// GCRA Rate Limiter
///
/// Implements precise rate control using the Generic Cell Rate Algorithm.
///
/// # Example
///
/// ```
/// use gcra::RateLimiter;
/// use std::time::Duration;
///
/// let mut limiter = RateLimiter::new(10, Duration::from_secs(1), 5);
///
/// // Acquire permits
/// for _ in 0..5 {
///     assert!(limiter.acquire().allowed);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Minimum time interval between requests (T) in nanoseconds
    tick_ns: u64,
    /// Maximum accumulated burst time (τ) in nanoseconds
    tolerance_ns: u64,
    /// Theoretical arrival time of the next request (TAT) in nanoseconds since first request
    tat_ns: Option<u64>,
    /// Baseline time for calculating relative time
    baseline: Instant,
}

impl RateLimiter {
    /// Creates a new rate limiter.
    ///
    /// # Arguments
    ///
    /// * `rate` - Number of requests allowed per time period, must be greater than 0
    /// * `per` - Time period
    /// * `burst` - Maximum number of burst requests allowed
    ///
    /// # Panics
    ///
    /// Panics if `rate` is 0.
    ///
    /// # Example
    ///
    /// ```
    /// use gcra::RateLimiter;
    /// use std::time::Duration;
    ///
    /// // 10 requests per second, burst of 5
    /// let limiter = RateLimiter::new(10, Duration::from_secs(1), 5);
    /// ```
    pub fn new(rate: u64, per: Duration, burst: u64) -> Self {
        assert!(rate > 0, "rate must be greater than 0");

        // Calculate tick in nanoseconds, guarding against overflow
        let tick_ns: u64 = (per.as_nanos() / rate as u128)
            .try_into()
            .expect("tick duration too large for u64");
        let tolerance_ns = tick_ns.saturating_mul(burst);

        Self {
            tick_ns,
            tolerance_ns,
            tat_ns: None,
            baseline: Instant::now(),
        }
    }

    /// Attempts to acquire a permit from the rate limiter.
    ///
    /// Returns an [`AcquireResult`] with details about the acquisition attempt.
    ///
    /// This method uses the GCRA algorithm:
    /// 1. If this is the first request, allow it and set TAT = now + T
    /// 2. Calculate the earliest allowed arrival time: TAT - τ
    /// 3. If now >= TAT - τ, the request is allowed, update TAT = max(now, TAT) + T
    /// 4. If now < TAT - τ, the request is denied
    ///
    /// # Example
    ///
    /// ```
    /// use gcra::RateLimiter;
    /// use std::time::Duration;
    ///
    /// let mut limiter = RateLimiter::new(10, Duration::from_secs(1), 5);
    ///
    /// let result = limiter.acquire();
    /// if result.allowed {
    ///     // Process the request
    /// } else if let Some(wait_time) = result.retry_after {
    ///     println!("Retry after {:?}", wait_time);
    /// }
    /// ```
    pub fn acquire(&mut self) -> AcquireResult {
        let now_ns = self.now_ns();
        self.acquire_at_ns(now_ns)
    }

    /// Attempts to acquire a permit at the given time.
    ///
    /// This is useful for testing or when you need to control the timing manually.
    fn acquire_at_ns(&mut self, now_ns: u64) -> AcquireResult {
        match self.tat_ns {
            None => {
                // First request: always allowed
                self.tat_ns = Some(now_ns + self.tick_ns);
                AcquireResult::allowed()
            }
            Some(tat_ns) => {
                // Calculate earliest allowed time
                let earliest_ns = match tat_ns.checked_sub(self.tolerance_ns) {
                    None => return self.allow_request(now_ns, tat_ns), // tolerance > tat, always allow
                    Some(earliest) => earliest,
                };

                if now_ns < earliest_ns {
                    // Request too fast: deny
                    let retry_after_ns = earliest_ns.saturating_sub(now_ns);
                    AcquireResult::denied(Some(Duration::from_nanos(retry_after_ns)))
                } else {
                    // Request allowed
                    self.allow_request(now_ns, tat_ns)
                }
            }
        }
    }

    /// Allows the request and updates TAT.
    #[inline]
    fn allow_request(&mut self, now_ns: u64, tat_ns: u64) -> AcquireResult {
        let new_tat_ns = if now_ns > tat_ns {
            now_ns.saturating_add(self.tick_ns)
        } else {
            tat_ns.saturating_add(self.tick_ns)
        };
        self.tat_ns = Some(new_tat_ns);
        AcquireResult::allowed()
    }

    /// Resets the rate limiter state.
    ///
    /// This clears the internal state, allowing the next request to pass immediately.
    ///
    /// # Example
    ///
    /// ```
    /// use gcra::RateLimiter;
    /// use std::time::Duration;
    ///
    /// let mut limiter = RateLimiter::new(1, Duration::from_secs(1), 0);
    ///
    /// limiter.acquire();
    /// assert!(!limiter.acquire().allowed); // Should be denied
    ///
    /// limiter.reset();
    /// assert!(limiter.acquire().allowed); // Should be allowed after reset
    /// ```
    pub fn reset(&mut self) {
        self.tat_ns = None;
    }

    /// Gets current time in nanoseconds relative to baseline.
    #[inline]
    fn now_ns(&self) -> u64 {
        Instant::now().duration_since(self.baseline).as_nanos() as u64
    }
}

/// Thread-safe GCRA Rate Limiter.
///
/// Uses atomic variables for lock-free thread safety, providing better concurrent performance
/// than mutex-based implementations. Suitable for high-concurrency scenarios.
///
/// # Example
///
/// ```
/// use gcra::SyncRateLimiter;
/// use std::time::Duration;
/// use std::sync::Arc;
/// use std::thread;
///
/// let limiter = Arc::new(SyncRateLimiter::new(10, Duration::from_secs(1), 5));
///
/// for _ in 0..5 {
///     let limiter = Arc::clone(&limiter);
///     thread::spawn(move || {
///         if limiter.acquire().allowed {
///             println!("Request allowed");
///         }
///     });
/// }
/// ```
pub struct SyncRateLimiter {
    /// Minimum time interval between requests (T) in nanoseconds
    tick_ns: u64,
    /// Maximum accumulated burst time (τ) in nanoseconds
    tolerance_ns: u64,
    /// TAT stored as nanoseconds relative to baseline (0 = uninitialized)
    tat_ns: AtomicU64,
    /// Baseline time for calculating relative time
    baseline: Instant,
}

impl std::fmt::Debug for SyncRateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncRateLimiter")
            .field("tick_ns", &self.tick_ns)
            .field("tolerance_ns", &self.tolerance_ns)
            .field("tat_ns", &self.tat_ns.load(Ordering::Relaxed))
            .finish()
    }
}

impl Clone for SyncRateLimiter {
    fn clone(&self) -> Self {
        Self {
            tick_ns: self.tick_ns,
            tolerance_ns: self.tolerance_ns,
            tat_ns: AtomicU64::new(self.tat_ns.load(Ordering::Acquire)),
            baseline: self.baseline,
        }
    }
}

impl SyncRateLimiter {
    /// Creates a new thread-safe rate limiter.
    ///
    /// # Arguments
    ///
    /// * `rate` - Number of requests allowed per time period, must be greater than 0
    /// * `per` - Time period
    /// * `burst` - Maximum number of burst requests allowed
    ///
    /// # Panics
    ///
    /// Panics if `rate` is 0.
    pub fn new(rate: u64, per: Duration, burst: u64) -> Self {
        assert!(rate > 0, "rate must be greater than 0");

        let tick_ns = (per.as_nanos() / rate as u128) as u64;
        let tolerance_ns = tick_ns.saturating_mul(burst);

        Self {
            tick_ns,
            tolerance_ns,
            tat_ns: AtomicU64::new(0),
            baseline: Instant::now(),
        }
    }

    /// Attempts to acquire a permit from the rate limiter.
    ///
    /// Returns an [`AcquireResult`] with details about the acquisition attempt.
    ///
    /// This method is thread-safe and uses lock-free atomic operations with optimized
    /// memory ordering (AcqRel instead of SeqCst) for better performance.
    pub fn acquire(&self) -> AcquireResult {
        let now_ns = self.now_ns();

        loop {
            let tat_ns = self.tat_ns.load(Ordering::Acquire);

            if tat_ns == 0 {
                // First request: try to initialize TAT
                let new_tat_ns = now_ns.saturating_add(self.tick_ns);
                match self.tat_ns.compare_exchange_weak(
                    0,
                    new_tat_ns,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => return AcquireResult::allowed(),
                    Err(_) => continue, // Another thread got there first, retry
                }
            }

            // Calculate earliest allowed time
            let earliest_ns = tat_ns.saturating_sub(self.tolerance_ns);

            if now_ns < earliest_ns {
                // Request too fast: deny
                let retry_after_ns = earliest_ns.saturating_sub(now_ns);
                return AcquireResult::denied(Some(Duration::from_nanos(retry_after_ns)));
            }

            // Request allowed: calculate new TAT
            let new_tat_ns = if now_ns > tat_ns {
                now_ns.saturating_add(self.tick_ns)
            } else {
                tat_ns.saturating_add(self.tick_ns)
            };

            // Try to atomically update TAT
            match self.tat_ns.compare_exchange_weak(
                tat_ns,
                new_tat_ns,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return AcquireResult::allowed(),
                Err(_) => continue, // TAT was modified by another thread, retry
            }
        }
    }

    /// Resets the rate limiter state.
    ///
    /// This clears the internal state, allowing the next request to pass immediately.
    pub fn reset(&self) {
        self.tat_ns.store(0, Ordering::Release);
    }

    /// Gets current time in nanoseconds relative to baseline.
    #[inline]
    fn now_ns(&self) -> u64 {
        Instant::now().duration_since(self.baseline).as_nanos() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_basic_rate_limiting() {
        let mut limiter = RateLimiter::new(10, Duration::from_secs(1), 3);

        // First 4 should be allowed (burst 3 + 1)
        assert!(limiter.acquire().allowed);
        assert!(limiter.acquire().allowed);
        assert!(limiter.acquire().allowed);
        assert!(limiter.acquire().allowed);

        // 5th should be denied
        assert!(!limiter.acquire().allowed);
    }

    #[test]
    fn test_burst() {
        let mut limiter = RateLimiter::new(10, Duration::from_secs(1), 5);

        // All 6 requests should be allowed (burst 5 + 1)
        for _ in 0..6 {
            assert!(limiter.acquire().allowed);
        }

        // 7th should be denied
        assert!(!limiter.acquire().allowed);
    }

    #[test]
    fn test_reset() {
        let mut limiter = RateLimiter::new(10, Duration::from_secs(1), 1);

        limiter.acquire();
        limiter.acquire();
        assert!(!limiter.acquire().allowed);

        limiter.reset();
        assert!(limiter.acquire().allowed);
    }

    #[test]
    fn test_sync_rate_limiter() {
        let limiter = SyncRateLimiter::new(10, Duration::from_secs(1), 3);

        // First 4 should be allowed (burst 3 + 1)
        assert!(limiter.acquire().allowed);
        assert!(limiter.acquire().allowed);
        assert!(limiter.acquire().allowed);
        assert!(limiter.acquire().allowed);

        // 5th should be denied
        assert!(!limiter.acquire().allowed);

        // reset restores
        limiter.reset();
        assert!(limiter.acquire().allowed);
    }

    #[test]
    fn test_sync_rate_limiter_thread_safety() {
        use std::sync::Arc;

        let limiter = Arc::new(SyncRateLimiter::new(100, Duration::from_secs(1), 50));
        let mut handles = vec![];

        for _ in 0..10 {
            let limiter = Arc::clone(&limiter);
            let handle = thread::spawn(move || {
                let mut allowed = 0;
                for _ in 0..10 {
                    if limiter.acquire().allowed {
                        allowed += 1;
                    }
                }
                allowed
            });
            handles.push(handle);
        }

        let total_allowed: i32 = handles.into_iter().map(|h| h.join().unwrap()).sum();

        // With burst of 50 and 100/sec rate, we should have at least 50+ allowed
        assert!(total_allowed >= 50);
    }

    #[test]
    #[should_panic(expected = "rate must be greater than 0")]
    fn test_rate_limiter_new_panics_on_zero_rate() {
        RateLimiter::new(0, Duration::from_secs(1), 5);
    }

    #[test]
    #[should_panic(expected = "rate must be greater than 0")]
    fn test_sync_rate_limiter_new_panics_on_zero_rate() {
        SyncRateLimiter::new(0, Duration::from_secs(1), 5);
    }

    #[test]
    fn test_acquire_result_helpers() {
        let result = AcquireResult::allowed();
        assert!(result.is_allowed());
        assert!(!result.is_denied());
        assert!(result.retry_after.is_none());

        let result = AcquireResult::denied(Some(Duration::from_millis(100)));
        assert!(!result.is_allowed());
        assert!(result.is_denied());
        assert!(result.retry_after.is_some());
    }

    #[test]
    fn test_retry_after() {
        let mut limiter = RateLimiter::new(1, Duration::from_secs(1), 0);

        limiter.acquire();
        let result = limiter.acquire();

        assert!(!result.allowed);
        assert!(result.retry_after.is_some());
    }

    #[test]
    fn test_long_running() {
        // Test that the rate limiter handles long-running scenarios without overflow
        let mut limiter = RateLimiter::new(1000, Duration::from_secs(1), 100);

        // Simulate 1 million requests
        for i in 0..1_000_000 {
            let _result = limiter.acquire();
            // Just ensure it doesn't panic
            if i % 100_000 == 0 {
                // Reset periodically to avoid all requests being denied
                limiter.reset();
            }
        }
    }

    #[test]
    fn test_boundary_conditions() {
        // Test with minimal rate
        let mut limiter = RateLimiter::new(1, Duration::from_secs(1), 0);
        assert!(limiter.acquire().allowed);
        assert!(!limiter.acquire().allowed);

        // Test with zero burst
        let mut limiter = RateLimiter::new(10, Duration::from_secs(1), 0);
        assert!(limiter.acquire().allowed);
        assert!(!limiter.acquire().allowed);

        // Test with large burst
        let mut limiter = RateLimiter::new(1, Duration::from_secs(1), 1000);
        for _ in 0..1001 {
            assert!(limiter.acquire().allowed);
        }
        assert!(!limiter.acquire().allowed);
    }

    #[test]
    fn test_sync_rate_limiter_long_running() {
        use std::sync::Arc;

        let limiter = Arc::new(SyncRateLimiter::new(1000, Duration::from_secs(1), 100));
        let mut handles = vec![];

        for _ in 0..4 {
            let limiter = Arc::clone(&limiter);
            let handle = thread::spawn(move || {
                for _ in 0..250_000 {
                    limiter.acquire();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
        // Test passes if no panic occurs
    }
}
