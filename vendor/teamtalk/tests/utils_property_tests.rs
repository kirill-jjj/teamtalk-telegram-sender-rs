use proptest::prelude::*;
use std::time::Duration;
use teamtalk::utils::backoff::ExponentialBackoff;
use teamtalk::utils::math::ref_gain;

proptest! {
    #[test]
    fn ref_gain_monotonic(a in 0u8..=100, b in 0u8..=100) {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let lo_gain = ref_gain(lo as f64);
        let hi_gain = ref_gain(hi as f64);
        prop_assert!(lo_gain <= hi_gain);
    }

    #[test]
    fn backoff_respects_max(initial_ms in 0u64..=500, max_ms in 1u64..=1000) {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_millis(initial_ms),
            Duration::from_millis(max_ms),
            2.0,
            0.0,
        );
        let delay = backoff.next_delay();
        prop_assert!(delay <= Duration::from_millis(max_ms));
    }
}
