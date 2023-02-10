//! Generic Lisp eval functions

use crate::{bindings::xsignal, lisp::LispObject};

/// Signal an error.  Args are ERROR-SYMBOL and associated DATA. This
/// function does not return.
///
/// An error symbol is a symbol with an `error-conditions' property
/// that is a list of condition names.  A handler for any of those
/// names will get to handle this signal.  The symbol `error' should
/// normally be one of them.
///
/// DATA should be a list.  Its elements are printed as part of the
/// error message.  See Info anchor `(elisp)Definition of signal' for
/// some details on how this error message is constructed.
/// If the signal is handled, DATA is made available to the handler.
/// See also the function `condition-case'.
pub fn signal_rust(error_symbol: LispObject, data: LispObject) -> ! {
    #[cfg(test)]
    {
        panic!("Fsignal called during tests.");
    }
    #[cfg(not(test))]
    {
        unsafe { xsignal(error_symbol, data) };
        unreachable!();
    }
}
