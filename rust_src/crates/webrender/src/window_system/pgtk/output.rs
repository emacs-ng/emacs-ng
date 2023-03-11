use emacs::lisp::ExternalPtr;

use crate::output::Canvas;
use crate::output::CanvasRef;

use std::ptr;

pub struct OutputInner {
    pub scale_factor: f64,
    pub canvas: CanvasRef,
}

impl Default for OutputInner {
    fn default() -> Self {
        OutputInner {
            scale_factor: 0.0,
            canvas: CanvasRef::new(ptr::null_mut() as *mut _ as *mut Canvas),
        }
    }
}

impl OutputInner {
    pub fn set_canvas(&mut self, canvas: Box<Canvas>) {
        self.canvas = CanvasRef::new(Box::into_raw(canvas));
    }
}

pub type OutputInnerRef = ExternalPtr<OutputInner>;

pub type output = emacs::bindings::pgtk_output;
