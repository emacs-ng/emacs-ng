use libc;
use std::{collections::HashMap, ptr};
use winit::window::WindowId;

use emacs::{
    bindings::{wr_display_info, Emacs_GC},
    frame::LispFrameRef,
    lisp::ExternalPtr,
};

use crate::{fringe::FringeBitmap, input::InputProcessor, output::OutputRef, term::TerminalRef};

pub struct DisplayInfoInner {
    pub terminal: TerminalRef,
    pub focus_frame: LispFrameRef,

    pub outputs: HashMap<WindowId, OutputRef>,

    pub input_processor: InputProcessor,

    pub scratch_cursor_gc: Box<Emacs_GC>,

    pub fringe_bitmap_caches: HashMap<i32, FringeBitmap>,
}

impl Default for DisplayInfoInner {
    fn default() -> Self {
        DisplayInfoInner {
            terminal: TerminalRef::new(ptr::null_mut()),
            focus_frame: LispFrameRef::new(ptr::null_mut()),
            outputs: HashMap::new(),
            input_processor: InputProcessor::new(),
            scratch_cursor_gc: Box::new(Emacs_GC {
                foreground: 0,
                background: 0,
            }),

            fringe_bitmap_caches: HashMap::new(),
        }
    }
}

pub type DisplayInfoInnerRef = ExternalPtr<DisplayInfoInner>;

#[derive(Default)]
#[repr(transparent)]
pub struct DisplayInfo(wr_display_info);

impl DisplayInfo {
    pub fn new() -> Self {
        let mut df = DisplayInfo::default();

        let inner = Box::new(DisplayInfoInner::default());
        df.0.inner = Box::into_raw(inner) as *mut libc::c_void;

        df
    }

    pub fn get_inner(&self) -> DisplayInfoInnerRef {
        DisplayInfoInnerRef::new(self.0.inner as *mut DisplayInfoInner)
    }

    pub fn get_raw(&mut self) -> ExternalPtr<wr_display_info> {
        (&mut self.0 as *mut wr_display_info).into()
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

pub type DisplayInfoRef = ExternalPtr<DisplayInfo>;
