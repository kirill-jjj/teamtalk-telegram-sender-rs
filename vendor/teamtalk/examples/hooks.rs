use std::env;
use teamtalk::{Client, ClientHooks, Event};

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
    let nickname = env_or("TT_NICK", "HookBot");
    let username = env_or("TT_USER", "guest");
    let password = env_or("TT_PASS", "guest");
    let client_name = env_or("TT_CLIENT", "TeamTalkRust");

    let client = Client::new()?;
    let hooks = ClientHooks::default()
        .on_connect_success(|_| println!("connected"))
        .on_logged_in(|_| println!("logged in"))
        .on_text_message(|_, msg| println!("text: {}", msg.text))
        .on_event(|_, event, _| println!("event: {event:?}"));
    client.set_hooks(hooks);

    client.connect(&host, tcp, udp, false)?;

    loop {
        if let Some((event, _)) = client.poll(100) {
            match event {
                Event::ConnectSuccess => {
                    client.login(&nickname, &username, &password, &client_name);
                }
                Event::ConnectionLost | Event::ConnectFailed => break,
                _ => {}
            }
        }
    }

    Ok(())
}
