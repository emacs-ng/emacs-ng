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
#[macro_export]
macro_rules! xsignal {
    ($symbol:expr) => {
        #[allow(unused_unsafe)]
        unsafe {
            $crate::eval::signal_rust($symbol, crate::remacs_sys::Qnil);
        }
    };
    ($symbol:expr, $($tt:tt)+) => {
        #[allow(unused_unsafe)]
        unsafe {
            $crate::eval::signal_rust($symbol, list!($($tt)+));
        }
    };
}

/// Macro to call Lisp functions with any number of arguments.
/// Replaces call0, call1, etc. in the C layer.
#[allow(unused_macros)]
#[macro_export]
macro_rules! call {
    ($func:expr, $($arg:expr),*) => {
        crate::eval::funcall(&mut [$func, $($arg),*])
    };
    ($func:expr) => {
        crate::eval::funcall(&mut [$func])
    }
}

/// Macro to format a "wrong argument type" error message.
#[macro_export]
macro_rules! wrong_type {
    ($pred:expr, $arg:expr) => {
        xsignal!($crate::remacs_sys::Qwrong_type_argument, $pred, $arg);
    };
}

#[macro_export]
macro_rules! list {
    ($arg:expr, $($tt:tt)+) => { $crate::lisp::LispObject::cons($arg, list!($($tt)+)) };
    ($arg:expr) => { $crate::lisp::LispObject::cons($arg, list!()) };
    () => { $crate::remacs_sys::Qnil };
}

#[macro_export]
macro_rules! error {
    ($str:expr) => {{
        #[allow(unused_unsafe)]
        let strobj = unsafe {
            $crate::remacs_sys::make_string($str.as_ptr() as *const ::libc::c_char,
                                      $str.len() as ::libc::ptrdiff_t)
        };
        xsignal!($crate::remacs_sys::Qerror, strobj);
    }};
    ($fmtstr:expr, $($arg:expr),*) => {{
        let formatted = format!($fmtstr, $($arg),*);
        #[allow(unused_unsafe)]
        let strobj = unsafe {
            $crate::remacs_sys::make_string(formatted.as_ptr() as *const ::libc::c_char,
                                      formatted.len() as ::libc::ptrdiff_t)
        };
        xsignal!($crate::remacs_sys::Qerror, strobj);
    }};
}

/// Macro that expands to nothing, but is used at build time to
/// generate the starting symbol table. Equivalent to the DEFSYM
/// macro. See also lib-src/make-docfile.c
#[macro_export]
macro_rules! def_lisp_sym {
    ($name:expr, $value:expr) => {};
}

/// Macros we use to define forwarded Lisp variables.
/// These are used in the syms_of_FILENAME functions.
///
/// An ordinary (not in buffer_defaults, per-buffer, or per-keyboard)
/// lisp variable is actually a field in `struct emacs_globals'.
///
/// In the C code, the field's name begins with "f_", which is a
/// convention enforced by these macros.  Each such global has a
/// corresponding #define in globals.h; the plain name should be used
/// in the C code.
///
/// E.g., the global "cons_cells_consed" is declared as "int
/// f_cons_cells_consed" in globals.h, but there is a define:
///
///    #define cons_cells_consed globals.f_cons_cells_consed
///
/// All C code uses the `cons_cells_consed' name.
///
/// As the Rust macro system has identifier hygine, the Rust code's
/// version of the struct emacs_globals does not include the f_ prefix
/// on the field names, and Rust code accesses the fields directly,
/// rather than through a macro.
///
/// This is all done this way to support indirection for
/// multi-threaded Emacs.
#[macro_export]
macro_rules! defvar_lisp {
    ($field_name:ident, $lisp_name:expr, $value:expr) => {{
        #[allow(unused_unsafe)]
        unsafe {
            use $crate::remacs_sys::Lisp_Objfwd;

            static mut o_fwd: Lisp_Objfwd = Lisp_Objfwd {
                type_: $crate::remacs_sys::Lisp_Fwd_Type::Lisp_Fwd_Obj,
                objvar: unsafe { &$crate::remacs_sys::globals.$field_name as *const _ as *mut _ },
            };

            $crate::remacs_sys::defvar_lisp(
                &o_fwd,
                concat!($lisp_name, "\0").as_ptr() as *const libc::c_char,
            );
            $crate::remacs_sys::globals.$field_name = $value;
        }
    }};
}

#[macro_export]
macro_rules! defvar_bool {
    ($field_name:ident, $lisp_name:expr, $value:expr) => {{
        unsafe {
            use $crate::remacs_sys::Lisp_Boolfwd;

            static mut o_fwd: Lisp_Boolfwd = Lisp_Boolfwd {
                type_: $crate::remacs_sys::Lisp_Fwd_Type::Lisp_Fwd_Bool,
                boolvar: unsafe { &$crate::remacs_sys::globals.$field_name as *const _ as *mut _ },
            };

            $crate::remacs_sys::defvar_bool(
                &o_fwd,
                concat!($lisp_name, "\0").as_ptr() as *const libc::c_char,
            );
            $crate::remacs_sys::globals.$field_name = $value;
        }
    }};
}

macro_rules! args_out_of_range {
    ($($tt:tt)+) => { xsignal!(crate::remacs_sys::Qargs_out_of_range, $($tt)+); };
}
