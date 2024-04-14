#![allow(clippy::cognitive_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
// we need this to be able to inclde FieldOffsets in C structs
#![allow(improper_ctypes)]
// we have a bunch of unused code during testing at the moment, somehow
#![cfg_attr(test, allow(unused))]
#![cfg_attr(feature = "strict", deny(warnings))]
#![feature(concat_idents)]
#![feature(never_type)]
#![feature(stmt_expr_attributes)]
#![feature(async_closure)]
#![feature(lazy_cell)]

#[cfg(all(glutin, surfman, winit))]
compile_error!("You cannot specify both `glutin` and `surfman` features for winit window system");
#[cfg(all(not(glutin), not(surfman), winit))]
compile_error!("One of `glutin` and `surfman` features is required for winit window system");

#[rustfmt::skip]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
#[rustfmt::skip]
pub mod definitions {
    include!(concat!(env!("OUT_DIR"), "/definitions.rs"));
}
#[rustfmt::skip]
pub mod globals {
    include!(concat!(env!("OUT_DIR"), "/globals.rs"));
}

#[macro_use]
pub mod sys;
#[macro_use]
pub mod eval_macros;
#[macro_use]
pub mod vector_macros;
pub mod lisp;

pub mod buffer;
#[cfg(use_webrender)]
pub mod color;
#[cfg(have_window_system)]
pub mod composite;
#[cfg(have_window_system)]
pub mod display_info;
pub mod display_traits;
pub mod eval;
pub mod font;
pub mod frame;
pub mod keyboard;
pub mod list;
pub mod multibyte;
pub mod number;
pub mod obarray;
#[cfg(have_window_system)]
pub mod output;
pub mod process;
pub mod string;
pub mod symbol;
pub mod terminal;
pub mod thread;
pub mod vector;
pub mod window;
#[cfg(all(any(have_winit, have_pgtk), use_webrender))]
mod window_system;
pub mod xdisp;
#[cfg(all(any(have_winit, have_pgtk), use_webrender))]
pub use window_system::*;
#[cfg(any(glutin, surfman, gtk3))]
pub mod gfx {
    pub mod context;

    pub mod context_impl {
        #[cfg(glutin)]
        pub use crate::gfx::context_impl::glutin::*;
        #[cfg(gtk3)]
        pub use crate::gfx::context_impl::gtk3::*;
        #[cfg(surfman)]
        pub use crate::gfx::context_impl::surfman::*;

        #[cfg(glutin)]
        pub mod glutin;
        #[cfg(gtk3)]
        pub mod gtk3;
        #[cfg(surfman)]
        pub mod surfman;
    }
}
