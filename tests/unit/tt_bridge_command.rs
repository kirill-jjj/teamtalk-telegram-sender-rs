use super::*;
use std::time::{Duration, Instant};

#[test]
fn disabled_hint_is_sent_when_no_previous_reply() {
    let now = Instant::now();
    assert!(should_send_disabled_hint(
        None,
        now,
        Duration::from_secs(120)
    ));
}

#[test]
fn disabled_hint_is_suppressed_within_cooldown() {
    let now = Instant::now();
    let last = now.checked_sub(Duration::from_secs(30)).unwrap();
    assert!(!should_send_disabled_hint(
        Some(last),
        now,
        Duration::from_secs(120)
    ));
}

#[test]
fn disabled_hint_is_sent_after_cooldown() {
    let now = Instant::now();
    let last = now.checked_sub(Duration::from_secs(180)).unwrap();
    assert!(should_send_disabled_hint(
        Some(last),
        now,
        Duration::from_secs(120)
    ));
}
