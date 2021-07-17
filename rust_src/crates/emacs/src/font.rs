//! font support

use crate::{
    bindings::font,
    bindings::{font_property_index::FONT_ENTITY_MAX, pvec_type},
    lisp::LispObject,
    vector::LispVectorlikeRef,
};

// A font is not a type in and of itself, it's just a group of three kinds of
// pseudovector. This newtype allows us to define methods that yield the actual
// font types: Spec, Entity, and Object.
#[repr(transparent)]
pub struct LispFontRef(LispVectorlikeRef);

impl LispFontRef {
    pub const fn from_vectorlike(v: LispVectorlikeRef) -> Self {
        Self(v)
    }

    pub fn as_font_mut(&mut self) -> *mut font {
        self.0.as_mut() as *mut font
    }
}

impl LispObject {
    pub fn is_font(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_FONT))
    }

    pub fn is_font_entity(self) -> bool {
        self.is_font()
            && self
                .as_vectorlike()
                .map_or(false, |v| v.pv_size() == FONT_ENTITY_MAX as isize)
    }

    pub fn as_font(self) -> Option<LispFontRef> {
        self.as_vectorlike().and_then(|v| {
            if v.is_pseudovector(pvec_type::PVEC_FONT) {
                Some(LispFontRef::from_vectorlike(v))
            } else {
                None
            }
        })
    }
}
