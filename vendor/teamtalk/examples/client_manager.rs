use std::env;
use teamtalk::{Client, ClientManager, Event, LoginParams, ReconnectConfig};

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
    let nickname = env_or("TT_NICK", "ManagerBot");
    let username = env_or("TT_USER", "");
    let password = env_or("TT_PASS", "");
    let client_name = env_or("TT_CLIENT", "");

    let mut manager = ClientManager::new();

    let client_a = Client::new()?.with_label("bot-a");
    client_a.enable_auto_reconnect(ReconnectConfig::default());
    client_a.set_login_params(LoginParams::new(
        &nickname,
        &username,
        &password,
        &client_name,
    ));
    let _ = client_a.connect_remember(&host, tcp, udp, false);

    let client_b = Client::new()?.with_label("bot-b");
    client_b.enable_auto_reconnect(ReconnectConfig::default());
    client_b.set_login_params(LoginParams::new(
        &nickname,
        &username,
        &password,
        &client_name,
    ));
    let _ = client_b.connect_remember(&host, tcp, udp, false);

    manager.add_client(client_a);
    manager.add_client(client_b);

    loop {
        manager.run_once();
        while let Ok(evt) = manager.events().try_recv() {
            if matches!(evt.event, Event::ConnectionLost | Event::ConnectFailed) {
                println!("{:?} {:?} {:?}", evt.client_id, evt.label, evt.event);
            }
        }
    }
}
