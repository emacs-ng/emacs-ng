//! Generic frame functions.

use crate::{
    lisp::{ExternalPtr, LispObject},
    generated::{Lisp_Frame, Lisp_Type},
};

/// LispFrameRef is a reference to the LispFrame
/// However a reference is guaranteed to point to an existing frame
/// therefore no NULL checks are needed while using it
#[allow(dead_code)]
pub type LispFrameRef = ExternalPtr<Lisp_Frame>;

impl From<LispFrameRef> for LispObject {
    fn from(f: LispFrameRef) -> Self {
        Self::tag_ptr(f, Lisp_Type::Lisp_Vectorlike)
    }
}
