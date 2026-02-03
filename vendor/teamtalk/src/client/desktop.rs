//! Desktop access and sharing APIs.
use super::Client;
use crate::types::UserId;
use teamtalk_sys as ffi;

impl Client {
    /// Closes a desktop window session.
    pub fn close_desktop_window(&self) -> bool {
        unsafe { ffi::api().TT_CloseDesktopWindow(self.ptr.0) == 1 }
    }

    /// Sends mouse cursor position to the desktop sharer.
    pub fn send_desktop_cursor_position(&self, x: u16, y: u16) -> bool {
        unsafe { ffi::api().TT_SendDesktopCursorPosition(self.ptr.0, x, y) == 1 }
    }

    /// Sends keyboard or mouse input to the desktop sharer.
    pub fn send_desktop_input(&self, user_id: UserId, input: &ffi::DesktopInput) -> bool {
        unsafe { ffi::api().TT_SendDesktopInput(self.ptr.0, user_id.0, input, 1) == 1 }
    }

    /// Acquires a desktop window update bitmap.
    pub fn acquire_user_desktop_window(&self, user_id: UserId) -> Option<*mut ffi::DesktopWindow> {
        unsafe {
            let ptr = ffi::api().TT_AcquireUserDesktopWindow(self.ptr.0, user_id.0);
            if ptr.is_null() { None } else { Some(ptr) }
        }
    }

    #[allow(clippy::missing_safety_doc)]
    /// Releases a previously acquired desktop window.
    ///
    /// # Safety
    /// `window` must be a valid pointer returned by `acquire_user_desktop_window`.
    pub unsafe fn release_user_desktop_window(&self, window: *mut ffi::DesktopWindow) -> bool {
        if window.is_null() {
            return false;
        }
        unsafe { ffi::api().TT_ReleaseUserDesktopWindow(self.ptr.0, window) == 1 }
    }
}
