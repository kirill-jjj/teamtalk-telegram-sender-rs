#[cfg(feature = "scripts")]
use teamtalk::Client;
#[cfg(feature = "scripts")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;
    client.enable_scripts();
    client.scripts_mut(|scripts| {
        let _ = scripts.load_script("events", "crates/teamtalk/examples/scripts/events.lua");
    });
    Ok(())
}

#[cfg(not(feature = "scripts"))]
fn main() {
    eprintln!("Enable scripts feature: cargo run --example lua_script_events --features scripts");
}
