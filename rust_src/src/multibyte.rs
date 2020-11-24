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

use libc::{c_char, ptrdiff_t};

use std::slice;

use crate::{
    lisp::{ExternalPtr, LispObject},
    remacs_sys::{Lisp_String, Lisp_Type, Qstringp},
};

pub type LispStringRef = ExternalPtr<Lisp_String>;

// String support (LispType == 4)

impl LispStringRef {
    /// Return the string's len in bytes.
    pub fn len_bytes(self) -> ptrdiff_t {
        let s = unsafe { self.u.s };
        if s.size_byte < 0 {
            s.size
        } else {
            s.size_byte
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.u.s.data as *const u8, self.len_bytes() as usize) }
    }
}

impl LispObject {
    pub fn is_string(self) -> bool {
        self.get_type() == Lisp_Type::Lisp_String
    }

    pub fn as_string(self) -> Option<LispStringRef> {
        self.into()
    }

    pub fn to_string_unchecked(self) -> LispStringRef {
        LispStringRef::new(self.get_untaggedptr() as *mut Lisp_String)
    }
}

impl From<LispObject> for LispStringRef {
    fn from(o: LispObject) -> Self {
        o.as_string().unwrap_or_else(|| wrong_type!(Qstringp, o))
    }
}

impl From<LispObject> for Option<LispStringRef> {
    fn from(o: LispObject) -> Self {
        if o.is_string() {
            Some(o.to_string_unchecked())
        } else {
            None
        }
    }
}
