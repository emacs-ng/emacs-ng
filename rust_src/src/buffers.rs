// //! Functions operating on buffers.

// use libc::{self, c_uchar, ptrdiff_t};

// use crate::{
//     lisp::{ExternalPtr, LispObject},
//     remacs_sys::{Lisp_Buffer, Lisp_Type},
// };

// pub const BEG: ptrdiff_t = 1;
// pub const BEG_BYTE: ptrdiff_t = 1;

// pub type LispBufferRef = ExternalPtr<Lisp_Buffer>;

// impl LispBufferRef {
//     pub const fn beg(self) -> ptrdiff_t {
//         BEG
//     }

//     pub const fn beg_byte(self) -> ptrdiff_t {
//         BEG_BYTE
//     }

//     // Characters, positions and byte positions.

//     /// Return the address of byte position N in current buffer.
//     pub fn byte_pos_addr(self, n: ptrdiff_t) -> *mut c_uchar {
//         let offset = if n >= self.gpt_byte() {
//             self.gap_size()
//         } else {
//             0
//         };

//         unsafe { self.beg_addr().offset(offset + n - self.beg_byte()) }
//     }

//     /// Return the byte at byte position N.
//     pub fn fetch_byte(self, n: ptrdiff_t) -> u8 {
//         unsafe { *self.byte_pos_addr(n) }
//     }

//     // Methods for accessing struct buffer_text fields

//     pub fn beg_addr(self) -> *mut c_uchar {
//         unsafe { (*self.text).beg }
//     }

//     pub fn gpt_byte(self) -> ptrdiff_t {
//         unsafe { (*self.text).gpt_byte }
//     }

//     pub fn gap_size(self) -> ptrdiff_t {
//         unsafe { (*self.text).gap_size }
//     }
// }

// impl From<LispBufferRef> for LispObject {
//     fn from(b: LispBufferRef) -> Self {
//         Self::tag_ptr(b, Lisp_Type::Lisp_Vectorlike)
//     }
// }

// // include!(concat!(env!("OUT_DIR"), "/buffers_exports.rs"));
