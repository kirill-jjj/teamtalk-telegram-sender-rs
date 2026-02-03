#[cfg(feature = "plugins")]
use teamtalk::extensions::plugins::PluginManager;

#[cfg(feature = "plugins")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = PluginManager::new();
    manager.load("sample", "plugins/sample_plugin.dll")?;
    Ok(())
}

#[cfg(not(feature = "plugins"))]
fn main() {
    eprintln!("Enable plugins feature: cargo run --example plugin_loader --features plugins");
}
