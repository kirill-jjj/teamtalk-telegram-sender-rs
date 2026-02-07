use crate::app::plugins::runtime::{
    PluginManifest, PluginRuntime, event_envelope, normalized_tg_context, normalized_tt_context,
    should_disable,
};
use crate::app::plugins::{parse_command_text, plugin_name_from_path};
use crate::core::types::{TtUserId, TtUsername};
use serde_json::json;
use std::collections::VecDeque;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::unbounded_channel;

fn make_temp_plugin_dir(prefix: &str) -> std::path::PathBuf {
    let unique = format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

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

#[test]
fn register_command_source_filter_blocks_tt_when_disabled() {
    let temp_dir = make_temp_plugin_dir("plugin_test_filter_tt");
    let entry_path = temp_dir.join("main.lua");
    std::fs::write(
        &entry_path,
        r#"
register_command("ping", function(args, ctx)
    return true
end, { tg = true, tt = false })
"#,
    )
    .expect("write lua");

    let manifest = PluginManifest {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        entry: "main.lua".to_string(),
        enabled: true,
    };
    let (tx, _rx) = unbounded_channel();
    let runtime = PluginRuntime::load(&temp_dir, &manifest, tx, Duration::from_millis(100))
        .expect("runtime load");

    let tg_ctx = json!({"source":"tg"});
    let tt_ctx = json!({"source":"tt"});
    assert!(
        runtime
            .dispatch_command("ping", &Vec::<String>::new(), &tg_ctx)
            .expect("dispatch tg")
    );
    assert!(
        !runtime
            .dispatch_command("ping", &Vec::<String>::new(), &tt_ctx)
            .expect("dispatch tt")
    );
    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn register_command_source_filter_defaults_to_both_sources() {
    let temp_dir = make_temp_plugin_dir("plugin_test_filter_default");
    let entry_path = temp_dir.join("main.lua");
    std::fs::write(
        &entry_path,
        r#"
register_command("ping", function(args, ctx)
    return true
end)
"#,
    )
    .expect("write lua");

    let manifest = PluginManifest {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        entry: "main.lua".to_string(),
        enabled: true,
    };
    let (tx, _rx) = unbounded_channel();
    let runtime = PluginRuntime::load(&temp_dir, &manifest, tx, Duration::from_millis(100))
        .expect("runtime load");

    let tg_ctx = json!({"source":"tg"});
    let tt_ctx = json!({"source":"tt"});
    assert!(
        runtime
            .dispatch_command("ping", &Vec::<String>::new(), &tg_ctx)
            .expect("dispatch tg")
    );
    assert!(
        runtime
            .dispatch_command("ping", &Vec::<String>::new(), &tt_ctx)
            .expect("dispatch tt")
    );
    let _ = std::fs::remove_dir_all(temp_dir);
}
