#![feature(concat_idents)]

extern crate git2;
extern crate lazy_static;
extern crate libc;

#[macro_use]
extern crate emacs;
extern crate lisp_macros;

#[macro_use]
macro_rules! export_lisp_fns {
    ($($(#[$($meta:meta),*])* $f:ident),+) => {
	pub fn rust_init_syms() {
	    #[allow(unused_unsafe)] // just in case the block is empty
	    unsafe {
		$(
		    $(#[$($meta),*])* emacs::bindings::defsubr(
			concat_idents!(S, $f).as_ptr() as *mut emacs::bindings::Aligned_Lisp_Subr
		    );
		)+
	    }
	}
    }
}

mod repository;

#[cfg(not(test))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/out/c_exports.rs"));
