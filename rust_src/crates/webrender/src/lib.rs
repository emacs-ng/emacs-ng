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
mod image;
mod texture;
mod util;
mod wrterm;

mod platform {
    #[cfg(macos_platform)]
    pub mod macos;
}

#[cfg(not(feature = "pselect"))]
mod future;
pub mod select {
    #[cfg(feature = "pselect")]
    pub use crate::select::plain::*;
    #[cfg(not(feature = "pselect"))]
    pub use crate::select::tokio::*;

    #[cfg(feature = "pselect")]
    pub mod plain;
    #[cfg(not(feature = "pselect"))]
    pub mod tokio;
}

pub mod gl {
    pub mod context;

    pub mod context_impl {
        #[cfg(feature = "glutin")]
        pub use crate::gl::context_impl::glutin::*;
        #[cfg(feature = "surfman")]
        pub use crate::gl::context_impl::surfman::*;

        #[cfg(feature = "glutin")]
        pub mod glutin;
        #[cfg(feature = "surfman")]
        pub mod surfman;
    }
}

#[cfg(macos_platform)]
pub use crate::platform::macos;

pub use crate::wrterm::{tip_frame, wr_display_list};

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
