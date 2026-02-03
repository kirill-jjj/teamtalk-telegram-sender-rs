use std::env;
use teamtalk::types::ChannelId;
use teamtalk::{Client, Event};

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
    let nickname = env_or("TT_NICK", "WaitBot");
    let username = env_or("TT_USER", "guest");
    let password = env_or("TT_PASS", "guest");
    let client_name = env_or("TT_CLIENT", "TeamTalkRust");
    let root_channel = ChannelId(1);

    let client = Client::new()?;
    client.connect(&host, tcp, udp, false)?;

    let _ = client.wait_for(Event::ConnectSuccess, 5_000);
    client.login(&nickname, &username, &password, &client_name);
    let _ = client.poll_until(5_000, |event, _| matches!(event, Event::MySelfLoggedIn));
    client.join_channel(root_channel, "");
    let _ = client.poll_until(5_000, |event, msg| {
        matches!(event, Event::UserJoined) && msg.user().is_some()
    });

    Ok(())
}
