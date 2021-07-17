#![feature(concat_idents)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate emacs;
extern crate lisp_macros;
#[macro_use]
extern crate lisp_util;

pub mod color;
pub mod display_info;
pub mod font;
pub mod frame;
pub mod input;
pub mod output;
pub mod term;

mod cursor;
mod draw_canvas;
mod event;
mod font_db;
mod fringe;
mod image;
mod texture;
mod util;

mod wrterm;

pub use crate::wrterm::{tip_frame, wr_display_list};

#[cfg(not(test))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/out/c_exports.rs"));
