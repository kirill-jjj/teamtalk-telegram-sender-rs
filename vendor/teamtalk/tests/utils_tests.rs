use std::time::Duration;
use teamtalk::client::ffi;
use teamtalk::utils::backoff::ExponentialBackoff;
use teamtalk::utils::math::ref_gain;
use teamtalk::utils::strings::{ToTT, copy_to_string, from_tt, to_string};

#[test]
fn ref_gain_zero() {
    assert_eq!(ref_gain(0.0), 0);
    assert_eq!(ref_gain(-1.0), 0);
}

#[test]
fn ref_gain_increases() {
    let low = ref_gain(1.0);
    let high = ref_gain(5.0);
    assert!(high > low);
}

#[test]
fn backoff_caps_delay() {
    let mut backoff = ExponentialBackoff::new(
        Duration::from_millis(10),
        Duration::from_millis(20),
        2.0,
        0.0,
    );
    let d1 = backoff.next_delay();
    let d2 = backoff.next_delay();
    assert!(d1 <= Duration::from_millis(20));
    assert!(d2 <= Duration::from_millis(20));
}

#[test]
fn backoff_reset() {
    let mut backoff = ExponentialBackoff::new(
        Duration::from_millis(10),
        Duration::from_millis(100),
        1.6,
        0.0,
    );
    let _ = backoff.next_delay();
    assert!(backoff.attempts() >= 1);
    backoff.reset();
    assert_eq!(backoff.attempts(), 0);
    assert_eq!(backoff.current_delay(), Duration::ZERO);
}

#[test]
fn backoff_zero_max_delay() {
    let mut backoff =
        ExponentialBackoff::new(Duration::from_millis(0), Duration::from_millis(0), 2.0, 0.0);
    let delay = backoff.next_delay();
    assert_eq!(delay, Duration::ZERO);
    assert_eq!(backoff.current_delay(), Duration::ZERO);
}

#[test]
fn string_roundtrip() {
    let input = "TeamTalk";
    let tt = input.tt();
    let output = unsafe { from_tt(tt.as_ptr()) };
    assert_eq!(output, input);
}

#[test]
fn string_copy_to_string() {
    let input = "hello";
    let tt = input.tt();
    let mut buf = vec![0 as ffi::TTCHAR; tt.len() + 4];
    buf[..tt.len()].copy_from_slice(&tt);
    let text = to_string(&buf);
    assert_eq!(text, input);
    let mut out = String::new();
    copy_to_string(&buf, &mut out);
    assert_eq!(out, input);
}
