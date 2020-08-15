use libc;
use std::ptr;

use lisp::{frame::LispFrameRef, lisp::ExternalPtr, remacs_sys::wr_display_info};

use super::{keyboard::KeyboardProcessor, output::OutputRef, term::TerminalRef};

pub struct DisplayInfoInner {
    pub terminal: TerminalRef,
    pub focus_frame: LispFrameRef,

    pub output: OutputRef,

    pub keyboard_processor: KeyboardProcessor,
}

impl Default for DisplayInfoInner {
    fn default() -> Self {
        DisplayInfoInner {
            terminal: TerminalRef::new(ptr::null_mut()),
            focus_frame: LispFrameRef::new(ptr::null_mut()),
            output: OutputRef::new(ptr::null_mut()),
            keyboard_processor: KeyboardProcessor::new(),
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
                Box::from_raw(self.0.inner as *mut DisplayInfoInner);
            }
        }
    }
}

pub type DisplayInfoRef = ExternalPtr<DisplayInfo>;
