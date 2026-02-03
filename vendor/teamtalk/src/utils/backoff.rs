//! Exponential backoff helper.
use rand::{Rng, thread_rng};
use std::time::Duration;

/// Exponential backoff with jitter and a maximum cap.
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    initial_delay: Duration,
    max_delay: Duration,
    factor: f32,
    attempts: u32,
    current_val: Duration,
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(120),
            factor: 1.6,
            attempts: 0,
            current_val: Duration::ZERO,
        }
    }
}

impl ExponentialBackoff {
    /// Creates a new backoff schedule.
    pub fn new(initial: Duration, max: Duration, factor: f32, _jitter: f32) -> Self {
        Self {
            initial_delay: initial,
            max_delay: max,
            factor,
            attempts: 0,
            current_val: Duration::ZERO,
        }
    }

    /// Returns the next delay in the schedule.
    pub fn next_delay(&mut self) -> Duration {
        if self.attempts == 0 && self.initial_delay.is_zero() {
            self.attempts += 1;
            self.current_val = Duration::ZERO;
            return Duration::ZERO;
        }

        let base = if self.initial_delay.is_zero() {
            Duration::from_millis(100)
        } else {
            self.initial_delay
        };

        let exponent = self.attempts as f32;
        let cap_secs = base.as_secs_f32() * self.factor.powf(exponent);
        let cap = Duration::from_secs_f32(cap_secs).min(self.max_delay);

        self.attempts += 1;

        let max_millis = cap.as_millis() as u64;
        if max_millis == 0 {
            self.current_val = Duration::ZERO;
            return Duration::ZERO;
        }

        let jittered = thread_rng().gen_range(0..=max_millis);
        self.current_val = Duration::from_millis(jittered);
        self.current_val
    }

    /// Returns the current delay without advancing.
    pub fn current_delay(&self) -> Duration {
        self.current_val
    }

    /// Resets the schedule to its initial state.
    pub fn reset(&mut self) {
        self.attempts = 0;
        self.current_val = Duration::ZERO;
    }

    /// Returns the number of attempts.
    pub fn attempts(&self) -> u32 {
        self.attempts
    }
}

#[cfg(test)]
mod tests {
    use super::ExponentialBackoff;
    use std::time::Duration;

    #[test]
    fn default_values() {
        let backoff = ExponentialBackoff::default();
        assert_eq!(backoff.attempts(), 0);
        assert_eq!(backoff.current_delay(), Duration::ZERO);
    }

    #[test]
    fn zero_initial_delay_returns_zero() {
        let mut backoff =
            ExponentialBackoff::new(Duration::ZERO, Duration::from_millis(100), 2.0, 0.0);
        let delay = backoff.next_delay();
        assert_eq!(delay, Duration::ZERO);
    }
}
