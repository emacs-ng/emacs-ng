//! obarray code

use crate::{remacs_sys::intern_1, symbols::LispSymbolRef};

/// Intern (e.g. create a symbol from) a string.
pub fn intern<T: AsRef<str>>(string: T) -> LispSymbolRef {
    let s = string.as_ref();
    unsafe {
        intern_1(
            s.as_ptr() as *const libc::c_char,
            s.len() as libc::ptrdiff_t,
        )
    }
    .into()
}

// include!(concat!(env!("OUT_DIR"), "/obarray_exports.rs"));
