//! Emacs-ng's extensions to the dynamic module interface, `emacs-module.h`.
//!
//! Since we want to be ABI-compatible with dynamic modules written for vanilla Emacs, we don't add
//! more functions to `emacs_env`. Instead, we provide a registry of named native functions that
//! dynamic modules can look up at run time. We call these `ng-module functions`. Unlike normal
//! module functions, ng-module functions have globally stable addresses, which should be looked up
//! once, at module load time (in `emacs_module_init`). Their names should start with `ng_module_`.
//!
//! To ensure ABI compatibility, once an ng-module function is added, its signature must not be
//! changed. If a new function is a slight modification of an existing function, consider using a
//! suffix (descriptive, or numeric) to distinguish them. It is ok to remove an ng-module function.
//! However, its name must not be reused.
//!
//! To make the ABI easy to consume, primitive data types are preferred over structs. If a struct
//! is used, it must be marked `#[repr(C)]`, and its layout must not be changed.

use emacs::{
    bindings::{buffer_text, current_thread, make_user_ptr, BYTE_POS_ADDR},
    globals::Qnil,
    lisp::LispObject,
    multibyte::LispStringRef,
};
use lisp_macros::lisp_fn;

/// Return the address of the ng-module function with the given NAME.
/// Return nil if Emacs-ng does not provide such a module function.
///
/// For the full list of available functions, see the file 'ng_module.rs'.
///
/// This function is intended to be used by dynamic modules at module
/// load time (in `emacs_module_init'), not normal Lisp code.
#[lisp_fn]
pub fn ng_module_function_address(name: LispStringRef) -> LispObject {
    macro_rules! expose {
        ($($name:ident)*) => {
            match name.to_utf8().as_ref() {
                $(stringify!($name) => unsafe { make_user_ptr(None, $name as *mut libc::c_void) },)*
                _ => Qnil,
            }
        }
    }
    expose! {
        ng_module_access_current_buffer_contents
    }
}

/// Returns the pointers to, and the sizes of the 2 contiguous segments inside the current buffer.
///
/// This can be used for direct read-only access to the current buffer's text.
///
/// # Safety
///
/// Various operations can invalidate the returned values: buffer modifications, garbage collection,
/// arena compaction...
unsafe extern "C" fn ng_module_access_current_buffer_contents(
    before_gap_ptr: *mut *const u8,
    before_gap_size: *mut isize,
    after_gap_ptr: *mut *const u8,
    after_gap_size: *mut isize,
) {
    let buffer = (*current_thread).m_current_buffer;
    let text = (*buffer).text;
    let buffer_text {
        beg,
        gpt_byte,
        gap_size,
        z_byte,
        ..
    } = *text;
    let beg_byte = 1;
    *before_gap_ptr = BYTE_POS_ADDR(beg_byte);
    *before_gap_size = gpt_byte - beg_byte;
    *after_gap_ptr = beg.add((*before_gap_size + gap_size) as usize);
    *after_gap_size = z_byte - gpt_byte;
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/out/ng_module_exports.rs"
));
