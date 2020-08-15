//! Generic frame functions.

use std::ffi::CString;

use crate::{
    lisp::{ExternalPtr, LispObject},
    remacs_sys::{gui_default_parameter, resource_types, Lisp_Frame, Lisp_Type},
};

/// LispFrameRef is a reference to the LispFrame
/// However a reference is guaranteed to point to an existing frame
/// therefore no NULL checks are needed while using it
#[allow(dead_code)]
pub type LispFrameRef = ExternalPtr<Lisp_Frame>;

impl LispFrameRef {
    pub fn gui_default_parameter(
        mut self,
        alist: LispObject,
        prop: LispObject,
        default: LispObject,
        xprop: &str,
        xclass: &str,
        res_type: resource_types::Type,
    ) {
        let xprop = CString::new(xprop).unwrap().as_ptr();
        let xclass = CString::new(xclass).unwrap().as_ptr();

        unsafe {
            gui_default_parameter(self.as_mut(), alist, prop, default, xprop, xclass, res_type);
        };
    }
}

impl From<LispFrameRef> for LispObject {
    fn from(f: LispFrameRef) -> Self {
        Self::tag_ptr(f, Lisp_Type::Lisp_Vectorlike)
    }
}
