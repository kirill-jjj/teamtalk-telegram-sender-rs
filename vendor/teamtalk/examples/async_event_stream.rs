#[cfg(feature = "async")]
use futures::StreamExt;
#[cfg(feature = "async")]
use teamtalk::{Client, Event};

#[cfg(feature = "async")]
fn main() -> teamtalk::Result<()> {
    // Convert the polling client into an async stream of events.
    let client = Client::new()?;
    let mut stream = client.into_async();

    futures::executor::block_on(async {
        while let Some((event, _msg)) = stream.next().await {
            if matches!(event, Event::ConnectionLost | Event::ConnectFailed) {
                break;
            }
        }
    });

    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    // This example requires the "async" feature.
    eprintln!("Enable the async feature: cargo run --example async_event_stream --features async");
}
