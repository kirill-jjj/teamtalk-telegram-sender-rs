use teamtalk::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;
    client.enable_user_cache(true);
    client.enable_channel_cache(true);
    let _ = client.refresh_user_cache();
    let _ = client.refresh_channel_cache();
    let users = client.cached_users();
    let channels = client.cached_channels();
    println!("users: {}", users.len());
    println!("channels: {}", channels.len());
    Ok(())
}
