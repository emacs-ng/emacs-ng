//! Operations on lists.

use crate::{lisp::LispObject, remacs_sys::Fcons};

impl LispObject {
    pub fn cons(car: impl Into<Self>, cdr: impl Into<Self>) -> Self {
        unsafe { Fcons(car.into(), cdr.into()) }
    }
}
