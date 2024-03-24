#![cfg_attr(feature = "strict", deny(warnings))]

mod attributes;

// Used by lisp-macros and lisp-doc
pub use self::attributes::parse_lisp_fn;
