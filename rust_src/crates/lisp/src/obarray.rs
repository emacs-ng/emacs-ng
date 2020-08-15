//! obarray code

use crate::{
    lisp::LispObject,
    multibyte::{LispStringRef, LispSymbolOrString},
    remacs_sys::Qvectorp,
    remacs_sys::{
        fatal_error_in_progress, globals, initial_obarray, intern_driver, oblookup, Fpurecopy,
    },
    symbol::LispSymbolRef,
    vector::LispVectorRef,
};

/// A lisp object containing an `obarray`.
#[repr(transparent)]
pub struct LispObarrayRef(LispObject);

impl From<LispObarrayRef> for LispObject {
    fn from(o: LispObarrayRef) -> Self {
        o.0
    }
}

impl From<&LispObarrayRef> for LispObject {
    fn from(o: &LispObarrayRef) -> Self {
        o.0
    }
}

impl LispObarrayRef {
    pub const fn new(obj: LispObject) -> Self {
        Self(obj)
    }

    /// Return a reference to the Lisp variable `obarray`.
    pub fn global() -> Self {
        Self(unsafe { globals.Vobarray }).check()
    }

    /// Return the symbol that matches NAME (either a symbol or string). If
    /// there is no such symbol, return the integer bucket number of where the
    /// symbol would be if it were present.
    pub fn lookup(&self, name: LispSymbolOrString) -> LispObject {
        let string: LispStringRef = name.into();
        unsafe {
            oblookup(
                self.into(),
                string.const_sdata_ptr(),
                string.len_chars(),
                string.len_bytes(),
            )
        }
    }

    /// Ensure that we have a valid obarray.
    pub fn check(self) -> Self {
        // We don't want to signal a wrong-type error when we are shutting
        // down due to a fatal error and we don't want to hit assertions
        // if the fatal error was during GC.
        if unsafe { fatal_error_in_progress } {
            return self;
        }

        // A valid obarray is a non-empty vector.
        let v = self.0.as_vector();
        if v.map_or(0, LispVectorRef::len) == 0 {
            // If Vobarray is now invalid, force it to be valid.
            if unsafe { globals.Vobarray }.eq(self.0) {
                unsafe { globals.Vobarray = initial_obarray };
            }
            wrong_type!(Qvectorp, self.0);
        }

        self
    }

    pub fn get(&self, idx: usize) -> LispSymbolRef {
        LispObject::from(self).force_vector().get(idx).into()
    }

    pub fn set<O: Into<LispObject>>(&mut self, idx: usize, item: O) {
        let mut vec = LispObject::from(&*self).force_vector();
        vec.set(idx, item.into());
    }

    /// Intern the string or symbol STRING. That is, return the new or existing
    /// symbol with that name in this `LispObarrayRef`. If Emacs is loading Lisp
    /// code to dump to an executable (ie. `purify-flag` is `t`), the symbol
    /// name will be transferred to pure storage.
    pub fn intern(&self, string: impl Into<LispSymbolOrString>) -> LispObject {
        let string = string.into();
        let tem = self.lookup(string);
        if tem.is_symbol() {
            tem
        } else {
            let string_copy: LispObject = if unsafe { globals.Vpurify_flag }.is_not_nil() {
                // When Emacs is running lisp code to dump to an executable, make
                // use of pure storage.
                unsafe { Fpurecopy(string.into()) }
            } else {
                string.into()
            };
            unsafe { intern_driver(string_copy, self.into(), tem) }
        }
    }
}

impl From<LispObject> for LispObarrayRef {
    fn from(o: LispObject) -> Self {
        Self::new(o).check()
    }
}

impl From<LispObject> for Option<LispObarrayRef> {
    fn from(o: LispObject) -> Self {
        if o.is_nil() {
            None
        } else {
            Some(o.into())
        }
    }
}
