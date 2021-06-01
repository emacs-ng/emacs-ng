#![cfg_attr(feature = "strict", deny(warnings))]

mod attributes;

// Used by remacs-macros and remacs-lib
pub use self::attributes::parse_lisp_fn;
