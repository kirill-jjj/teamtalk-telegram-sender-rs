use std::env;
use teamtalk::{Client, ClientRegistry};

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

    let registry = ClientRegistry::new();

    let client_a = Client::new()?.with_label("bot-a");
    let client_b = Client::new()?.with_label("bot-b");

    registry.register(&client_a);
    registry.register(&client_b);

    let _ = client_a.connect(&host, tcp, udp, false);
    let _ = client_b.connect(&host, tcp, udp, false);

    let mut tick: u64 = 0;
    loop {
        if let Some((event, _)) = client_a.poll(50) {
            registry.update_event(&client_a, event);
        }
        if let Some((event, _)) = client_b.poll(50) {
            registry.update_event(&client_b, event);
        }

        if tick.is_multiple_of(100) {
            for info in registry.list() {
                println!("{:?} {:?} {:?}", info.id, info.label, info.state);
            }
        }

        tick = tick.wrapping_add(1);

        if matches!(
            client_a.connection_state(),
            teamtalk::ConnectionState::Disconnected
        ) && matches!(
            client_b.connection_state(),
            teamtalk::ConnectionState::Disconnected
        ) {
            break;
        }
    }

    Ok(())
}
