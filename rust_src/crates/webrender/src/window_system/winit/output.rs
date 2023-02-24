use emacs::lisp::ExternalPtr;
use webrender::api::ColorF;

use crate::output::Canvas;
use crate::output::CanvasRef;
use raw_window_handle::RawWindowHandle;

use std::ptr;

use crate::display_info::DisplayInfoRef;
use crate::font::FontRef;

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

#[derive(Default)]
#[repr(transparent)]
pub struct Output(output);
pub type OutputRef = ExternalPtr<Output>;

impl Output {
    pub fn new() -> Self {
        let ret = Output::default();
        ret
    }

    pub fn empty_inner(&mut self) {
        let _ = unsafe { Box::from_raw(self.get_inner().as_mut()) };
        self.0.inner = ptr::null_mut();
    }

    pub fn set_inner(&mut self, inner: Box<OutputInner>) {
        self.0.inner = Box::into_raw(inner) as *mut libc::c_void;
    }

    pub fn set_display_info(&mut self, mut dpyinfo: DisplayInfoRef) {
        self.0.display_info = dpyinfo.get_raw().as_mut();
    }

    pub fn set_font(&mut self, mut font: FontRef) {
        self.0.font = font.as_mut();
    }

    pub fn set_fontset(&mut self, fontset: i32) {
        self.0.fontset = fontset;
    }

    pub fn display_info(&self) -> DisplayInfoRef {
        DisplayInfoRef::new(self.0.display_info as *mut _)
    }

    pub fn get_font(&self) -> FontRef {
        FontRef::new(self.0.font as *mut _)
    }

    pub fn get_fontset(&self) -> i32 {
        self.0.fontset
    }

    pub fn get_canvas(&mut self) -> CanvasRef {
        self.get_inner().canvas
    }

    pub fn get_inner(&mut self) -> OutputInnerRef {
        if self.0.inner.is_null() {
            self.set_inner(Box::new(OutputInner::default()));
        }

        OutputInnerRef::new(self.0.inner as *mut OutputInner)
    }

    pub fn as_raw(&mut self) -> ExternalPtr<output> {
        (&mut self.0 as *mut output).into()
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        if self.0.inner != ptr::null_mut() {
            if OutputInnerRef::new(self.0.inner as *mut OutputInner)
                .canvas
                .as_mut()
                != ptr::null_mut()
            {
                unsafe {
                    let _ = Box::from_raw(
                        OutputInnerRef::new(self.0.inner as *mut OutputInner)
                            .canvas
                            .as_mut(),
                    );
                }
            }

            unsafe {
                let _ = Box::from_raw(self.0.inner as *mut OutputInner);
            }
        }
    }
}
