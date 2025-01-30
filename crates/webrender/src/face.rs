use crate::color::pixel_to_color;
use emacs_sys::bindings::face_underline_type;
use emacs_sys::display_traits::FaceRef;
use webrender::api::ColorF;
use webrender::api::LineStyle;

pub trait WrFace {
    fn bg_color_f(&self) -> ColorF;
    fn fg_color_f(&self) -> ColorF;
    fn underline_style(&self) -> Option<LineStyle>;
}

impl WrFace for FaceRef {
    fn bg_color_f(&self) -> ColorF {
        pixel_to_color(self.bg_color())
    }

    fn fg_color_f(&self) -> ColorF {
        pixel_to_color(self.fg_color())
    }

    fn underline_style(&self) -> Option<LineStyle> {
        wr_underline_style(self.underline())
    }
}

// TODO face_underline_type::FACE_UNDERLINE_DOUBLE_LINE
fn wr_underline_style(underline_type: face_underline_type::Type) -> Option<LineStyle> {
    match underline_type {
        face_underline_type::FACE_UNDERLINE_SINGLE => Some(LineStyle::Solid),
        face_underline_type::FACE_UNDERLINE_DOTS => Some(LineStyle::Dotted),
        face_underline_type::FACE_UNDERLINE_DASHES => Some(LineStyle::Dashed),
        face_underline_type::FACE_UNDERLINE_WAVE => Some(LineStyle::Wavy),
        _ => None,
    }
}
