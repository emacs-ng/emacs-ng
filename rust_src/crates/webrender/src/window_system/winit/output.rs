use emacs::lisp::ExternalPtr;
use webrender::api::ColorF;

use crate::output::Canvas;
use crate::output::CanvasRef;
use raw_window_handle::RawWindowHandle;

use std::ptr;

pub struct OutputInner {
    pub background_color: ColorF,
    pub cursor_color: ColorF,
    pub cursor_foreground_color: ColorF,
    pub window_handle: Option<RawWindowHandle>,
    pub window: Option<crate::window_system::api::window::Window>,
    #[cfg(use_winit)]
    pub cursor_position: crate::window_system::api::dpi::PhysicalPosition<f64>,
    pub canvas: CanvasRef,
}

impl Default for OutputInner {
    fn default() -> Self {
        OutputInner {
            background_color: ColorF::WHITE,
            cursor_color: ColorF::BLACK,
            cursor_foreground_color: ColorF::WHITE,
            window_handle: None,
            #[cfg(window_system_winit)]
            window: None,
            #[cfg(all(window_system_winit, use_winit))]
            cursor_position: crate::window_system::api::dpi::PhysicalPosition::new(0.0, 0.0),
            canvas: CanvasRef::new(ptr::null_mut() as *mut _ as *mut Canvas),
        }
    }
}

impl OutputInner {
    pub fn set_canvas(&mut self, canvas: Box<Canvas>) {
        self.canvas = CanvasRef::new(Box::into_raw(canvas));
    }

    #[cfg(window_system_winit)]
    pub fn set_window(&mut self, window: crate::window_system::api::window::Window) {
        self.window = Some(window);
    }

    pub fn set_cursor_color(&mut self, color: ColorF) {
        self.cursor_color = color;
    }

    #[cfg(use_winit)]
    pub fn set_cursor_position(
        &mut self,
        pos: crate::window_system::api::dpi::PhysicalPosition<f64>,
    ) {
        self.cursor_position = pos;
    }

    pub fn set_background_color(&mut self, color: ColorF) {
        self.background_color = color;
    }
}

pub type OutputInnerRef = ExternalPtr<OutputInner>;

pub type output = emacs::bindings::winit_output;
