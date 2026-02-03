use teamtalk::{Client, ClientHooks, Event, ReconnectConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;
    client.enable_auto_reconnect(ReconnectConfig::default());

    let hooks = ClientHooks::default()
        .on_before_reconnect(|_, _| println!("before reconnect"))
        .on_reconnecting(|_, _| println!("reconnecting"))
        .on_after_reconnect(|_, _| println!("after reconnect"))
        .on_reconnect_failed(|_, _| println!("reconnect failed"));
    client.set_hooks(hooks);

    loop {
        if let Some((event, _)) = client.poll(100)
            && matches!(event, Event::ConnectionLost | Event::ConnectFailed)
        {
            break;
        }
    }

    Ok(())
}
