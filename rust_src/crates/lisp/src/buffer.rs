//! Functions operating on buffers

use libc::ptrdiff_t;

use crate::{
    lisp::{ExternalPtr, LispObject},
    remacs_sys::{Lisp_Buffer, Lisp_Type},
};

pub const BEG: ptrdiff_t = 1;
pub const BEG_BYTE: ptrdiff_t = 1;

pub type LispBufferRef = ExternalPtr<Lisp_Buffer>;

impl LispBufferRef {
    pub const fn beg(self) -> ptrdiff_t {
        BEG
    }

    pub const fn beg_byte(self) -> ptrdiff_t {
        BEG_BYTE
    }
}

impl From<LispBufferRef> for LispObject {
    fn from(b: LispBufferRef) -> Self {
        Self::tag_ptr(b, Lisp_Type::Lisp_Vectorlike)
    }
}
