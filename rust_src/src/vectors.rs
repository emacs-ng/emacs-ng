//! Functions operating on vector(like)s, and general sequences.

use std::mem;

use crate::{
    lisp::{ExternalPtr, LispObject, LispSubrRef},
    process::LispProcessRef,
    remacs_sys::{
        pvec_type, Lisp_Type, Lisp_Vector, Lisp_Vectorlike, More_Lisp_Bits, PSEUDOVECTOR_FLAG,
    },
};

pub type LispVectorlikeRef = ExternalPtr<Lisp_Vectorlike>;
#[allow(dead_code)]
pub type LispVectorRef = ExternalPtr<Lisp_Vector>;

// Vectorlike support (LispType == 5)

impl LispObject {
    pub fn is_vectorlike(self) -> bool {
        self.get_type() == Lisp_Type::Lisp_Vectorlike
    }

    pub fn as_vectorlike(self) -> Option<LispVectorlikeRef> {
        if self.is_vectorlike() {
            Some(unsafe { self.as_vectorlike_unchecked() })
        } else {
            None
        }
    }

    pub unsafe fn as_vectorlike_unchecked(self) -> LispVectorlikeRef {
        LispVectorlikeRef::new(self.get_untaggedptr() as *mut Lisp_Vectorlike)
    }
}

impl LispVectorlikeRef {
    pub fn is_pseudovector(self, tp: pvec_type) -> bool {
        unsafe {
            self.header.size
                & (PSEUDOVECTOR_FLAG | More_Lisp_Bits::PVEC_TYPE_MASK as usize) as isize
                == (PSEUDOVECTOR_FLAG | ((tp as usize) << More_Lisp_Bits::PSEUDOVECTOR_AREA_BITS))
                    as isize
        }
    }

    pub fn as_subr(self) -> Option<LispSubrRef> {
        if self.is_pseudovector(pvec_type::PVEC_SUBR) {
            Some(unsafe { mem::transmute(self) })
        } else {
            None
        }
    }

    pub fn as_process(self) -> Option<LispProcessRef> {
        if self.is_pseudovector(pvec_type::PVEC_PROCESS) {
            Some(self.cast())
        } else {
            None
        }
    }
}
