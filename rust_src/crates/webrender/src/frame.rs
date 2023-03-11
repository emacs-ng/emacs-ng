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
    fn set_scale_factor(&mut self, scale_factor: f64) -> bool;
    fn unique_id(&self) -> FrameId;
}

pub trait LispFrameExt {
    fn canvas(&self) -> CanvasRef;
    fn logical_size(&self) -> LayoutSize;
    fn physical_size(&self) -> DeviceIntSize;
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

    fn logical_size(&self) -> LayoutSize {
        LayoutSize::new(self.pixel_width as f32, self.pixel_height as f32)
    }

    fn physical_size(&self) -> DeviceIntSize {
        let size = self.logical_size() * euclid::Scale::new(self.scale_factor() as f32);
        size.to_i32()
    }

    fn handle_size_change(&mut self, size: DeviceIntSize, scale_factor: f64) {
        log::trace!("frame handle_size_change: {size:?}");
        self.handle_scale_factor_change(scale_factor);

        self.change_size(
            size.width as i32,
            size.height as i32 - self.menu_bar_height,
            false,
            true,
            false,
        );

        unsafe { do_pending_window_change(false) };

        self.canvas().update();
    }

    fn handle_scale_factor_change(&mut self, scale_factor: f64) {
        log::trace!("frame handle_scale_factor_change... {scale_factor:?}");
        if self.set_scale_factor(scale_factor) {
            self.canvas().update();
        }
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
