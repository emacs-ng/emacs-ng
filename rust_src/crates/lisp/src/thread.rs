//! Threading code

use std::mem;

use crate::{
    buffer::LispBufferRef,
    lisp::ExternalPtr,
    remacs_sys::{current_thread as current_thread_pointer, thread_state},
};

pub type ThreadStateRef = ExternalPtr<thread_state>;

pub struct ThreadState {}

impl ThreadState {
    pub fn current_buffer_unchecked() -> LispBufferRef {
        unsafe { mem::transmute((*current_thread_pointer).m_current_buffer) }
    }

    pub fn current_buffer() -> Option<LispBufferRef> {
        unsafe {
            LispBufferRef::from_ptr((*current_thread_pointer).m_current_buffer as *mut libc::c_void)
        }
    }
}
