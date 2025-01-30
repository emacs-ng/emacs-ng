use crate::bindings::glyph;
use crate::bindings::glyph_type;
use crate::display_traits::FaceId;
use crate::lisp::ExternalPtr;

pub type GlyphRef = ExternalPtr<glyph>;

impl GlyphRef {
    pub fn glyph_type(&self) -> glyph_type::Type {
        self.type_()
    }

    pub fn face_id2(&self) -> FaceId {
        let face_id =
            unsafe { std::mem::transmute::<u32, crate::bindings::face_id>(self.face_id()) };
        face_id.into()
    }
}
