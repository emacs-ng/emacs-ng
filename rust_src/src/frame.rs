//! Generic frame functions.

use crate::{lisp::ExternalPtr, remacs_sys::Lisp_Frame};

/// LispFrameRef is a reference to the LispFrame
/// However a reference is guaranteed to point to an existing frame
/// therefore no NULL checks are needed while using it
pub type LispFrameRef = ExternalPtr<Lisp_Frame>;
