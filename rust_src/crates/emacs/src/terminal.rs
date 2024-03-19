//! Functions related to terminal devices.

use std::ptr;

use libc::c_void;

use crate::bindings::build_string;
use crate::bindings::pvec_type;
use crate::bindings::terminal;
use crate::bindings::Fselected_frame;
use crate::globals::Qnil;
use crate::globals::Qterminal_live_p;
use crate::lisp::ExternalPtr;
use crate::lisp::LispObject;
use crate::vector::LispVectorlikeRef;

pub type TerminalRef = ExternalPtr<terminal>;

#[cfg(have_window_system)]
use crate::display_info::DisplayInfoRef;

impl TerminalRef {
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

    #[allow(unreachable_code)]
    #[cfg(have_window_system)]
    pub fn display_info(self) -> DisplayInfoRef {
        #[cfg(feature = "window-system-pgtk")]
        return DisplayInfoRef::new(unsafe { self.display_info.pgtk });
        #[cfg(feature = "window-system-winit")]
        return DisplayInfoRef::new(unsafe { self.display_info.winit });
        unimplemented!();
    }
}

impl LispObject {
    pub fn is_terminal(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_TERMINAL))
    }

    pub fn as_terminal(self) -> Option<TerminalRef> {
        self.as_vectorlike()
            .and_then(LispVectorlikeRef::as_terminal)
    }
}

impl From<LispObject> for Option<TerminalRef> {
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

        if let Some(term_ref) = TerminalRef::from_ptr(term as *mut c_void) {
            if term_ref.is_live() {
                return Some(term_ref);
            }
        }

        None
    }
}

impl From<LispObject> for TerminalRef {
    fn from(obj: LispObject) -> Self {
        let value: Option<Self> = obj.into();
        value.unwrap_or_else(|| wrong_type!(Qterminal_live_p, obj))
    }
}
