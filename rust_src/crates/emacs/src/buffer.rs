//! Functions operating on buffers

use libc::ptrdiff_t;

use crate::{
    bindings::{buffer, pvec_type, Lisp_Type},
    globals::Qbufferp,
    lisp::{ExternalPtr, LispObject},
    vector::LispVectorlikeRef,
};

pub const BEG: ptrdiff_t = 1;
pub const BEG_BYTE: ptrdiff_t = 1;

pub type Lisp_Buffer = buffer;

pub type LispBufferRef = ExternalPtr<Lisp_Buffer>;

impl LispBufferRef {
    pub const fn beg(self) -> ptrdiff_t {
        BEG
    }

    pub const fn beg_byte(self) -> ptrdiff_t {
        BEG_BYTE
    }
}

impl From<LispObject> for LispBufferRef {
    fn from(o: LispObject) -> Self {
        o.as_buffer().unwrap_or_else(|| wrong_type!(Qbufferp, o))
    }
}

impl From<LispBufferRef> for LispObject {
    fn from(b: LispBufferRef) -> Self {
        Self::tag_ptr(b, Lisp_Type::Lisp_Vectorlike)
    }
}

impl From<LispObject> for Option<LispBufferRef> {
    fn from(o: LispObject) -> Self {
        o.as_vectorlike().and_then(LispVectorlikeRef::as_buffer)
    }
}

impl LispObject {
    pub fn is_buffer(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_BUFFER))
    }

    pub fn as_buffer(self) -> Option<LispBufferRef> {
        self.into()
    }
}
