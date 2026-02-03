//! Encryption context management.
use super::Client;
pub use crate::types::EncryptionContext;
use teamtalk_sys as ffi;

impl Client {
    /// Sets the encryption context for future connections.
    pub fn set_encryption_context(&self, context: &EncryptionContext) -> bool {
        unsafe { ffi::api().TT_SetEncryptionContext(self.ptr.0, &context.to_ffi()) == 1 }
    }
}
