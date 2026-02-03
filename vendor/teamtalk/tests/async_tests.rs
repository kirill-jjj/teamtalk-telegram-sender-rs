#![cfg(feature = "async")]

use teamtalk::async_api::AsyncConfig;

#[test]
fn async_config_defaults() {
    let cfg = AsyncConfig::default();
    assert_eq!(cfg.poll_timeout_ms, 100);
    assert!(cfg.buffer > 0);
}

#[test]
fn async_config_builder() {
    let cfg = AsyncConfig::new().poll_timeout_ms(5).buffer(12);
    assert_eq!(cfg.poll_timeout_ms, 5);
    assert_eq!(cfg.buffer, 12);
}
