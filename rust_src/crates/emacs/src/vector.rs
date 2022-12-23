//! Functions operating on vector(like)s, and general sequences.

use libc::ptrdiff_t;
use std::mem;

use lazy_static::lazy_static;

use crate::{
    bindings::{pvec_type, Lisp_Type, Lisp_Vector, More_Lisp_Bits},
    frame::LispFrameRef,
    lisp::{ExternalPtr, LispObject, LispSubrRef},
    process::LispProcessRef,
    sys::{Lisp_Vectorlike, PSEUDOVECTOR_FLAG},
    terminal::LispTerminalRef,
    window::LispWindowRef,
};

pub type LispVectorlikeRef = ExternalPtr<Lisp_Vectorlike>;
#[allow(dead_code)]
pub type LispVectorRef = ExternalPtr<Lisp_Vector>;

// Vectorlike support (LispType == 5)

impl LispObject {
    pub fn is_vectorlike(self) -> bool {
        self.get_type() == Lisp_Type::Lisp_Vectorlike
    }

    pub fn as_vectorlike(self) -> Option<LispVectorlikeRef> {
        if self.is_vectorlike() {
            Some(unsafe { self.as_vectorlike_unchecked() })
        } else {
            None
        }
    }

    pub unsafe fn as_vectorlike_unchecked(self) -> LispVectorlikeRef {
        LispVectorlikeRef::new(self.get_untaggedptr() as *mut Lisp_Vectorlike)
    }

    pub unsafe fn as_vector_unchecked(self) -> LispVectorRef {
        self.as_vectorlike_unchecked().as_vector_unchecked()
    }

    pub fn force_vector(self) -> LispVectorRef {
        unsafe { self.as_vector_unchecked() }
    }

    pub fn as_vector(self) -> Option<LispVectorRef> {
        self.as_vectorlike().and_then(LispVectorlikeRef::as_vector)
    }
}

impl LispVectorlikeRef {
    pub fn is_vector(self) -> bool {
        unsafe { self.header.size & (PSEUDOVECTOR_FLAG as isize) == 0 }
    }

    pub fn as_vector(self) -> Option<LispVectorRef> {
        if self.is_vector() {
            Some(self.cast())
        } else {
            None
        }
    }

    pub fn is_pseudovector(self, tp: pvec_type) -> bool {
        unsafe {
            self.header.size
                & (PSEUDOVECTOR_FLAG | More_Lisp_Bits::PVEC_TYPE_MASK as usize) as isize
                == (PSEUDOVECTOR_FLAG | ((tp as usize) << More_Lisp_Bits::PSEUDOVECTOR_AREA_BITS))
                    as isize
        }
    }

    pub fn as_subr(self) -> Option<LispSubrRef> {
        if self.is_pseudovector(pvec_type::PVEC_SUBR) {
            Some(unsafe { mem::transmute(self) })
        } else {
            None
        }
    }

    pub fn as_process(self) -> Option<LispProcessRef> {
        if self.is_pseudovector(pvec_type::PVEC_PROCESS) {
            Some(self.cast())
        } else {
            None
        }
    }

    pub fn as_window(self) -> Option<LispWindowRef> {
        if self.is_pseudovector(pvec_type::PVEC_WINDOW) {
            Some(self.cast())
        } else {
            None
        }
    }

    pub unsafe fn as_vector_unchecked(self) -> LispVectorRef {
        self.cast()
    }

    pub fn as_frame(self) -> Option<LispFrameRef> {
        if self.is_pseudovector(pvec_type::PVEC_FRAME) {
            Some(self.cast())
        } else {
            None
        }
    }

    pub fn as_terminal(self) -> Option<LispTerminalRef> {
        if self.is_pseudovector(pvec_type::PVEC_TERMINAL) {
            Some(self.cast())
        } else {
            None
        }
    }

    pub fn pv_size(self) -> isize {
        unsafe { self.header.size & More_Lisp_Bits::PSEUDOVECTOR_SIZE_MASK as isize }
    }
}

macro_rules! impl_vectorlike_ref {
    ($type:ident, $itertype:ident, $size_mask:expr) => {
        impl From<$type> for LispObject {
            fn from(v: $type) -> Self {
                Self::tag_ptr(v, Lisp_Type::Lisp_Vectorlike)
            }
        }

        impl $type {
            pub fn len(self) -> usize {
                (unsafe { self.header.size } & ($size_mask as isize)) as usize
            }

            pub fn is_empty(self) -> bool {
                self.len() == 0
            }

            pub fn as_slice(&self) -> &[LispObject] {
                let l = self.len();
                unsafe { self.contents.as_slice(l) }
            }

            pub fn as_mut_slice(&mut self) -> &mut [LispObject] {
                let l = self.len();
                unsafe { self.contents.as_mut_slice(l) }
            }

            pub fn get(self, idx: usize) -> LispObject {
                assert!(idx < self.len());
                unsafe { self.get_unchecked(idx) }
            }

            pub unsafe fn get_unchecked(self, idx: usize) -> LispObject {
                self.as_slice()[idx]
            }

            pub fn set(&mut self, idx: usize, item: LispObject) {
                assert!(idx < self.len());
                unsafe { self.set_unchecked(idx, item) };
            }

            pub fn set_checked(&mut self, idx: usize, item: LispObject) {
                if idx >= self.len() {
                    args_out_of_range!(*self, idx);
                }

                unsafe { self.set_unchecked(idx, item) };
            }

            pub unsafe fn set_unchecked(&mut self, idx: usize, item: LispObject) {
                self.as_mut_slice()[idx] = item
            }

            pub fn iter(&self) -> $itertype {
                $itertype::new(self)
            }
        }

        pub struct $itertype<'a> {
            vec: &'a $type,
            cur: usize,
            rev: usize,
        }

        impl<'a> $itertype<'a> {
            pub fn new(vec: &'a $type) -> Self {
                Self {
                    vec,
                    cur: 0,
                    rev: vec.len(),
                }
            }
        }

        impl<'a> Iterator for $itertype<'a> {
            type Item = LispObject;

            fn next(&mut self) -> Option<Self::Item> {
                if self.cur < self.rev {
                    let res = unsafe { self.vec.get_unchecked(self.cur) };
                    self.cur += 1;
                    Some(res)
                } else {
                    None
                }
            }
        }

        impl<'a> DoubleEndedIterator for $itertype<'a> {
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.rev > self.cur {
                    let res = unsafe { self.vec.get_unchecked(self.rev - 1) };
                    self.rev -= 1;
                    Some(res)
                } else {
                    None
                }
            }
        }

        impl<'a> ExactSizeIterator for $itertype<'a> {}
    };
}

impl_vectorlike_ref! { LispVectorRef, LispVecIterator, ptrdiff_t::max_value() }

lazy_static! {
    pub static ref HEADER_SIZE: usize =
        memoffset::offset_of!(crate::bindings::Lisp_Vector, contents);
    pub static ref WORD_SIZE: usize = ::std::mem::size_of::<crate::lisp::LispObject>();
}
