#![feature(concat_idents)]

#[macro_use]
extern crate emacs;
extern crate lisp_macros;
#[macro_use]
extern crate lisp_util;

mod repository;

#[cfg(not(test))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/out/c_exports.rs"));
