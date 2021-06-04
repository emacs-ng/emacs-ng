#![feature(concat_idents)]

extern crate emacs;
#[macro_use]
extern crate lisp_util;

mod ng_module;

#[cfg(not(test))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/out/c_exports.rs"));
