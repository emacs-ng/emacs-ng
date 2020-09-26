//! Beginnings of a Emacs-encoded string handling library.
//!
//! Emacs Lisp strings (and by extension, most strings handled by the
//! Emacs C API) are encoded in one of two ways:
//!
//! * "unibyte" strings are just sequences of 8-bit bytes that don't
//!   carry encoding information.  Their interpretation is governed
//!   by runtime settings (`set-language-environment').
//!
//! * "multibyte" strings are sequences of characters from an extended
//!   set of character codes, encoded in a fashion similar to UTF-8.
//!
//! The uniqueness of the Multibyte encoding is due to these features:
//!
//! * Codepoints up to 0x10FFFF coincide with Unicode.  However, the
//!   maximum codepoint is 0x3FFFFF.  The additional codepoints are
//!   used for "characters not unified with Unicode" and for 8-bit
//!   bytes, see below.
//!
//! * "Raw 8-bit" bytes, e.g. used when opening a file which is not
//!   properly encoded in a single encoding, are supported.
//!
//!   Raw 8-bit bytes are represented by codepoints 0x3FFF80 to
//!   0x3FFFFF.  However, in the UTF-8 like encoding, where they
//!   should be represented by a 5-byte sequence starting with 0xF8,
//!   they are instead represented by a 2-byte sequence starting with
//!   0xC0 or 0xC1.  These 2-byte sequences are disallowed in UTF-8,
//!   because they would form a duplicate encoding for the the 1-byte
//!   ASCII range.
//!
//! Due to these specialties, we cannot treat Emacs strings as Rust
//! `&str`, and this module regrettably contains adapted copies of
//! stretches of `std::str` functions.

use std::ffi::CString;

use libc::ptrdiff_t;


use crate::{
    lisp::{ExternalPtr, LispObject},
    remacs_sys::{Lisp_String, make_string} ,
};

pub type LispStringRef = ExternalPtr<Lisp_String>;

// cannot use `char`, it takes values out of its range
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Codepoint(u32);

impl From<u8> for Codepoint {
    fn from(u: u8) -> Self {
        Self(u32::from(u))
    }
}

impl From<u16> for Codepoint {
    fn from(u: u16) -> Self {
        Self(u32::from(u))
    }
}

impl From<u32> for Codepoint {
    fn from(u: u32) -> Self {
        Self(u)
    }
}

impl From<char> for Codepoint {
    fn from(c: char) -> Self {
        Self(u32::from(c))
    }
}

impl From<Codepoint> for u32 {
    fn from(c: Codepoint) -> Self {
        c.0
    }
}

impl From<Codepoint> for i64 {
    fn from(c: Codepoint) -> Self {
        c.0.into()
    }
}

impl From<Codepoint> for u64 {
    fn from(c: Codepoint) -> Self {
        c.0.into()
    }
}

/// Copies a Rust str into a new Lisp string
impl<'a> From<&'a str> for LispObject {
    fn from(s: &str) -> Self {
        let s = s.as_ptr() as *mut libc::c_char;
        unsafe { make_string(s, libc::strlen(s) as ptrdiff_t) }
    }
}
