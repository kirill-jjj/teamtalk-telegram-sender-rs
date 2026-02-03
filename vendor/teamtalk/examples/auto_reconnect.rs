use std::env;
use teamtalk::types::ChannelId;
use teamtalk::{Client, Event, LoginParams, ReconnectConfig};

fn env_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn env_or_i32(name: &str, default: i32) -> i32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(default)
}

fn main() -> teamtalk::Result<()> {
    let host = env_or("TT_HOST", "127.0.0.1");
    let tcp = env_or_i32("TT_TCP", 10333);
    let udp = env_or_i32("TT_UDP", 10333);
    let nickname = env_or("TT_NICK", "AutoBot");
    let username = env_or("TT_USER", "guest");
    let password = env_or("TT_PASS", "guest");
    let client_name = env_or("TT_CLIENT", "TeamTalkRust");
    let root_channel = ChannelId(1);

    let client = Client::new()?;
    client.enable_auto_reconnect(ReconnectConfig::default());
    client.set_login_params(LoginParams::new(
        &nickname,
        &username,
        &password,
        &client_name,
    ));
    client.connect_remember(&host, tcp, udp, false)?;

    loop {
        if let Some((event, _)) = client.poll(100)
            && matches!(event, Event::MySelfLoggedIn)
        {
            client.join_channel(root_channel, "");
        }
    }
}
