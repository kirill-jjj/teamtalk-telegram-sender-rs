use super::*;

fn parse_config(toml_str: &str) -> Config {
    toml::from_str::<Config>(toml_str).unwrap()
}

#[test]
fn display_name_prefers_server_name() {
    let cfg = parse_config(
        r#"
            [general]
            admin_username = "admin"

            [database]
            db_file = "test.db"

            [telegram]
            event_token = "t"
            message_token = "m"
            admin_chat_id = 1

            [teamtalk]
            host_name = "host"
            port = 1
            encrypted = false
            user_name = "u"
            password = "p"
            channel = "/"
            nick_name = "n"
            client_name = "c"
            server_name = "srv"
            "#,
    );

    assert_eq!(cfg.teamtalk.display_name(), "srv");
}

#[test]
fn display_name_falls_back_to_host() {
    let cfg = parse_config(
        r#"
            [general]
            admin_username = "admin"

            [database]
            db_file = "test.db"

            [telegram]
            event_token = "t"
            message_token = "m"
            admin_chat_id = 1

            [teamtalk]
            host_name = "host"
            port = 1
            encrypted = false
            user_name = "u"
            password = "p"
            channel = "/"
            nick_name = "n"
            client_name = "c"
            server_name = ""
            "#,
    );

    assert_eq!(cfg.teamtalk.display_name(), "host");
}

#[test]
fn defaults_are_applied() {
    let cfg = parse_config(
        r#"
            [general]
            admin_username = "admin"

            [database]
            db_file = "test.db"

            [telegram]
            event_token = "t"
            message_token = "m"
            admin_chat_id = 1

            [teamtalk]
            host_name = "host"
            port = 1
            encrypted = false
            user_name = "u"
            password = "p"
            channel = "/"
            nick_name = "n"
            client_name = "c"
            "#,
    );

    assert_eq!(cfg.general.default_lang, "en");
    assert_eq!(cfg.general.log_level, "info");
    assert_eq!(cfg.general.gender, "None");
    assert_eq!(cfg.operational_parameters.deeplink_ttl, 300);
    assert_eq!(cfg.operational_parameters.tt_reconnect_retry, 10);
}

#[test]
fn operational_parameters_override() {
    let cfg = parse_config(
        r#"
            [general]
            admin_username = "admin"

            [database]
            db_file = "test.db"

            [telegram]
            event_token = "t"
            message_token = "m"
            admin_chat_id = 1

            [teamtalk]
            host_name = "host"
            port = 1
            encrypted = false
            user_name = "u"
            password = "p"
            channel = "/"
            nick_name = "n"
            client_name = "c"

            [operational_parameters]
            deeplink_ttl_seconds = 111
            tt_reconnect_retry_seconds = 22
            deeplink_cleanup_interval_seconds = 33
            tt_reconnect_check_interval_seconds = 44
            "#,
    );

    assert_eq!(cfg.operational_parameters.deeplink_ttl, 111);
    assert_eq!(cfg.operational_parameters.tt_reconnect_retry, 22);
    assert_eq!(cfg.operational_parameters.deeplink_cleanup_interval, 33);
    assert_eq!(cfg.operational_parameters.tt_reconnect_check_interval, 44);
}
