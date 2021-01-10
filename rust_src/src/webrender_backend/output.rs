use libc;
use std::{
    ops::{Deref, DerefMut},
    ptr,
};

use lisp::{lisp::ExternalPtr, remacs_sys::wr_output};

use super::display_info::DisplayInfoRef;
use super::font::FontRef;

pub struct OutputInner {
    pub display_info: DisplayInfoRef,
    pub font: FontRef,
}

pub type OutputInnerRef = ExternalPtr<OutputInner>;

#[derive(Default)]
#[repr(transparent)]
pub struct Output(wr_output);

impl Output {
    pub fn new() -> Self {
        let mut output = Output::default();

        let inner = Box::new(OutputInner {
            display_info: DisplayInfoRef::new(ptr::null_mut()),
            font: FontRef::new(ptr::null_mut()),
        });
        output.0.inner = Box::into_raw(inner) as *mut libc::c_void;

        output
    }

    pub fn get_inner(&self) -> OutputInnerRef {
        OutputInnerRef::new(self.0.inner as *mut OutputInner)
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        if self.0.inner != ptr::null_mut() {
            unsafe {
                Box::from_raw(self.0.inner as *mut OutputInner);
            }
        }
    }
}

#[repr(transparent)]
pub struct OutputRef(*mut Output);

impl Copy for OutputRef {}

// Derive fails for this type so do it manually
impl Clone for OutputRef {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl OutputRef {
    pub const fn new(p: *mut Output) -> Self {
        Self(p)
    }

    pub fn as_mut(&mut self) -> *mut wr_output {
        self.0 as *mut wr_output
    }
}

impl Deref for OutputRef {
    type Target = Output;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for OutputRef {
    fn deref_mut(&mut self) -> &mut Output {
        unsafe { &mut *self.0 }
    }
}

impl From<*mut wr_output> for OutputRef {
    fn from(o: *mut wr_output) -> Self {
        Self::new(o as *mut Output)
    }
}
