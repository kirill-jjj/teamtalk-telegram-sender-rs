#[cfg(all(feature = "scripts", feature = "mock"))]
mod scripts_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use teamtalk::events::Event;
    use teamtalk::extensions::scripts::ScriptManager;
    use teamtalk::mock::MockMessage;

    fn temp_script(name: &str, contents: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_nanos();
        path.push(format!("teamtalk_{name}_{nanos}.lua"));
        fs::write(&path, contents).expect("write script");
        path
    }

    #[test]
    fn register_command_and_unload_removes_command() {
        let path = temp_script(
            "register_command",
            r#"
            register_command("ping", function(args)
                return true
            end)
        "#,
        );
        let mut manager = ScriptManager::new();
        manager.load_script("test", &path).expect("load");
        let handled = manager.call_command("ping", &[]).expect("call_command");
        assert!(handled);
        manager.unload_script("test").expect("unload");
        let handled = manager
            .call_command("ping", &[])
            .expect("call_command after unload");
        assert!(!handled);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn timeout_interrupts_long_running_command() {
        let path = temp_script(
            "timeout",
            r#"
            register_command("spin", function(args)
                while true do end
            end)
        "#,
        );
        let mut manager = ScriptManager::new();
        manager.set_timeout(Duration::from_millis(1));
        manager.set_hook_instruction_count(1);
        manager.load_script("timeout", &path).expect("load");
        let err = manager
            .call_command("spin", &[])
            .expect_err("expected timeout");
        let msg = err.to_string();
        assert!(msg.contains("timeout"), "{msg}");
        manager.unload_script("timeout").expect("unload");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn errors_include_context() {
        let path = temp_script(
            "event_error",
            r#"
            function on_event(evt)
                error("boom")
            end
        "#,
        );
        let mut manager = ScriptManager::new();
        manager.load_script("event_error", &path).expect("load");
        let message = MockMessage::empty();
        let err = manager
            .handle_event(Event::None, &message)
            .expect_err("expected error");
        let text = err.to_string();
        assert!(text.contains("lua on_event error (None)"), "{text}");
        manager.unload_script("event_error").expect("unload");
        let _ = fs::remove_file(path);
    }
}
