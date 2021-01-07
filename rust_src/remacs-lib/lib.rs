#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![cfg_attr(feature = "strict", deny(warnings))]

extern crate errno;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate lisp_util;
extern crate rand;
extern crate regex;
extern crate time as time_crate;

mod docfile;
mod files;
mod math;

pub use crate::{
    // Used by make-docfile
    docfile::scan_rust_file,
    // Used for creating temporary files in emacs
    files::rust_make_temp,

    math::{rust_count_one_bits, rust_count_trailing_zero_bits},
};

#[cfg(all(not(test), target_os = "windows"))]
#[no_mangle]
pub unsafe extern "C" fn __chkstk() {}
