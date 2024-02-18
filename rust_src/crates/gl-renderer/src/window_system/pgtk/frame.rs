use crate::frame::FrameExtGlRenderer;
use emacs::color::pixel_to_color;
use emacs::frame::FrameRef;
use emacs::FrameExtPgtk;
use gtk::prelude::WidgetExt;
use webrender::api::ColorF;

impl FrameExtGlRenderer for FrameRef {
    fn cursor_color(&self) -> ColorF {
        let color = self.output().cursor_color;
        pixel_to_color(color)
    }

    fn scale_factor(&self) -> f64 {
        if !self.parent_frame.is_nil() {
            let parent: FrameRef = self.parent_frame.into();
            return parent.scale_factor();
        }

        // fallback using widget
        if let Some(widget) = self.edit_widget() {
            return widget.scale_factor() as f64;
        }

        1.0
    }

    fn cursor_foreground_color(&self) -> ColorF {
        let color = self.output().cursor_foreground_color;
        pixel_to_color(color)
    }
}
