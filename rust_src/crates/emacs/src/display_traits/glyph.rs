use crate::bindings::glyph;
use crate::display_traits::FaceId;
use crate::display_traits::GlyphType;
use crate::lisp::ExternalPtr;

pub type GlyphRef = ExternalPtr<glyph>;

impl GlyphRef {
    pub fn glyph_type(&self) -> GlyphType {
        self.type_().into()
    }

    pub fn face_id2(&self) -> FaceId {
        let face_id =
            unsafe { std::mem::transmute::<u32, crate::bindings::face_id>(self.face_id()) };
        face_id.into()
    }
}
