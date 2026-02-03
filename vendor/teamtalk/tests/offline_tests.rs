#![cfg(feature = "offline")]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use teamtalk::loader::find_or_download_dll;

fn cwd_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("teamtalk_test_{}_{}", std::process::id(), nanos))
}

fn with_temp_cwd<F, R>(f: F) -> R
where
    F: FnOnce(&Path) -> R + std::panic::UnwindSafe,
{
    let _lock = cwd_lock();
    let dir = temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let old = env::current_dir().unwrap();
    env::set_current_dir(&dir).unwrap();
    let result = std::panic::catch_unwind(|| f(&dir));
    env::set_current_dir(old).unwrap();
    let _ = fs::remove_dir_all(&dir);
    match result {
        Ok(value) => value,
        Err(err) => std::panic::resume_unwind(err),
    }
}

fn dll_name() -> &'static str {
    if cfg!(windows) {
        "TeamTalk5.dll"
    } else {
        "libTeamTalk5.so"
    }
}

#[test]
fn offline_missing_dll_returns_error() {
    with_temp_cwd(|_| {
        let result = find_or_download_dll();
        assert!(result.is_err());
    });
}

#[test]
fn offline_existing_dll_returns_path() {
    with_temp_cwd(|dir| {
        let sdk_dir = dir.join("TEAMTALK_DLL");
        fs::create_dir_all(&sdk_dir).unwrap();
        let dll_path = sdk_dir.join(dll_name());
        fs::write(&dll_path, vec![0u8; 2048]).unwrap();
        let result = find_or_download_dll().unwrap();
        let abs = env::current_dir().unwrap().join(result);
        assert_eq!(abs, dll_path);
    });
}
