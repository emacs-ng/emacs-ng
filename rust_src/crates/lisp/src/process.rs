//! Functions operating on process.

pub type LispProcessRef = ExternalPtr<Lisp_Process>;

use crate::vectors::LispVectorlikeRef;

use crate::{
    lisp::{ExternalPtr, LispObject},
    remacs_sys::{Lisp_Process, Qprocessp},
};

impl LispObject {
    pub fn as_process(self) -> Option<LispProcessRef> {
        self.into()
    }
}

impl From<LispObject> for LispProcessRef {
    fn from(o: LispObject) -> Self {
        o.as_process().unwrap_or_else(|| wrong_type!(Qprocessp, o))
    }
}

impl From<LispObject> for Option<LispProcessRef> {
    fn from(o: LispObject) -> Self {
        o.as_vectorlike().and_then(LispVectorlikeRef::as_process)
    }
}
