#[cfg(feature = "dispatch")]
use teamtalk::Client;
#[cfg(feature = "dispatch")]
use teamtalk::dispatch::{DispatchFlow, Dispatcher};

#[cfg(feature = "dispatch")]
fn main() -> teamtalk::Result<()> {
    // Route events through a dispatcher with handlers.
    let client = Client::new()?;
    let mut dispatcher = Dispatcher::new(client)
        .on_connect_success(|_| DispatchFlow::Continue)
        .on_connection_lost(|_| DispatchFlow::Stop)
        .on_text_message(|_| DispatchFlow::Continue);

    dispatcher.run();
    Ok(())
}

#[cfg(not(feature = "dispatch"))]
fn main() {
    // This example requires the "dispatch" feature.
    eprintln!(
        "Enable the dispatch feature: cargo run --example dispatch_handlers --features dispatch"
    );
}
