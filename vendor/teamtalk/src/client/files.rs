//! File transfer APIs.
use super::Client;
use crate::types::{ChannelId, FileId, RemoteFile, TransferId};
use crate::utils::ToTT;
use teamtalk_sys as ffi;

impl Client {
    /// Returns files available in a channel.
    pub fn get_channel_files(&self, channel_id: ChannelId) -> Vec<RemoteFile> {
        let mut count: i32 = 0;
        unsafe {
            ffi::api().TT_GetChannelFiles(
                self.ptr.0,
                channel_id.0,
                std::ptr::null_mut(),
                &mut count,
            );
            let mut files = vec![std::mem::zeroed::<ffi::RemoteFile>(); count as usize];
            if ffi::api().TT_GetChannelFiles(
                self.ptr.0,
                channel_id.0,
                files.as_mut_ptr(),
                &mut count,
            ) == 1
            {
                files.into_iter().map(RemoteFile::from).collect()
            } else {
                vec![]
            }
        }
    }

    /// Sends a local file to a channel.
    pub fn send_file(&self, channel_id: ChannelId, local_path: &str) -> i32 {
        unsafe { ffi::api().TT_DoSendFile(self.ptr.0, channel_id.0, local_path.tt().as_ptr()) }
    }

    /// Receives a remote file into a local directory.
    pub fn recv_file(&self, channel_id: ChannelId, remote_file_id: FileId, local_dir: &str) -> i32 {
        unsafe {
            ffi::api().TT_DoRecvFile(
                self.ptr.0,
                channel_id.0,
                remote_file_id.0,
                local_dir.tt().as_ptr(),
            )
        }
    }

    /// Deletes a remote file from a channel.
    pub fn delete_file(&self, channel_id: ChannelId, remote_file_id: FileId) -> i32 {
        unsafe { ffi::api().TT_DoDeleteFile(self.ptr.0, channel_id.0, remote_file_id.0) }
    }

    /// Returns file transfer info by transfer id.
    pub fn get_file_transfer_info(
        &self,
        transfer_id: TransferId,
    ) -> Option<crate::types::FileTransfer> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::FileTransfer>() };
        if unsafe { ffi::api().TT_GetFileTransferInfo(self.ptr.0, transfer_id.0, &mut raw) } == 1 {
            Some(crate::types::FileTransfer::from(raw))
        } else {
            None
        }
    }

    /// Cancels an in-progress file transfer.
    pub fn cancel_file_transfer(&self, transfer_id: TransferId) -> bool {
        unsafe { ffi::api().TT_CancelFileTransfer(self.ptr.0, transfer_id.0) == 1 }
    }
}
