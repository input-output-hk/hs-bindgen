//! This module defines convenient traits to let user-defined function take as
//! argument or return type either `CString`, `&CStr`, `String` or `&str`

use crate::{FromReprC, FromReprRust};
use std::ffi::{c_char, CStr, CString};

impl FromReprRust<*const c_char> for CString {
    #[inline]
    fn from(ptr: *const c_char) -> Self {
        let r: &str = FromReprRust::from(ptr);
        CString::new(r).unwrap()
    }
}

impl FromReprRust<*const c_char> for &CStr {
    #[inline]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn from(ptr: *const c_char) -> Self {
        unsafe { CStr::from_ptr(ptr) }
    }
}

impl FromReprRust<*const c_char> for String {
    #[inline]
    fn from(ptr: *const c_char) -> Self {
        let r: &str = FromReprRust::from(ptr);
        r.to_string()
    }
}

impl FromReprRust<*const c_char> for &str {
    #[inline]
    fn from(ptr: *const c_char) -> Self {
        let r: &CStr = FromReprRust::from(ptr);
        r.to_str().unwrap()
    }
}

impl FromReprC<CString> for *const c_char {
    #[inline]
    fn from(s: CString) -> Self {
        let x = s.as_ptr();
        // FIXME: this pattern is somehow duplicated in `vec` module and should
        // rather live behind in a `AsPtr` trait, similar to the one defined by
        // https://crates.io/crates/ptrplus
        std::mem::forget(s);
        x
    }
}

impl FromReprC<String> for *const c_char {
    #[inline]
    fn from(s: String) -> Self {
        FromReprC::from(CString::new(s).unwrap())
    }
}

#[test]
fn _1() {
    let x = "hello"; // FIXME: use Arbitrary crate
    let y: &str = FromReprRust::from(FromReprC::from(x.to_string()));
    assert!(x == y);
}
