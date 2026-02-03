//! Global hotkey management.
#[cfg(windows)]
use super::Client;
#[cfg(windows)]
use crate::types::TT_STRLEN;
#[cfg(windows)]
use crate::utils::strings::tt_buf;
#[cfg(windows)]
use teamtalk_sys as ffi;

#[cfg(windows)]
impl Client {
    /// Registers a global hotkey.
    pub fn register_hotkey(&self, id: i32, vk_codes: &[i32]) -> bool {
        unsafe {
            ffi::api().TT_HotKey_Register(self.ptr.0, id, vk_codes.as_ptr(), vk_codes.len() as i32)
                == 1
        }
    }

    /// Unregisters a global hotkey.
    pub fn unregister_hotkey(&self, id: i32) -> bool {
        unsafe { ffi::api().TT_HotKey_Unregister(self.ptr.0, id) == 1 }
    }

    /// Checks if a hotkey is active.
    pub fn is_hotkey_active(&self, id: i32) -> bool {
        unsafe { ffi::api().TT_HotKey_IsActive(self.ptr.0, id) == 1 }
    }

    /// Installs a hotkey test hook (Windows only).
    ///
    /// # Safety
    /// - `hwnd` must be a valid window handle.
    /// - `msg` must be a valid message ID routed to `hwnd`.
    /// - The window's message loop must remain alive while the hook is installed.
    pub unsafe fn install_hotkey_test_hook(&self, hwnd: ffi::HWND, msg: u32) -> bool {
        unsafe { ffi::api().TT_HotKey_InstallTestHook(self.ptr.0, hwnd, msg) == 1 }
    }

    /// Removes the hotkey test hook.
    pub fn remove_hotkey_test_hook(&self) -> bool {
        unsafe { ffi::api().TT_HotKey_RemoveTestHook(self.ptr.0) == 1 }
    }

    /// Returns the string representation of a key.
    pub fn get_key_string(&self, vk_code: i32) -> String {
        let mut buf = tt_buf::<TT_STRLEN>();
        unsafe {
            if ffi::api().TT_HotKey_GetKeyString(self.ptr.0, vk_code, buf.as_mut_ptr()) == 1 {
                crate::utils::strings::to_string(&buf)
            } else {
                String::new()
            }
        }
    }
}
