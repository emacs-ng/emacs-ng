use crate::gl::context::GLContextTrait;
use crate::output::CanvasRef;
use crate::window_system::frame::FrameId;
use crate::window_system::output::OutputRef;
use emacs::frame::LispFrameRef;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use webrender::api::ColorF;
use webrender::{self, api::units::*};

use super::display_info::DisplayInfoRef;

pub trait LispFrameExt {
    fn output(&self) -> OutputRef;
    fn canvas(&self) -> CanvasRef;
    fn cursor_color(&self) -> ColorF;
    fn cursor_foreground_color(&self) -> ColorF;
    fn display_info(&self) -> DisplayInfoRef;
    fn window_handle(&self) -> Option<RawWindowHandle>;
    fn display_handle(&self) -> Option<RawDisplayHandle>;
    fn size(&self) -> DeviceIntSize;
    fn unique_id(&self) -> FrameId;
}
pub trait LispFrameGlExt {
    fn create_gl_context(&self) -> crate::gl::context::GLContext;
}

impl LispFrameGlExt for LispFrameRef {
    fn create_gl_context(&self) -> crate::gl::context::GLContext {
        let display_handle = self
            .display_handle()
            .expect("Failed to raw display handle from frame");
        let window_handle = self
            .window_handle()
            .expect("Failed to get raw window handle from frame");
        let size = self.size();
        crate::gl::context::GLContext::build(size, display_handle, window_handle)
    }
}
