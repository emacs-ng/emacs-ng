#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![feature(async_closure)]
#![feature(maybe_uninit_extra)]

#[macro_use]
extern crate serde_json;
extern crate futures;

#[macro_use]
extern crate emacs;
#[macro_use]
extern crate lisp_util;

mod javascript;
mod subcommands;

#[cfg(not(test))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/out/c_exports.rs"));
