#![deny(warnings)]
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
extern crate emacs;
#[cfg(feature = "libgit")]
extern crate git;
extern crate lisp_macros;
#[macro_use]
extern crate lisp_util;
extern crate ng_async;
extern crate remacs_lib;

#[cfg(feature = "javascript")]
extern crate deno;
#[cfg(feature = "javascript")]
extern crate deno_core;
#[cfg(feature = "javascript")]
extern crate deno_runtime;
extern crate futures;
#[cfg(feature = "javascript")]
extern crate rusty_v8;
#[cfg(feature = "javascript")]
extern crate tokio;

#[cfg(feature = "javascript")]
mod javascript;
mod javascript_stubs;
#[cfg(feature = "ng-module")]
mod ng_module;
#[cfg(feature = "javascript")]
mod subcommands;

#[cfg(feature = "window-system-webrender")]
mod webrender_backend;
#[cfg(feature = "window-system-webrender")]
mod wrterm;

#[cfg(feature = "window-system-webrender")]
pub use crate::wrterm::{tip_frame, wr_display_list};

#[cfg(not(feature = "javascript"))]
mod javascript {
    include!(concat!(env!("OUT_DIR"), "/javascript_exports.rs"));
}

// TODO: move to ng_async::ng_async
#[allow(dead_code)]
fn def_syms() {
    def_lisp_sym!(QCinchannel, "inchannel");
    def_lisp_sym!(QCoutchannel, "outchannel");
}

// TODO: move to ng_async::parsing
// In order to have rust generate symbols at compile time,
// I need a line of code starting with "def_lisp_sym"
// This function does not actually run any code, it should
// not be called at runtime. Doing so would actually be harmless
// as 'def_lisp_sym' generates no runtime code.
#[allow(dead_code)]
fn init_syms() {
    def_lisp_sym!(QCnull, ":null");
    def_lisp_sym!(QCfalse, ":false");
    def_lisp_sym!(QCobject_type, ":object-type");
    def_lisp_sym!(QCarray_type, ":array-type");
    def_lisp_sym!(QCnull_object, ":null-object");
    def_lisp_sym!(QCfalse_object, ":false-object");
    def_lisp_sym!(QCjson_config, ":json-config");
    def_lisp_sym!(Qalist, "alist");
    def_lisp_sym!(Qplist, "plist");
    def_lisp_sym!(Qarray, "array");
}

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
