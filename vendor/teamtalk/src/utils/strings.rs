//! Helpers for TeamTalk string conversions.
use std::borrow::Cow;
use teamtalk_sys as ffi;

#[cfg(not(windows))]
fn ttchar_bytes(slice: &[ffi::TTCHAR]) -> &[u8] {
    // Safety: on non-Windows TTCHAR is `char` (1 byte). We only reinterpret the same length.
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len()) }
}

/// Creates a zeroed TeamTalk string buffer.
pub fn tt_buf<const N: usize>() -> [ffi::TTCHAR; N] {
    [0 as ffi::TTCHAR; N]
}

/// Converts Rust strings into TeamTalk UTF-16 or UTF-8 buffers.
pub trait ToTT {
    fn tt(&self) -> Vec<ffi::TTCHAR>;
}

impl ToTT for str {
    fn tt(&self) -> Vec<ffi::TTCHAR> {
        #[cfg(windows)]
        {
            let mut v: Vec<u16> = self.encode_utf16().collect();
            v.push(0);
            v
        }
        #[cfg(not(windows))]
        {
            let mut v: Vec<i8> = self.as_bytes().iter().map(|&b| b as i8).collect();
            v.push(0);
            v
        }
    }
}

impl ToTT for String {
    fn tt(&self) -> Vec<ffi::TTCHAR> {
        self.as_str().tt()
    }
}

/// Converts a raw TeamTalk string pointer into `String`.
///
/// # Safety
/// `ptr` must be a valid null-terminated TeamTalk string.
pub unsafe fn from_tt(ptr: *const ffi::TTCHAR) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    #[cfg(windows)]
    {
        String::from_utf16_lossy(slice)
    }
    #[cfg(not(windows))]
    {
        String::from_utf8_lossy(ttchar_bytes(slice)).into_owned()
    }
}

/// Converts a TeamTalk string buffer into `String`.
pub fn to_string(arr: &[ffi::TTCHAR]) -> String {
    let len = arr.iter().position(|&c| c == 0).unwrap_or(arr.len());
    #[cfg(windows)]
    {
        String::from_utf16_lossy(&arr[..len])
    }
    #[cfg(not(windows))]
    {
        String::from_utf8_lossy(ttchar_bytes(&arr[..len])).into_owned()
    }
}

/// Converts a TeamTalk string buffer into a `Cow<str>`.
pub fn to_cow(arr: &[ffi::TTCHAR]) -> Cow<'_, str> {
    let len = arr.iter().position(|&c| c == 0).unwrap_or(arr.len());
    #[cfg(windows)]
    {
        Cow::Owned(String::from_utf16_lossy(&arr[..len]))
    }
    #[cfg(not(windows))]
    {
        String::from_utf8_lossy(ttchar_bytes(&arr[..len]))
    }
}

/// Copies a TeamTalk string buffer into a reusable `String`.
pub fn copy_to_string(arr: &[ffi::TTCHAR], out: &mut String) {
    out.clear();
    let len = arr.iter().position(|&c| c == 0).unwrap_or(arr.len());
    #[cfg(windows)]
    {
        *out = String::from_utf16_lossy(&arr[..len]);
    }
    #[cfg(not(windows))]
    {
        out.push_str(&String::from_utf8_lossy(ttchar_bytes(&arr[..len])));
    }
}

#[cfg(test)]
mod tests {
    use super::{ToTT, copy_to_string, from_tt, to_cow, to_string, tt_buf};

    #[test]
    fn tt_buf_is_zeroed() {
        let buf = tt_buf::<4>();
        assert!(buf.iter().all(|&c| c == 0));
    }

    #[test]
    fn to_tt_roundtrip() {
        let input = "TeamTalk";
        let tt = input.tt();
        let output = unsafe { from_tt(tt.as_ptr()) };
        assert_eq!(output, input);
    }

    #[test]
    fn to_cow_matches_to_string() {
        let input = "abc";
        let tt = input.tt();
        assert_eq!(to_cow(&tt).as_ref(), to_string(&tt));
    }

    #[test]
    fn copy_to_string_clears_and_reuses() {
        let input = "hello";
        let tt = input.tt();
        let mut out = String::from("placeholder");
        copy_to_string(&tt, &mut out);
        assert_eq!(out, input);
    }
}
