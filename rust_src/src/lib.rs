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
#![feature(ptr_offset_from)]
#![feature(slice_patterns)]
#![feature(specialization)]
#![feature(stmt_expr_attributes)]
#![feature(untagged_unions)]

extern crate errno;
#[macro_use]
extern crate lazy_static;

extern crate libc;

extern crate field_offset;

extern crate core;

// // Wilfred/remacs#38 : Need to override the allocator for legacy unexec support on Mac.
// #[cfg(all(not(test), target_os = "macos", feature = "unexecmacosx"))]
// extern crate alloc_unexecmacosx;

// Needed for linking.
extern crate remacs_lib;
extern crate remacs_macros;

mod remacs_sys;
#[macro_use]
mod lisp;
mod lists;
// #[cfg(all(not(test), target_os = "macos", feature = "unexecmacosx"))]
// use alloc_unexecmacosx::OsxUnexecAlloc;

// #[cfg(all(not(test), target_os = "macos", feature = "unexecmacosx"))]
// #[global_allocator]
// static ALLOCATOR: OsxUnexecAlloc = OsxUnexecAlloc;

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));

// // #[cfg(test)]
// // pub use crate::functions::{lispsym, make_string, make_unibyte_string, Fcons};

// // mod hacks {
// //     use core::mem::ManuallyDrop;

// //     #[allow(unions_with_drop_fields)]
// //     pub union Hack<T> {
// //         t: ManuallyDrop<T>,
// //         u: (),
// //     }

// //     impl<T> Hack<T> {
// //         pub const unsafe fn uninitialized() -> Self {
// //             Self { u: () }
// //         }

// //         pub unsafe fn get_mut(&mut self) -> &mut T {
// //             &mut *self.t
// //         }
// //     }
// // }

