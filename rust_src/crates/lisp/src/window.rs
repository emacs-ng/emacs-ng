use crate::{
    lisp::{ExternalPtr, LispObject},
    remacs_sys::{pvec_type, Lisp_Type, Lisp_Window, Qwindowp},
    vector::LispVectorlikeRef,
};

pub type LispWindowRef = ExternalPtr<Lisp_Window>;

impl LispWindowRef {
    /// A window of any sort, leaf or interior, is "valid" if its
    /// contents slot is non-nil.
    pub fn is_valid(self) -> bool {
        self.contents.is_not_nil()
    }
}

impl LispObject {
    pub fn is_window(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_WINDOW))
    }

    pub fn as_window(self) -> Option<LispWindowRef> {
        self.into()
    }

    pub fn as_valid_window(self) -> Option<LispWindowRef> {
        self.as_window()
            .and_then(|w| if w.is_valid() { Some(w) } else { None })
    }
}

impl From<LispObject> for LispWindowRef {
    fn from(o: LispObject) -> Self {
        o.as_window().unwrap_or_else(|| wrong_type!(Qwindowp, o))
    }
}

impl From<LispWindowRef> for LispObject {
    fn from(w: LispWindowRef) -> Self {
        Self::tag_ptr(w, Lisp_Type::Lisp_Vectorlike)
    }
}

impl From<LispObject> for Option<LispWindowRef> {
    fn from(o: LispObject) -> Self {
        o.as_vectorlike().and_then(LispVectorlikeRef::as_window)
    }
}
