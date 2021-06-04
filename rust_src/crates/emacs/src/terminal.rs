//! Functions related to terminal devices.

use std::ptr;

use libc::c_void;

use crate::{
    bindings::{build_string, pvec_type, terminal, Fselected_frame},
    globals::{Qnil, Qterminal_live_p},
    lisp::{ExternalPtr, LispObject},
    vector::LispVectorlikeRef,
};

pub type LispTerminalRef = ExternalPtr<terminal>;

impl LispTerminalRef {
    pub fn is_live(self) -> bool {
        !self.name.is_null()
    }

    pub fn name(self) -> LispObject {
        if self.name.is_null() {
            Qnil
        } else {
            unsafe { build_string(self.name) }
        }
    }
}

impl LispObject {
    pub fn is_terminal(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_TERMINAL))
    }

    pub fn as_terminal(self) -> Option<LispTerminalRef> {
        self.as_vectorlike()
            .and_then(LispVectorlikeRef::as_terminal)
    }
}

impl From<LispObject> for Option<LispTerminalRef> {
    fn from(obj: LispObject) -> Self {
        let obj = if obj.is_nil() {
            unsafe { Fselected_frame() }
        } else {
            obj
        };

        let term = if let Some(frame) = obj.as_frame() {
            frame.terminal
        } else if let Some(mut terminal) = obj.as_terminal() {
            terminal.as_mut()
        } else {
            ptr::null_mut()
        };

        if let Some(term_ref) = LispTerminalRef::from_ptr(term as *mut c_void) {
            if term_ref.is_live() {
                return Some(term_ref);
            }
        }

        None
    }
}

impl From<LispObject> for LispTerminalRef {
    fn from(obj: LispObject) -> Self {
        let value: Option<Self> = obj.into();
        value.unwrap_or_else(|| wrong_type!(Qterminal_live_p, obj))
    }
}
