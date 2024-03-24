//! Display generation from window structure and buffer text.

/// Display a null-terminated echo area message M.  If M is 0, clear
/// out any existing message, and let the mini-buffer text show through.

/// The buffer M must continue to exist until after the echo area gets
/// cleared or some other message gets displayed there.  Do not pass
/// text that is stored in a Lisp string.  Do not pass text in a buffer
/// that was alloca'd.
pub fn message1(m: *const ::libc::c_char) {
    unsafe { crate::bindings::message1(m) };
}
