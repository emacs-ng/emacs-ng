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

use libc::{c_char, c_uchar, ptrdiff_t};

use std::{fmt, slice};

use crate::{
    bindings::{encode_string_utf_8, Lisp_String, Lisp_Type},
    globals::{Qnil, Qstringp, Qt},
    lisp::{ExternalPtr, LispObject},
    obarray::LispObarrayRef,
    symbol::LispSymbolRef,
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

    // Same as the SCHARS function
    /// Return the string's length in characters.  Differs from
    /// `len_bytes` for multibyte strings.
    pub fn len_chars(self) -> ptrdiff_t {
        let s = unsafe { self.u.s };
        s.size
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.u.s.data as *const u8, self.len_bytes() as usize) }
    }

    pub fn to_utf8(self) -> String {
        let tagged = LispObject::tag_ptr(self, Lisp_Type::Lisp_String);
        let encoded = unsafe { encode_string_utf_8(tagged, Qnil, false, Qt, Qt) };
        let encoded_string: LispStringRef = encoded.into();
        String::from_utf8_lossy(encoded_string.as_slice()).into_owned()
    }

    pub fn const_data_ptr(self) -> *const c_uchar {
        let s = unsafe { self.u.s };
        s.data as *const c_uchar
    }

    pub fn const_sdata_ptr(self) -> *const c_char {
        let s = unsafe { self.u.s };
        s.data as *const c_char
    }
}

impl LispObject {
    pub fn is_string(self) -> bool {
        self.get_type() == Lisp_Type::Lisp_String
    }

    pub fn force_string(self) -> LispStringRef {
        self.to_string_unchecked()
    }

    pub fn as_string(self) -> Option<LispStringRef> {
        self.into()
    }

    pub fn to_string_unchecked(self) -> LispStringRef {
        LispStringRef::new(self.get_untaggedptr() as *mut Lisp_String)
    }

    // We can excuse not using an option here because extracting the value checks the type
    // TODO: this is false with the enum model, change this
    pub fn as_symbol_or_string(self) -> LispSymbolOrString {
        self.into()
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
            Some(o.force_string())
        } else {
            None
        }
    }
}

impl From<LispStringRef> for LispObject {
    fn from(s: LispStringRef) -> Self {
        Self::tag_ptr(s, Lisp_Type::Lisp_String)
    }
}

impl fmt::Display for LispStringRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice =
            unsafe { slice::from_raw_parts(self.const_data_ptr(), self.len_bytes() as usize) };
        write!(f, "{}", String::from_utf8_lossy(slice).into_owned())
    }
}

impl fmt::Debug for LispStringRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum LispSymbolOrString {
    String(LispStringRef),
    Symbol(LispSymbolRef),
}

impl LispSymbolOrString {
    pub fn is_string(self) -> bool {
        match self {
            LispSymbolOrString::String(_) => true,
            _ => false,
        }
    }

    pub fn is_symbol(self) -> bool {
        match self {
            LispSymbolOrString::Symbol(_) => true,
            _ => false,
        }
    }
}

impl From<LispSymbolOrString> for LispObject {
    fn from(s: LispSymbolOrString) -> Self {
        match s {
            LispSymbolOrString::String(s) => s.into(),
            LispSymbolOrString::Symbol(sym) => sym.into(),
        }
    }
}

impl From<LispSymbolOrString> for LispStringRef {
    fn from(s: LispSymbolOrString) -> Self {
        match s {
            LispSymbolOrString::String(s) => s,
            LispSymbolOrString::Symbol(sym) => sym.symbol_name().into(),
        }
    }
}

impl From<LispStringRef> for LispSymbolOrString {
    fn from(s: LispStringRef) -> Self {
        Self::String(s)
    }
}

impl From<LispSymbolOrString> for LispSymbolRef {
    fn from(s: LispSymbolOrString) -> Self {
        match s {
            LispSymbolOrString::String(s) => LispObarrayRef::global().intern(s).into(),
            LispSymbolOrString::Symbol(sym) => sym,
        }
    }
}

impl From<LispSymbolRef> for LispSymbolOrString {
    fn from(s: LispSymbolRef) -> Self {
        Self::Symbol(s)
    }
}

impl From<LispObject> for LispSymbolOrString {
    fn from(o: LispObject) -> Self {
        if let Some(s) = o.as_string() {
            Self::String(s)
        } else if let Some(sym) = o.as_symbol() {
            Self::Symbol(sym)
        } else {
            wrong_type!(Qstringp, o)
        }
    }
}

impl PartialEq<LispObject> for LispSymbolOrString {
    fn eq(&self, other: &LispObject) -> bool {
        (*other).eq(*self)
    }
}
