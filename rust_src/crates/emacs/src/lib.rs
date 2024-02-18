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

#[cfg(feature = "window-system")]
pub mod color;
#[cfg(feature = "window-system")]
pub mod display_info;
pub mod eval;
pub mod font;
pub mod frame;
#[cfg(feature = "window-system")]
pub mod glyph;
pub mod keyboard;
#[cfg(feature = "window-system")]
pub mod lglyph;
pub mod list;
pub mod multibyte;
pub mod number;
pub mod obarray;
#[cfg(feature = "window-system")]
pub mod output;
pub mod process;
pub mod string;
pub mod symbol;
pub mod terminal;
pub mod vector;
pub mod window;
#[cfg(feature = "window-system")]
mod window_system;
pub mod xdisp;
#[cfg(feature = "window-system")]
pub use window_system::*;
