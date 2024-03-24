#[cfg(use_webrender)]
use webrender_api::ColorF;
#[cfg(use_webrender)]
use webrender_api::LineStyle;

use crate::bindings::face as Face;
#[cfg(use_webrender)]
use crate::color::pixel_to_color;
use crate::font::FontRef;
use crate::lisp::ExternalPtr;

use super::FaceUnderlineType;

pub type FaceRef = ExternalPtr<Face>;

impl FaceRef {
    pub fn font(&self) -> FontRef {
        FontRef::new(self.font)
    }

    pub fn underline_type(&self) -> FaceUnderlineType {
        self.underline().into()
    }

    #[cfg(use_webrender)]
    pub fn underline_style(&self) -> Option<LineStyle> {
        self.underline_type().into()
    }

    #[cfg(use_webrender)]
    pub fn bg_color(&self) -> ColorF {
        pixel_to_color(self.background)
    }

    #[cfg(use_webrender)]
    pub fn fg_color(&self) -> ColorF {
        pixel_to_color(self.foreground)
    }
}
