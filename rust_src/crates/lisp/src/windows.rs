use crate::{lisp::LispObject, remacs_sys::pvec_type};

impl LispObject {
    pub fn is_window(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_WINDOW))
    }
}
