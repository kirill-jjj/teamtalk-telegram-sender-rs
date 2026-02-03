#[cfg(feature = "scripts")]
use teamtalk::extensions::scripts::ScriptManager;

#[cfg(feature = "scripts")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = ScriptManager::new();
    manager.load_script("commands", "crates/teamtalk/examples/scripts/commands.lua")?;
    let handled = manager.call_command("start", &["channel".to_string()])?;
    println!("handled: {}", handled);
    Ok(())
}

#[cfg(not(feature = "scripts"))]
fn main() {
    eprintln!("Enable scripts feature: cargo run --example lua_script_commands --features scripts");
}
