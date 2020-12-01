//! Generic Lisp eval macros.

/*
 * N.B. Wherever unsafe occurs in this file the line should be preceded
 * by `#[allow(unused_unsafe)]`. This allows the macro to be called
 * from within an `unsafe` block without the compiler complaining that
 * the unsafe is not used.
 */

/// Macro to generate an error with a list from any number of arguments.
/// Replaces xsignal0, etc. in the C layer.
///
/// Like `Fsignal`, but never returns. Can be used for any error
/// except `Qquit`, which can return from `Fsignal`. See the elisp docstring
/// for `signal` for an explanation of the arguments.
macro_rules! xsignal {
    ($symbol:expr) => {
        #[allow(unused_unsafe)]
        unsafe {
            crate::eval::signal_rust($symbol, crate::remacs_sys::Qnil);
        }
    };
    ($symbol:expr, $($tt:tt)+) => {
        #[allow(unused_unsafe)]
        unsafe {
            crate::eval::signal_rust($symbol, list!($($tt)+));
        }
    };
}

/// Macro to format a "wrong argument type" error message.
macro_rules! wrong_type {
    ($pred:expr, $arg:expr) => {
        xsignal!(crate::remacs_sys::Qwrong_type_argument, $pred, $arg);
    };
}

macro_rules! list {
    ($arg:expr, $($tt:tt)+) => { $crate::lisp::LispObject::cons($arg, list!($($tt)+)) };
    ($arg:expr) => { $crate::lisp::LispObject::cons($arg, list!()) };
    () => { crate::remacs_sys::Qnil };
}

macro_rules! error {
    ($str:expr) => {{
        #[allow(unused_unsafe)]
        let strobj = unsafe {
            crate::remacs_sys::make_string($str.as_ptr() as *const ::libc::c_char,
                                      $str.len() as ::libc::ptrdiff_t)
        };
        xsignal!(crate::remacs_sys::Qerror, strobj);
    }};
    ($fmtstr:expr, $($arg:expr),*) => {{
        let formatted = format!($fmtstr, $($arg),*);
        #[allow(unused_unsafe)]
        let strobj = unsafe {
            crate::remacs_sys::make_string(formatted.as_ptr() as *const ::libc::c_char,
                                      formatted.len() as ::libc::ptrdiff_t)
        };
        xsignal!(crate::remacs_sys::Qerror, strobj);
    }};
}
