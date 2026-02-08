use super::load_config;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_temp_config(contents: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("tt_sender_cfg_{ts}.toml"));
    fs::write(&path, contents).expect("write temp config");
    path
}

fn valid_base_config() -> &'static str {
    r#"
[general]
admin_username = "admin"
default_lang = "ru"

[database]
db_file = "bot_data.db"

[telegram]
event_token = "token"
message_token = "token"
admin_chat_id = 123456

[teamtalk]
host_name = "example.com"
port = 10333
encrypted = false
user_name = "bot"
password = "pass"
channel = "/"
nick_name = "Bot"
client_name = "client"
"#
}

#[test]
fn invalid_type_reports_file_and_position() {
    let cfg = valid_base_config().replace("admin_chat_id = 123456", "admin_chat_id = \"oops\"");
    let path = write_temp_config(&cfg);

    let err = match load_config(&path) {
        Ok(_) => panic!("must fail"),
        Err(err) => err.to_string(),
    };

    assert!(err.contains("Configuration error in"));
    assert!(err.contains("Reason:"));
    assert!(err.contains("Location: line"));
    assert!(err.contains("How to fix:"));
    assert!(err.to_ascii_lowercase().contains("invalid type"));

    let _ = fs::remove_file(path);
}

#[test]
fn missing_field_reports_required_field_hint() {
    let cfg = valid_base_config().replace("admin_chat_id = 123456\n", "");
    let path = write_temp_config(&cfg);

    let err = match load_config(&path) {
        Ok(_) => panic!("must fail"),
        Err(err) => err.to_string(),
    };

    assert!(err.to_ascii_lowercase().contains("missing field"));
    assert!(err.contains("How to fix:"));
    assert!(err.contains("required sections and keys"));

    let _ = fs::remove_file(path);
}

#[test]
fn unknown_variant_reports_enum_hint() {
    let cfg = valid_base_config().replace("default_lang = \"ru\"", "default_lang = \"german\"");
    let path = write_temp_config(&cfg);

    let err = match load_config(&path) {
        Ok(_) => panic!("must fail"),
        Err(err) => err.to_string(),
    };

    assert!(err.to_ascii_lowercase().contains("unknown variant"));
    assert!(err.contains("supported enum values"));

    let _ = fs::remove_file(path);
}

#[test]
fn syntax_error_reports_snippet() {
    let cfg = valid_base_config().replace("[telegram]", "[telegram");
    let path = write_temp_config(&cfg);

    let err = match load_config(&path) {
        Ok(_) => panic!("must fail"),
        Err(err) => err.to_string(),
    };

    assert!(err.contains("Configuration error in"));
    assert!(err.contains("Location: line"));
    assert!(err.contains('|'));
    assert!(err.contains('^'));

    let _ = fs::remove_file(path);
}
