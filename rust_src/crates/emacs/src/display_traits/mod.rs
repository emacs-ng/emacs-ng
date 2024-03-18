// Interface definitions for display code.

use crate::bindings::glyph;
use crate::lisp::ExternalPtr;
pub type GlyphRef = ExternalPtr<glyph>;
#[cfg(feature = "window-system")]
mod glyph_string;
#[cfg(feature = "window-system")]
pub use glyph_string::*;
