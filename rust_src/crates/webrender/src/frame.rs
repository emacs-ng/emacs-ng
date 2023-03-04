use super::font::update_wrfonts;
use super::font::FontRef;
use crate::gl::context::GLContextTrait;
use crate::output::Canvas;
use crate::output::CanvasRef;
use crate::output::OutputRef;
use crate::window_system::frame::FrameId;
use emacs::bindings::do_pending_window_change;
use emacs::frame::LispFrameRef;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use webrender::api::ColorF;
use webrender::{self, api::units::*};

use super::display_info::DisplayInfoRef;

pub trait LispFrameWindowSystemExt {
    fn output(&self) -> OutputRef;
    fn cursor_color(&self) -> ColorF;
    fn cursor_foreground_color(&self) -> ColorF;
    fn window_handle(&self) -> Option<RawWindowHandle>;
    fn display_handle(&self) -> Option<RawDisplayHandle>;
    fn scale_factor(&self) -> f64;
    fn unique_id(&self) -> FrameId;
}

pub trait LispFrameExt {
    fn canvas(&self) -> CanvasRef;
    fn size(&self) -> DeviceIntSize;
    fn font(&self) -> FontRef;
    fn set_font(&mut self, font: FontRef);
    fn fontset(&self) -> i32;
    fn set_fontset(&mut self, fontset: i32);
    fn display_info(&self) -> DisplayInfoRef;
    fn set_display_info(&mut self, dpyinfo: DisplayInfoRef);
    fn handle_size_change(&mut self, size: DeviceIntSize, scale_factor: f64);
    fn handle_scale_factor_change(&mut self, _scale_factor: f64);
    fn create_gl_context(&self) -> crate::gl::context::GLContext;
}

impl LispFrameExt for LispFrameRef {
    fn font(&self) -> FontRef {
        FontRef::new(self.output().as_raw().font as *mut _)
    }

    fn fontset(&self) -> i32 {
        self.output().as_raw().fontset
    }

    fn set_font(&mut self, mut font: FontRef) {
        self.output().as_raw().font = font.as_mut();
    }

    fn set_fontset(&mut self, fontset: i32) {
        self.output().as_raw().fontset = fontset;
    }

    fn display_info(&self) -> DisplayInfoRef {
        self.output().display_info()
    }

    fn set_display_info(&mut self, mut dpyinfo: DisplayInfoRef) {
        self.output().as_raw().display_info = dpyinfo.get_raw().as_mut();
    }

    fn size(&self) -> DeviceIntSize {
        DeviceIntSize::new(self.pixel_width, self.pixel_height)
    }

    fn handle_size_change(&mut self, size: DeviceIntSize, _scale_factor: f64) {
        self.change_size(
            size.width as i32,
            size.height as i32 - self.menu_bar_height,
            false,
            true,
            false,
        );

        unsafe { do_pending_window_change(false) };

        // resize after frame size been set
        // canvas read size from frame
        self.canvas().resize(&size);
    }

    fn handle_scale_factor_change(&mut self, scale_factor: f64) {
        log::trace!("frame handle_scale_factor_change...");
        update_wrfonts(self.unique_id(), scale_factor as f32);
    }

    fn canvas(&self) -> CanvasRef {
        if self.output().canvas().is_null() {
            log::debug!("canvas_data empty");
            let canvas = Box::new(Canvas::build(self.clone()));
            self.output().inner().set_canvas(canvas);
        }

        self.output().canvas()
    }

    fn create_gl_context(&self) -> crate::gl::context::GLContext {
        crate::gl::context::GLContext::build(self)
    }
}
