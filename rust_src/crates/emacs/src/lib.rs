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
#![feature(never_type)]
#![feature(stmt_expr_attributes)]
#![feature(async_closure)]

#[rustfmt::skip]
pub mod bindings;
#[rustfmt::skip]
pub mod definitions;
#[rustfmt::skip]
pub mod globals;

#[macro_use]
pub mod sys;
#[macro_use]
pub mod eval_macros;
#[macro_use]
pub mod vector_macros;
pub mod lisp;

pub mod eval;
pub mod font;
pub mod frame;
#[cfg(feature = "window-system")]
pub mod glyph;
pub mod keyboard;
pub mod list;
pub mod multibyte;
pub mod number;
pub mod obarray;
pub mod process;
pub mod string;
pub mod symbol;
pub mod terminal;
pub mod vector;
pub mod window;
pub mod xdisp;
