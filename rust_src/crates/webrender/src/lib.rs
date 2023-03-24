#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![allow(non_upper_case_globals)]
#![feature(once_cell)]

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

mod cursor;
#[cfg(window_system_winit)]
pub mod event;
#[cfg(window_system_winit)]
pub mod event_loop;
#[cfg(use_tokio_select)]
pub mod future;
#[cfg(window_system_winit)]
pub mod input;
#[cfg(window_system_winit)]
pub mod select {
    #[cfg(use_pselect)]
    pub use super::select::plain::*;
    #[cfg(use_tokio_select)]
    pub use super::select::tokio::*;

    #[cfg(use_pselect)]
    pub mod plain;
    #[cfg(use_tokio_select)]
    pub mod tokio;
}

#[cfg(have_window_system)]
pub mod window_system {

    #[cfg(window_system_pgtk)]
    pub use crate::window_system::pgtk::*;
    #[cfg(window_system_winit)]
    pub use crate::window_system::winit::*;
    #[cfg(window_system_winit)]
    pub use crate::window_system::winit_impl::*;

    // frame
    pub mod frame {
        #[cfg(window_system_pgtk)]
        pub use crate::window_system::pgtk::frame::*;
        #[cfg(window_system_winit)]
        pub use crate::window_system::winit::frame::*;
    }

    pub mod display_info {
        #[cfg(window_system_pgtk)]
        pub use crate::window_system::pgtk::display_info::*;
        #[cfg(window_system_winit)]
        pub use crate::window_system::winit::display_info::*;
    }

    pub mod output {
        #[cfg(window_system_pgtk)]
        pub use crate::window_system::pgtk::output::*;
        #[cfg(window_system_winit)]
        pub use crate::window_system::winit::output::*;
    }

    #[cfg(window_system_pgtk)]
    mod pgtk {
        pub mod display_info;
        pub mod frame;
        pub mod output;
    }

    #[cfg(window_system_winit)]
    mod winit {
        pub mod clipboard;
        pub mod cursor;
        pub mod display_info;
        pub mod frame;
        pub mod output;
        pub mod term;

        pub mod api {
            #[cfg(use_tao)]
            pub use tao::*;
            #[cfg(use_winit)]
            pub use winit::*;
        }
    }

    #[cfg(window_system_winit)]
    mod winit_impl {
        // macro for building key_name c string
        macro_rules! kn {
            ($e:expr) => {
                concat!($e, '\0').as_ptr() as *const libc::c_char
            };
        }

        #[cfg(use_tao)]
        pub use crate::window_system::winit_impl::tao::*;
        #[cfg(use_winit)]
        pub use crate::window_system::winit_impl::winit::*;

        #[cfg(use_tao)]
        pub mod tao;
        #[cfg(use_winit)]
        pub mod winit;
    }
}

pub mod image;
mod image_cache;
pub mod output;
pub mod term;

mod draw_canvas;
mod font_db;
mod fringe;
mod renderer {
    pub use crate::draw_canvas::*;
}
mod texture;
mod util;
#[cfg(window_system_winit)]
mod wrterm;

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

mod platform {
    #[cfg(all(window_system_winit, macos_platform))]
    pub mod macos;
}
#[cfg(all(window_system_winit, macos_platform))]
pub use crate::platform::macos;

pub use crate::color::*;
pub use crate::font::*;
pub use crate::term::*;
#[cfg(window_system_winit)]
pub use crate::window_system::term::*;
#[cfg(window_system_winit)]
pub use crate::wrterm::*;

#[cfg(not(test))]
#[cfg(window_system_winit)]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
#[no_mangle]
#[cfg(not(window_system_winit))]
pub extern "C" fn wrterm_init_syms() {
    term::rust_init_syms();
}
