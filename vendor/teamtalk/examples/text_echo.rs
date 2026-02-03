use std::env;
use teamtalk::client::ffi;
use teamtalk::types::{ChannelId, MessageTarget};
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
    // Read connection settings.
    let host = env_or("TT_HOST", "127.0.0.1");
    let tcp = env_or_i32("TT_TCP", 10333);
    let udp = env_or_i32("TT_UDP", 10333);
    let nickname = env_or("TT_NICK", "EchoBot");
    let username = env_or("TT_USER", "guest");
    let password = env_or("TT_PASS", "guest");
    let client_name = env_or("TT_CLIENT", "TeamTalkRust");
    let root_channel = ChannelId(1);

    let client = Client::new()?;
    client.connect(&host, tcp, udp, false)?;

    // Echo channel messages back to the channel.
    loop {
        if let Some((event, msg)) = client.poll(100) {
            match event {
                Event::ConnectSuccess => {
                    client.login(&nickname, &username, &password, &client_name);
                }
                Event::MySelfLoggedIn => {
                    client.join_channel(root_channel, "");
                }
                Event::TextMessage => {
                    if let Some(text) = msg.text()
                        && text.msg_type == ffi::TextMsgType::MSGTYPE_CHANNEL
                        && text.channel_id == root_channel
                    {
                        let reply = format!("echo: {}", text.text);
                        client.send_text(MessageTarget::Channel(root_channel), &reply);
                    }
                }
                Event::ConnectionLost | Event::ConnectFailed => break,
                _ => {}
            }
        }
    }

    Ok(())
}
