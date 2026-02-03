//! Math helpers used by TeamTalk types.

/// Converts a percentage (0.0-100.0) into a gain value.
pub fn ref_gain(percent: f64) -> i32 {
    if percent <= 0.0 {
        return 0;
    }
    let gain = 82.832 * (0.0508 * percent).exp() - 50.0;
    gain as i32
}

#[cfg(test)]
mod tests {
    use super::ref_gain;

    #[test]
    fn ref_gain_handles_zero_and_negative() {
        assert_eq!(ref_gain(0.0), 0);
        assert_eq!(ref_gain(-0.1), 0);
    }

    #[test]
    fn ref_gain_increases_with_percent() {
        let low = ref_gain(1.0);
        let high = ref_gain(10.0);
        assert!(high > low);
    }
}
