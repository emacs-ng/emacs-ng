use crate::frame::FrameExtGlRenderer;
use emacs::frame::FrameRef;

use webrender::api::ColorF;
use winit_term::output::OutputExtWinitTerm;

impl FrameExtGlRenderer for FrameRef {
    fn cursor_color(&self) -> ColorF {
        self.output().winit_term_data().cursor_color
    }

    fn scale_factor(&self) -> f64 {
        self.output()
            .winit_term_data()
            .window
            .as_ref()
            .expect("no winit window")
            .scale_factor()
    }

    fn cursor_foreground_color(&self) -> ColorF {
        self.output().winit_term_data().cursor_foreground_color
    }
}
