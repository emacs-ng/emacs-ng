#![allow(unused)]

//! This module contains all FFI declarations.
//!
//! These types and constants are generated at build time to mimic how they are
//! in C:
//!
//! - `EmacsInt`
//! - `EmacsUint`
//! - `EmacsDouble`
//! - `EMACS_INT_MAX`
//! - `EMACS_INT_SIZE`
//! - `EMACS_FLOAT_SIZE`
//! - `GCTYPEBITS`
//! - `USE_LSB_TAG`
//! - `BoolBF`

use libc::{self, c_char, c_int, c_void, ptrdiff_t};
use std::mem;

pub type Lisp_Object = LispObject;

use crate::{
    bindings::{
        hash_table_test, vectorlike_header, Aligned_Lisp_Subr, Lisp_Subr, Lisp_Type,
        __IncompleteArrayField, GCTYPEBITS,
    },
    definitions::{EmacsInt, EMACS_INT_MAX, USE_LSB_TAG},
    lisp::{ExternalPtr, LispObject},
};

pub const VAL_MAX: EmacsInt = (EMACS_INT_MAX >> (GCTYPEBITS - 1));
pub const VALMASK: EmacsInt = [VAL_MAX, -(1 << GCTYPEBITS)][USE_LSB_TAG as usize];

pub const PSEUDOVECTOR_FLAG: usize = 0x4000_0000_0000_0000;

// These signal an error, therefore are marked as non-returning.
extern "C" {
    pub fn circular_list(tail: Lisp_Object) -> !;
    pub fn wrong_type_argument(predicate: Lisp_Object, value: Lisp_Object) -> !;
    // defined in eval.c, where it can actually take an arbitrary
    // number of arguments.
    // TODO: define a Rust version of this that uses Rust strings.
    pub fn error(m: *const u8, ...) -> !;
}

// bindgen apparently misses these, for various reasons
extern "C" {
    // these weren't declared in a header, for example
    pub static Vprocess_alist: Lisp_Object;
    pub fn concat(
        nargs: ptrdiff_t,
        args: *mut LispObject,
        target_type: Lisp_Type,
        last_special: bool,
    ) -> LispObject;
}

// In order to use `lazy_static!` with LispSubr, it must be Sync. Raw
// pointers are not Sync, but it isn't a problem to define Sync if we
// never mutate LispSubr values. If we do, we will need to create
// these objects at runtime, perhaps using forget().
//
// Based on http://stackoverflow.com/a/28116557/509706
unsafe impl Sync for Lisp_Subr {}
unsafe impl Sync for Aligned_Lisp_Subr {}
unsafe impl Sync for crate::lisp::LispSubrRef {}

#[repr(C)]
pub struct Lisp_Vectorlike {
    pub header: vectorlike_header,
    // shouldn't look at the contents without knowing the structure...
}

// No C equivalent.  Generic type for a vectorlike with one or more
// LispObject slots after the header.
#[repr(C)]
pub struct Lisp_Vectorlike_With_Slots {
    pub header: vectorlike_header,
    // actually any number of items... not sure how to express this
    pub contents: __IncompleteArrayField<Lisp_Object>,
}

//// declare this ourselves so that the arg isn't mutable
//extern "C" {
//    pub fn staticpro(arg1: *const Lisp_Object);
//}

impl Clone for hash_table_test {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            user_hash_function: self.user_hash_function,
            user_cmp_function: self.user_cmp_function,
            cmpfn: self.cmpfn,
            hashfn: self.hashfn,
        }
    }
}

pub mod EmacsModifiers {
    pub type Type = u32;

    pub const up_modifier: Type = 1;
    pub const down_modifier: Type = 2;
    pub const drag_modifier: Type = 4;
    pub const click_modifier: Type = 8;
    pub const double_modifier: Type = 16;
    pub const triple_modifier: Type = 32;
    pub const alt_modifier: Type = 4194304;
    pub const super_modifier: Type = 8388608;
    pub const hyper_modifier: Type = 16777216;
    pub const shift_modifier: Type = 33554432;
    pub const ctrl_modifier: Type = 67108864;
    pub const meta_modifier: Type = 134217728;
}
