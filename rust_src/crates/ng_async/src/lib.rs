#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(concat_idents)]
#![feature(async_closure)]

extern crate libc;

extern crate lazy_static;
extern crate futures;
extern crate lsp_server;
#[macro_use]
extern crate serde_json;
extern crate crossbeam;

#[macro_use]
extern crate emacs;
extern crate lisp_macros;
#[macro_use]
extern crate lisp_util;

mod ng_async;
mod parsing;

#[cfg(not(test))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/out/c_exports.rs"));
