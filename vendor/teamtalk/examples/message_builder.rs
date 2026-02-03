use teamtalk::Client;
use teamtalk::types::{ChannelId, MessageBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;
    let _ = MessageBuilder::new(ChannelId(1))
        .text("hello from builder")
        .send(&client);
    Ok(())
}
