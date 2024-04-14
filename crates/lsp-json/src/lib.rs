#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![feature(async_closure)]
#![feature(lazy_cell)]

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate emacs_sys;
#[macro_use]
extern crate lisp_util;

pub mod parsing;

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
