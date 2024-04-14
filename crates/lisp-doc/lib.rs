#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![feature(lazy_cell)]
#![cfg_attr(feature = "strict", deny(warnings))]

extern crate libc;
extern crate lisp_util;
extern crate regex;

mod docfile;

pub use crate::{
    // Used by make-docfile
    docfile::scan_rust_file,
};
