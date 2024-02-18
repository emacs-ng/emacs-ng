use emacs::lisp::ExternalPtr;
use webrender_api::ColorF;

use emacs::output::OutputRef;

use std::ptr;

pub struct WinitTermOutputData {
    pub background_color: ColorF,
    pub cursor_color: ColorF,
    pub cursor_foreground_color: ColorF,
    pub window: Option<crate::api::window::Window>,
    pub cursor_position: crate::api::dpi::PhysicalPosition<f64>,
}

impl Default for WinitTermOutputData {
    fn default() -> Self {
        WinitTermOutputData {
            background_color: ColorF::WHITE,
            cursor_color: ColorF::BLACK,
            cursor_foreground_color: ColorF::WHITE,
            window: None,
            cursor_position: crate::api::dpi::PhysicalPosition::new(0.0, 0.0),
        }
    }
}

impl WinitTermOutputData {
    pub fn set_window(&mut self, window: crate::api::window::Window) {
        self.window = Some(window);
    }

    pub fn set_cursor_color(&mut self, color: ColorF) {
        self.cursor_color = color;
    }

    pub fn set_cursor_position(&mut self, pos: crate::api::dpi::PhysicalPosition<f64>) {
        self.cursor_position = pos;
    }

    pub fn set_background_color(&mut self, color: ColorF) {
        self.background_color = color;
    }
}

pub type WinitTermOutputDataRef = ExternalPtr<WinitTermOutputData>;

pub trait OutputExtWinitTerm {
    fn free_winit_term_data(&mut self);
    fn set_winit_term_data(&mut self, inner: Box<WinitTermOutputData>);
    fn winit_term_data(&mut self) -> WinitTermOutputDataRef;
}

impl OutputExtWinitTerm for OutputRef {
    fn free_winit_term_data(&mut self) {
        let _ = unsafe { Box::from_raw(self.winit_term_data().as_mut()) };
        self.winit = ptr::null_mut();
    }

    fn set_winit_term_data(&mut self, data: Box<WinitTermOutputData>) {
        self.winit = Box::into_raw(data) as *mut libc::c_void;
    }

    fn winit_term_data(&mut self) -> WinitTermOutputDataRef {
        if self.winit.is_null() {
            let data = Box::new(WinitTermOutputData::default());
            self.winit = Box::into_raw(data) as *mut libc::c_void;
        }
        WinitTermOutputDataRef::new(self.winit as *mut WinitTermOutputData)
    }
}
