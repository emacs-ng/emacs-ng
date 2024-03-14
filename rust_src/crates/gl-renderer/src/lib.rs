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

pub mod gl_renderer;
pub mod image;
mod image_cache;
pub mod output;

mod fringe;
mod glyph;
mod texture;
mod util;

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));