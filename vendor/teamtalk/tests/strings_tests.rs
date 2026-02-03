use teamtalk::utils::strings::{ToTT, copy_to_string, from_tt, to_cow, to_string, tt_buf};

#[test]
fn tt_buf_zeroed() {
    let buf = tt_buf::<8>();
    assert!(buf.iter().all(|&c| c == 0));
}

#[test]
fn from_tt_null_is_empty() {
    let text = unsafe { from_tt(std::ptr::null()) };
    assert!(text.is_empty());
}

#[test]
fn to_cow_matches_to_string() {
    let input = "teamtalk";
    let tt = input.tt();
    let owned = to_string(&tt);
    let cow = to_cow(&tt);
    assert_eq!(cow.as_ref(), owned);
}

#[test]
fn copy_to_string_reuses_buffer() {
    let input = "reuse";
    let tt = input.tt();
    let mut out = String::from("placeholder");
    copy_to_string(&tt, &mut out);
    assert_eq!(out, input);
}

#[test]
fn to_tt_has_null_terminator() {
    let tt = "hello".tt();
    assert_eq!(tt.last().copied().unwrap_or(1), 0);
}

#[test]
fn to_string_without_null_uses_full_len() {
    let mut tt = "abc".tt();
    if let Some(last) = tt.last_mut() {
        *last = 1;
    }
    let text = to_string(&tt);
    assert_eq!(text, "abc\u{1}");
}
