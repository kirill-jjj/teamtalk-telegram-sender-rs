use crate::app::plugins::runtime::{
    event_envelope, normalized_tg_context, normalized_tt_context, should_disable,
};
use crate::app::plugins::{parse_command_text, plugin_name_from_path};
use crate::core::types::{TtUserId, TtUsername};
use serde_json::json;
use std::collections::VecDeque;
use std::path::Path;
use std::time::{Duration, Instant};

#[test]
fn parse_command_text_parses_basic_command() {
    let parsed = parse_command_text("/ping one two").expect("command should parse");
    assert_eq!(parsed.0, "ping");
    assert_eq!(parsed.1, vec!["one".to_string(), "two".to_string()]);
}

#[test]
fn parse_command_text_strips_bot_suffix() {
    let parsed = parse_command_text("/ping@my_bot alpha").expect("command should parse");
    assert_eq!(parsed.0, "ping");
    assert_eq!(parsed.1, vec!["alpha".to_string()]);
}

#[test]
fn parse_command_text_rejects_empty() {
    assert!(parse_command_text("").is_none());
    assert!(parse_command_text("   ").is_none());
}

#[test]
fn plugin_name_from_path_extracts_top_level_dir() {
    let root = Path::new("plugins");
    let path = Path::new("plugins/rec/main.lua");
    let name = plugin_name_from_path(root, path).expect("name expected");
    assert_eq!(name, "rec");
}

#[test]
fn plugin_name_from_path_returns_none_outside_root() {
    let root = Path::new("plugins");
    let path = Path::new("other/rec/main.lua");
    assert!(plugin_name_from_path(root, path).is_none());
}

#[test]
fn should_disable_triggers_at_threshold() {
    let mut errors = VecDeque::new();
    let window = Duration::from_secs(60);
    assert!(!should_disable(&mut errors, window, 3));
    assert!(!should_disable(&mut errors, window, 3));
    assert!(should_disable(&mut errors, window, 3));
}

#[test]
fn should_disable_drops_old_errors_by_window() {
    let old = Instant::now()
        .checked_sub(Duration::from_secs(120))
        .expect("instant subtraction should be valid");
    let mut errors = VecDeque::from([old]);
    let window = Duration::from_secs(60);
    assert!(!should_disable(&mut errors, window, 2));
    assert_eq!(errors.len(), 1);
}

#[test]
fn event_envelope_contains_all_fields() {
    let normalized = json!({"a": 1});
    let raw = json!({"b": 2});
    let event = event_envelope("UserLoggedIn", "tt", &normalized, &raw);
    assert_eq!(event["name"], "UserLoggedIn");
    assert_eq!(event["source"], "tt");
    assert_eq!(event["normalized"]["a"], 1);
    assert_eq!(event["raw"]["b"], 2);
}

#[test]
fn normalized_tg_context_contains_expected_shape() {
    let ctx = crate::app::plugins::TgCommandContext {
        chat_id: 10,
        user_id: 20,
        is_admin: true,
        text: "/ping".to_string(),
    };
    let value = normalized_tg_context(&ctx);
    assert_eq!(value["source"], "tg");
    assert_eq!(value["chat_id"], 10);
    assert_eq!(value["user_id"], 20);
    assert_eq!(value["is_admin"], true);
}

#[test]
fn normalized_tt_context_contains_expected_shape() {
    let ctx = crate::app::plugins::TtCommandContext {
        user_id: TtUserId::from(11),
        username: TtUsername::from("user"),
        nickname: "Nick".to_string(),
        is_admin: false,
        text: "/ping".to_string(),
    };
    let value = normalized_tt_context(&ctx);
    assert_eq!(value["source"], "tt");
    assert_eq!(value["user_id"], 11);
    assert_eq!(value["username"], "user");
    assert_eq!(value["nickname"], "Nick");
    assert_eq!(value["is_admin"], false);
}
