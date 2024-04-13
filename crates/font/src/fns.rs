use emacs_sys::bindings::globals;
use emacs_sys::bindings::CHECK_SYMBOL;
use emacs_sys::bindings::CONSP;
use emacs_sys::bindings::XCAR;
use emacs_sys::bindings::XCDR;
use emacs_sys::globals::Qnil;
use emacs_sys::lisp::LispObject;
use lisp_macros::lisp_fn;

/// Convert emacs script name to OTF 4-letter script code.
/// See `otf-script-alist'.
#[lisp_fn(min = "1")]
pub fn script_to_otf(script: LispObject) -> LispObject {
    use emacs_sys::bindings::Frassq;
    unsafe { CHECK_SYMBOL(script) };
    let otf = unsafe { Frassq(script, globals.Votf_script_alist) };
    if otf.is_cons() {
        let otf = otf.force_cons();
        return otf.car();
    }
    Qnil
}

/// Convert a font registry.
/// See `registry-script-alist'.
#[lisp_fn(min = "1")]
pub fn registry_to_script(reg: LispObject) -> LispObject {
    unsafe { CHECK_SYMBOL(reg) };

    use emacs_sys::bindings::strncmp;
    use emacs_sys::bindings::SBYTES;
    use emacs_sys::bindings::SSDATA;
    use emacs_sys::bindings::SYMBOL_NAME;

    let mut rsa = unsafe { globals.Vregistry_script_alist };
    let mut r;
    while unsafe { CONSP(rsa) } {
        r = unsafe { XCAR(XCAR(rsa)) };
        if !unsafe {
            strncmp(
                SSDATA(r),
                SSDATA(SYMBOL_NAME(reg)),
                SBYTES(r).try_into().unwrap(),
            ) != 0
        } {
            return unsafe { XCDR(XCAR(rsa)) };
        }
        rsa = unsafe { XCDR(rsa) };
    }
    return Qnil;
}

include!(concat!(env!("OUT_DIR"), "/fns_exports.rs"));
