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
#![feature(const_fn)]
#![feature(const_fn_union)]
#![feature(never_type)]
#![feature(stmt_expr_attributes)]
#![feature(untagged_unions)]
#![feature(maybe_uninit_extra)]
#![feature(async_closure)]

extern crate errno;
extern crate lazy_static;

extern crate core;
extern crate field_offset;
extern crate libc;

// Needed for linking.
#[macro_use]
extern crate lisp;
extern crate lisp_macros;
extern crate remacs_lib;

extern crate futures;
extern crate lsp_server;
#[macro_use]
extern crate serde_json;
extern crate crossbeam;
extern crate deno;
extern crate deno_core;
extern crate deno_runtime;
extern crate rusty_v8;
extern crate tokio;

#[macro_use]
macro_rules! export_lisp_fns {
    ($($(#[$($meta:meta),*])* $f:ident),+) => {
	pub fn rust_init_syms() {
	    #[allow(unused_unsafe)] // just in case the block is empty
	    unsafe {
		$(
		    $(#[$($meta),*])* lisp::remacs_sys::defsubr(
			concat_idents!(S, $f).as_ptr() as *mut lisp::remacs_sys::Aligned_Lisp_Subr
		    );
		)+
	    }
	}
    }
}

mod javascript;
mod ng_async;
mod parsing;

#[cfg(feature = "window-system-webrender")]
mod webrender_backend;
#[cfg(feature = "window-system-webrender")]
mod wrterm;

#[cfg(feature = "window-system-webrender")]
pub use crate::wrterm::{tip_frame, wr_display_list};

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
