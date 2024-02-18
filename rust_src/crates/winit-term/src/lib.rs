#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate emacs;
extern crate lisp_macros;
#[macro_use]
extern crate lisp_util;

pub mod clipboard;
pub mod cursor;
pub mod event;
pub mod frame;
pub mod input;
pub mod output;
pub mod term;

pub mod api {
    #[cfg(use_tao)]
    pub use tao::*;
    #[cfg(use_winit)]
    pub use winit::*;
}

mod winit_impl {
    // macro for building key_name c string
    macro_rules! kn {
        ($e:expr) => {
            concat!($e, '\0').as_ptr() as *const libc::c_char
        };
    }

    #[cfg(use_tao)]
    pub use crate::winit_impl::tao::*;
    #[cfg(use_winit)]
    pub use crate::winit_impl::winit::*;

    #[cfg(use_tao)]
    pub mod tao;
    #[cfg(use_winit)]
    pub mod winit;
}

mod wrterm;

mod platform {
    #[cfg(all(macos_platform))]
    pub mod macos;
}
#[cfg(all(macos_platform))]
pub use crate::platform::macos;

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
