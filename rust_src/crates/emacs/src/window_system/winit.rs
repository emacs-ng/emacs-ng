use crate::lisp::ExternalPtr;
use webrender_api::ColorF;

use crate::output::OutputRef;

use std::ptr;

pub struct WinitTermOutputData {
    pub background_color: ColorF,
    pub cursor_color: ColorF,
    pub cursor_foreground_color: ColorF,
    pub window: Option<winit::window::Window>,
    pub cursor_position: winit::dpi::PhysicalPosition<f64>,
}

impl Default for WinitTermOutputData {
    fn default() -> Self {
        WinitTermOutputData {
            background_color: ColorF::WHITE,
            cursor_color: ColorF::BLACK,
            cursor_foreground_color: ColorF::WHITE,
            window: None,
            cursor_position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
        }
    }
}

impl WinitTermOutputData {
    pub fn set_window(&mut self, window: winit::window::Window) {
        self.window = Some(window);
    }

    pub fn set_cursor_color(&mut self, color: ColorF) {
        self.cursor_color = color;
    }

    pub fn set_cursor_position(&mut self, pos: winit::dpi::PhysicalPosition<f64>) {
        self.cursor_position = pos;
    }

    pub fn set_background_color(&mut self, color: ColorF) {
        self.background_color = color;
    }
}

pub type WinitTermOutputDataRef = ExternalPtr<WinitTermOutputData>;

impl OutputRef {
    pub fn free_winit_term_data(&mut self) {
        let _ = unsafe { Box::from_raw(self.winit_term_data().as_mut()) };
        self.winit = ptr::null_mut();
    }

    pub fn set_winit_term_data(&mut self, data: Box<WinitTermOutputData>) {
        self.winit = Box::into_raw(data) as *mut libc::c_void;
    }

    pub fn winit_term_data(&mut self) -> WinitTermOutputDataRef {
        if self.winit.is_null() {
            let data = Box::new(WinitTermOutputData::default());
            self.winit = Box::into_raw(data) as *mut libc::c_void;
        }
        WinitTermOutputDataRef::new(self.winit as *mut WinitTermOutputData)
    }
}

use crate::frame::FrameRef;
use crate::terminal::TerminalRef;
use arboard::Clipboard;
use raw_window_handle::HasRawDisplayHandle;
use raw_window_handle::RawDisplayHandle;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopBuilder;
use winit::monitor::MonitorHandle;

pub struct WinitTermData {
    pub terminal: TerminalRef,
    pub focus_frame: FrameRef,
    pub clipboard: Clipboard,
    pub all_frames: Vec<FrameRef>,
    pub event_loop: EventLoop<i32>,
}

impl Default for WinitTermData {
    fn default() -> Self {
        let event_loop = EventLoopBuilder::<i32>::with_user_event()
            .build()
            .ok()
            .unwrap();
        let clipboard = Clipboard::new().unwrap();
        WinitTermData {
            terminal: TerminalRef::new(ptr::null_mut()),
            focus_frame: FrameRef::new(ptr::null_mut()),
            all_frames: Vec::new(),
            event_loop,
            clipboard,
        }
    }
}

pub type WinitTermDataRef = ExternalPtr<WinitTermData>;

impl TerminalRef {
    pub fn available_monitors(&mut self) -> impl Iterator<Item = MonitorHandle> {
        self.winit_term_data().event_loop.available_monitors()
    }

    pub fn primary_monitor(&mut self) -> MonitorHandle {
        self.winit_term_data()
            .event_loop
            .primary_monitor()
            .unwrap_or_else(|| -> MonitorHandle { self.available_monitors().next().unwrap() })
    }

    pub fn init_winit_term_data(&mut self) {
        let winit_term_data = Box::new(WinitTermData::default());
        self.winit_term_data = Box::into_raw(winit_term_data) as *mut libc::c_void;
    }

    pub fn winit_term_data(&mut self) -> WinitTermDataRef {
        if self.winit_term_data.is_null() {
            self.init_winit_term_data();
        }
        WinitTermDataRef::new(self.winit_term_data as *mut WinitTermData)
    }

    pub fn get_color_bits(&self) -> u8 {
        24
    }

    pub fn free_winit_term_data(&mut self) {
        if self.winit_term_data != ptr::null_mut() {
            unsafe {
                let _ = Box::from_raw(self.winit_term_data as *mut WinitTermData);
            }
        }
    }
}

use raw_window_handle::HasRawWindowHandle;
use raw_window_handle::RawWindowHandle;

unsafe impl HasRawWindowHandle for FrameRef {
    fn raw_window_handle(&self) -> RawWindowHandle {
        if let Some(window) = &self.output().winit_term_data().window {
            return window.raw_window_handle();
        } else {
            panic!("raw window handle not avaiable")
        }
    }
}

unsafe impl HasRawDisplayHandle for TerminalRef {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        if self.winit_term_data.is_null() {
            panic!("raw display handle not avaiable")
        }
        let data = WinitTermDataRef::new(self.winit_term_data as *mut WinitTermData);
        data.event_loop.raw_display_handle()
    }
}

unsafe impl HasRawDisplayHandle for FrameRef {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        if let Some(window) = &self.output().winit_term_data().window {
            return window.raw_display_handle();
        } else {
            panic!("raw display handle not avaiable")
        }
    }
}

impl FrameRef {
    pub fn cursor_color(&self) -> ColorF {
        self.output().winit_term_data().cursor_color
    }

    pub fn scale_factor(&self) -> f64 {
        self.output()
            .winit_term_data()
            .window
            .as_ref()
            .expect("no winit window")
            .scale_factor()
    }

    pub fn cursor_foreground_color(&self) -> ColorF {
        self.output().winit_term_data().cursor_foreground_color
    }
}
