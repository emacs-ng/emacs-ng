#![feature(concat_idents)]
#![feature(lazy_cell)]

#[macro_use]
extern crate emacs_sys;
extern crate lisp_macros;
#[macro_use]
extern crate lisp_util;

mod repository;

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
