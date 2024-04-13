#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![feature(lazy_cell)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate emacs_sys;
extern crate lisp_macros;
#[macro_use]
extern crate lisp_util;

// macro for building key_name c string
macro_rules! kn {
    ($e:expr) => {
        concat!($e, '\0').as_ptr() as *const libc::c_char
    };
}

pub mod cursor;
pub mod event;
pub mod frame;
pub mod input;
pub mod term;

mod fns;

mod platform {
    #[cfg(all(macos_platform))]
    pub mod macos;
}
#[cfg(all(macos_platform))]
pub use crate::platform::macos;

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
