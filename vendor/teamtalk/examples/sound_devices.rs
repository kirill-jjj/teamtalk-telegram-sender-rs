use teamtalk::{Client, Event};

fn main() -> teamtalk::Result<()> {
    // Print available sound devices after initialization.
    let client = Client::new()?;

    for device in client.get_sound_devices() {
        println!("{} ({})", device.name, device.id);
    }

    // Keep polling to avoid early exit in environments expecting an event loop.
    loop {
        if let Some((event, _msg)) = client.poll(100)
            && matches!(event, Event::ConnectionLost | Event::ConnectFailed)
        {
            break;
        }
    }

    Ok(())
}
