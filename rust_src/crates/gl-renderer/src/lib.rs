#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate emacs;
extern crate lisp_macros;
#[macro_use]
extern crate lisp_util;

pub mod display_info;
pub mod frame;

mod cursor;

#[cfg(have_window_system)]
pub mod window_system {

    #[cfg(window_system_pgtk)]
    mod pgtk {
        pub mod frame;
    }

    #[cfg(window_system_winit)]
    mod winit {
        pub mod frame;
    }
}

pub mod gl_renderer;
pub mod image;
mod image_cache;
pub mod output;

mod fringe;
mod glyph;
mod texture;
mod util;

pub mod gl {
    pub mod context;

    pub mod context_impl {
        #[cfg(use_glutin)]
        pub use crate::gl::context_impl::glutin::*;
        #[cfg(use_gtk3)]
        pub use crate::gl::context_impl::gtk3::*;
        #[cfg(use_surfman)]
        pub use crate::gl::context_impl::surfman::*;

        #[cfg(use_glutin)]
        pub mod glutin;
        #[cfg(use_gtk3)]
        pub mod gtk3;
        #[cfg(use_surfman)]
        pub mod surfman;
    }
}

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
