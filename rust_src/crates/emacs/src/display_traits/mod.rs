// Interface definitions for display code.
use crate::bindings::glyph;
use crate::lisp::ExternalPtr;

pub type GlyphRef = ExternalPtr<glyph>;
#[cfg(have_window_system)]
mod glyph_string;
#[cfg(have_window_system)]
pub use glyph_string::*;
