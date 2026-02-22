use super::*;
use crate::bootstrap::config::TeamTalkConfig;
use crate::core::types::{
    TtChannelName, TtClientName, TtHostName, TtLoginName, TtNickname, TtPassword, TtServerName,
};

fn base_config() -> TeamTalkConfig {
    TeamTalkConfig {
        host_name: TtHostName::from("host"),
        port: 1,
        encrypted: false,
        user_name: TtLoginName::from("u"),
        password: TtPassword::from("p"),
        channel: TtChannelName::try_from("/").unwrap(),
        channel_password: None,
        nick_name: TtNickname::from("n"),
        status_text: String::new(),
        client_name: TtClientName::from("c"),
        server_name: None,
        global_ignore_usernames: Vec::new(),
        guest_username: None,
    }
}

#[test]
fn resolve_server_name_prefers_config_server_name() {
    let mut cfg = base_config();
    cfg.server_name = Some(TtServerName::from("cfg"));
    let name = resolve_server_name(&cfg, Some("real"));
    assert_eq!(name, "cfg");
}

#[test]
fn resolve_server_name_uses_real_name_when_config_empty() {
    let mut cfg = base_config();
    cfg.server_name = Some(TtServerName::from(""));
    let name = resolve_server_name(&cfg, Some("real"));
    assert_eq!(name, "real");
}

#[test]
fn resolve_server_name_falls_back_to_host() {
    let cfg = base_config();
    let name = resolve_server_name(&cfg, Some(""));
    assert_eq!(name, "host");
}

#[test]
fn parse_status_gender_handles_values() {
    assert_eq!(
        parse_status_gender("male"),
        teamtalk::types::UserGender::Male
    );
    assert_eq!(
        parse_status_gender("female"),
        teamtalk::types::UserGender::Female
    );
    assert_eq!(
        parse_status_gender("unknown"),
        teamtalk::types::UserGender::Neutral
    );
}
