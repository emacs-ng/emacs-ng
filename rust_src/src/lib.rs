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

extern crate errno;
extern crate lazy_static;

extern crate core;
extern crate field_offset;
extern crate libc;

// Needed for linking.
extern crate remacs_lib;
extern crate remacs_macros;

extern crate futures;
extern crate lsp_server;
#[macro_use]
extern crate serde_json;

#[macro_use]
mod remacs_sys;
#[macro_use]
mod lisp;
#[macro_use]
mod eval_macros;
mod ng_async;
mod parsing;

mod eval;
mod lists;
mod multibyte;
mod process;
mod vectors;

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
