use teamtalk::client::ffi;
use teamtalk::types::{ChannelId, FileTransfer, FileTransferStatus, TransferId};

#[test]
fn file_transfer_status_mapping() {
    assert!(matches!(
        FileTransferStatus::from(ffi::FileTransferStatus::FILETRANSFER_CLOSED),
        FileTransferStatus::Closed
    ));
    assert!(matches!(
        FileTransferStatus::from(ffi::FileTransferStatus::FILETRANSFER_ERROR),
        FileTransferStatus::Error
    ));
    assert!(matches!(
        FileTransferStatus::from(ffi::FileTransferStatus::FILETRANSFER_ACTIVE),
        FileTransferStatus::Active
    ));
    assert!(matches!(
        FileTransferStatus::from(ffi::FileTransferStatus::FILETRANSFER_FINISHED),
        FileTransferStatus::Finished
    ));
}

#[test]
fn file_transfer_progress_fraction() {
    let t = FileTransfer {
        status: FileTransferStatus::Active,
        id: TransferId(1),
        channel_id: ChannelId(2),
        local_path: String::new(),
        remote_name: String::new(),
        size: 200,
        transferred: 50,
        inbound: false,
    };
    assert_eq!(t.progress(), 0.25);
}
