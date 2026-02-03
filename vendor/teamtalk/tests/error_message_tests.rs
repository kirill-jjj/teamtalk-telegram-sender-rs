use teamtalk::client::ffi;
use teamtalk::types::ErrorMessage;
use teamtalk::utils::strings::ToTT;

fn copy_tt(src: &str, dst: &mut [ffi::TTCHAR]) {
    let tt = src.tt();
    let len = tt.len().min(dst.len());
    dst[..len].copy_from_slice(&tt[..len]);
}

#[test]
fn error_message_from_ffi() {
    let mut raw = ffi::ClientErrorMsg {
        nErrorNo: -7,
        ..Default::default()
    };
    copy_tt("oops", &mut raw.szErrorMsg);
    let err = ErrorMessage::from(raw);
    assert_eq!(err.code, -7);
    assert_eq!(err.message, "oops");
}
