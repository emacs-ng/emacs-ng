#![feature(concat_idents)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate emacs;
extern crate lisp_macros;
#[macro_use]
extern crate lisp_util;
extern crate colors;

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
mod event_loop;
mod font_db;
mod fringe;
mod future;
mod image;
mod texture;
mod util;
mod wrterm;

mod platform {
    #[cfg(target_os = "macos")]
    pub mod macos;
}

#[cfg(target_os = "macos")]
pub use crate::platform::macos;

pub use crate::wrterm::{tip_frame, wr_display_list};

#[cfg(not(test))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/out/c_exports.rs"));
