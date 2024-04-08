use std::time::Duration;

/// Represents a backoff policy for retrying operations. If the reset duration
/// passes, the backoff delay is reset to the minimum delay.
#[derive(Debug, Clone, Copy)]
pub struct BackoffPolicy {
    min_delay: Duration,
    max_delay: Duration,
    reset_after: Duration,
    multiplier: f64,
}

impl BackoffPolicy {
    /// Creates a new `BackoffPolicy` with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `min_delay` - The minimum delay between retries.
    /// * `max_delay` - The maximum delay between retries.
    /// * `reset_after` - The maximum delay between retries.
    /// * `multiplier` - The multiplier applied to the delay between retries.
    pub fn new(
        min_delay: std::time::Duration,
        max_delay: std::time::Duration,
        reset_after: std::time::Duration,
        multiplier: f64,
    ) -> Self {
        debug_assert!(reset_after > max_delay);
        debug_assert!(max_delay > min_delay);
        debug_assert!(min_delay > Duration::from_millis(0));
        debug_assert!(multiplier > 1.0);
        Self {
            min_delay,
            max_delay,
            reset_after,
            multiplier,
        }
    }

    /// Returns the minimum delay between retries.
    pub fn min_delay(&self) -> Duration {
        self.min_delay
    }

    /// Returns the maximum delay between retries.
    pub fn max_delay(&self) -> Duration {
        self.max_delay
    }

    /// Returns the reset duration
    pub fn reset_after(&self) -> Duration {
        self.reset_after
    }

    /// Returns the multiplier applied to the delay between retries.
    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }
}

impl Default for BackoffPolicy {
    /// Returns the default `BackoffPolicy`.
    ///
    /// The default values are:
    /// * `min_delay` - 50 milliseconds
    /// * `max_delay` - 60 seconds
    /// * `reset_after` - 120 seconds
    /// * `multiplier` - 1.2
    fn default() -> Self {
        Self {
            min_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(60),
            reset_after: Duration::from_secs(120),
            multiplier: 1.2,
        }
    }
}
