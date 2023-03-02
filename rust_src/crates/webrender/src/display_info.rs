use crate::window_system::display_info::*;
use libc;
use raw_window_handle::RawDisplayHandle;
use std::{collections::HashMap, ptr};

use emacs::{bindings::Emacs_GC, frame::LispFrameRef, lisp::ExternalPtr};

use crate::window_system::frame::FrameId;
use crate::{fringe::FringeBitmap, term::TerminalRef};

pub struct DisplayInfoInner {
    pub terminal: TerminalRef,
    pub focus_frame: LispFrameRef,

    pub frames: HashMap<FrameId, LispFrameRef>,

    pub scratch_cursor_gc: Box<Emacs_GC>,

    pub fringe_bitmap_caches: HashMap<i32, FringeBitmap>,

    pub raw_display_handle: Option<RawDisplayHandle>,
}

impl Default for DisplayInfoInner {
    fn default() -> Self {
        DisplayInfoInner {
            terminal: TerminalRef::new(ptr::null_mut()),
            focus_frame: LispFrameRef::new(ptr::null_mut()),
            frames: HashMap::new(),
            scratch_cursor_gc: Box::new(Emacs_GC {
                foreground: 0,
                background: 0,
            }),

            fringe_bitmap_caches: HashMap::new(),
            raw_display_handle: None,
        }
    }
}

pub type DisplayInfoInnerRef = ExternalPtr<DisplayInfoInner>;

#[derive(Default)]
#[repr(transparent)]
pub struct DisplayInfo(display_info);
pub type DisplayInfoRef = ExternalPtr<DisplayInfo>;

impl DisplayInfo {
    pub fn new() -> Self {
        let mut df = DisplayInfo::default();

        let inner = Box::new(DisplayInfoInner::default());
        df.0.inner = Box::into_raw(inner) as *mut libc::c_void;

        df
    }

    pub fn init_inner(&mut self) {
        let inner = Box::new(DisplayInfoInner::default());
        self.0.inner = Box::into_raw(inner) as *mut libc::c_void;
    }

    pub fn get_inner(&mut self) -> DisplayInfoInnerRef {
        if self.0.inner.is_null() {
            self.init_inner();
        }
        DisplayInfoInnerRef::new(self.0.inner as *mut DisplayInfoInner)
    }

    pub fn get_raw(&mut self) -> ExternalPtr<display_info> {
        (&mut self.0 as *mut display_info).into()
    }

    pub fn get_color_bits(&self) -> u8 {
        24
    }
}

impl Drop for DisplayInfo {
    fn drop(&mut self) {
        if self.0.inner != ptr::null_mut() {
            unsafe {
                let _ = Box::from_raw(self.0.inner as *mut DisplayInfoInner);
            }
        }
    }
}
