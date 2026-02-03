use std::time::Duration;

use teamtalk::client::connection::{ConnectParamsOwned, ReconnectConfig, ReconnectHandler};

#[test]
fn reconnect_handler_resets_after_stable_connection() {
    let config = ReconnectConfig {
        max_attempts: 10,
        min_delay: Duration::from_millis(0),
        max_delay: Duration::from_millis(0),
        stability_threshold: Duration::from_millis(0),
    };
    let mut handler = ReconnectHandler::new(config);
    handler.record_attempt();
    handler.record_attempt();
    assert_eq!(handler.attempts(), 2);
    handler.mark_connected();
    handler.mark_disconnected();
    assert_eq!(handler.attempts(), 0);
    assert_eq!(handler.current_delay(), Duration::from_millis(0));
}

#[test]
fn reconnect_handler_respects_max_attempts() {
    let config = ReconnectConfig {
        max_attempts: 1,
        min_delay: Duration::from_millis(0),
        max_delay: Duration::from_millis(0),
        stability_threshold: Duration::from_millis(0),
    };
    let mut handler = ReconnectHandler::new(config);
    assert!(handler.can_attempt());
    handler.record_attempt();
    assert!(!handler.can_attempt());
}

#[test]
fn connect_params_from_env_parses_values() {
    unsafe {
        std::env::set_var("TT_HOST", "example.com");
        std::env::set_var("TT_TCP", "10443");
        std::env::set_var("TT_UDP", "10555");
        std::env::set_var("TT_ENCRYPTED", "true");
    }

    let params = ConnectParamsOwned::from_env();
    assert_eq!(params.host, "example.com");
    assert_eq!(params.tcp, 10443);
    assert_eq!(params.udp, 10555);
    assert!(params.encrypted);

    unsafe {
        std::env::remove_var("TT_HOST");
        std::env::remove_var("TT_TCP");
        std::env::remove_var("TT_UDP");
        std::env::remove_var("TT_ENCRYPTED");
    }
}
