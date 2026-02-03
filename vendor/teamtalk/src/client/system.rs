//! System-level helpers and licensing APIs.
use super::Client;
use crate::utils::ToTT;
use teamtalk_sys as ffi;

impl Client {
    /// Returns the TeamTalk SDK version string.
    pub fn get_version() -> String {
        unsafe {
            let ptr = ffi::api().TT_GetVersion();
            crate::utils::strings::from_tt(ptr)
        }
    }

    /// Sets license information for the SDK.
    pub fn set_license(&self, name: &str, key: &str) -> bool {
        unsafe { ffi::api().TT_SetLicenseInformation(name.tt().as_ptr(), key.tt().as_ptr()) == 1 }
    }

    #[cfg(windows)]
    /// Returns whether the Windows firewall is enabled.
    pub fn is_firewall_enabled(&self) -> bool {
        unsafe { ffi::api().TT_Firewall_IsEnabled() == 1 }
    }

    #[cfg(windows)]
    /// Returns whether a firewall exception exists for the executable.
    pub fn firewall_app_exception_exists(&self, exe_path: &str) -> bool {
        unsafe { ffi::api().TT_Firewall_AppExceptionExists(exe_path.tt().as_ptr()) == 1 }
    }

    #[cfg(windows)]
    /// Enables or disables the Windows firewall.
    pub fn enable_firewall(&self, enable: bool) -> bool {
        unsafe { ffi::api().TT_Firewall_Enable(if enable { 1 } else { 0 }) == 1 }
    }

    #[cfg(windows)]
    /// Adds a firewall exception.
    pub fn add_firewall_exception(&self, name: &str, exe_path: &str) -> bool {
        unsafe {
            ffi::api().TT_Firewall_AddAppException(name.tt().as_ptr(), exe_path.tt().as_ptr()) == 1
        }
    }

    #[cfg(windows)]
    /// Removes a firewall exception.
    pub fn remove_firewall_exception(&self, exe_path: &str) -> bool {
        unsafe { ffi::api().TT_Firewall_RemoveAppException(exe_path.tt().as_ptr()) == 1 }
    }
}
