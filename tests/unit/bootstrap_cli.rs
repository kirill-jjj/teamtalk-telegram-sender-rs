use crate::bootstrap::cli::{collect_config_paths, instance_name_from_path, read_log_level};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn collect_config_paths_defaults_to_config_toml() {
    let args = vec!["bin".to_string()];
    let paths = collect_config_paths(&args).unwrap();
    assert_eq!(paths, vec!["config.toml".to_string()]);
}

#[test]
fn collect_config_paths_supports_multiple() {
    let args = vec![
        "bin".to_string(),
        "--config".to_string(),
        "a.toml".to_string(),
        "--config".to_string(),
        "b.toml".to_string(),
    ];
    let paths = collect_config_paths(&args).unwrap();
    assert_eq!(paths, vec!["a.toml".to_string(), "b.toml".to_string()]);
}

#[test]
fn collect_config_paths_missing_value_is_error() {
    let args = vec!["bin".to_string(), "--config".to_string()];
    assert!(collect_config_paths(&args).is_err());
}

#[test]
fn instance_name_uses_file_stem() {
    let name = instance_name_from_path("configs/server-a.toml");
    assert_eq!(name, "server-a");
}

#[test]
fn read_log_level_from_config() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let config_path = std::env::temp_dir().join(format!("tt_sender_config_{nonce}.toml"));
    std::fs::write(
        &config_path,
        r#"
        [general]
        default_lang = "en"
        log_level = "warn"
        [database]
        db_file = "test.db"
        [telegram]
        admin_chat_id = 1
        [teamtalk]
        host_name = "host"
        port = 1
        encrypted = false
        user_name = "u"
        password = "p"
        channel = "/"
        channel_password = ""
        nick_name = "n"
        status_text = ""
        client_name = "c"
        global_ignore_usernames = []
        "#,
    )
    .unwrap();
    let level = read_log_level(config_path.to_str().unwrap()).unwrap();
    assert_eq!(level, "warn");
    let _ = std::fs::remove_file(config_path);
}
