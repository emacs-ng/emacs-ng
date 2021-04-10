//! symbols support

use std::ptr;

use crate::{
    bindings::make_lisp_symbol,
    bindings::{lispsym, Lisp_Symbol, Lisp_Type},
    definitions::{EmacsInt, USE_LSB_TAG},
    globals::Qsymbolp,
    lisp::{ExternalPtr, LispObject},
};

pub type LispSymbolRef = ExternalPtr<Lisp_Symbol>;

impl LispSymbolRef {
    pub fn symbol_name(self) -> LispObject {
        let s = unsafe { self.u.s.as_ref() };
        s.name
    }

    pub fn get_function(self) -> LispObject {
        let s = unsafe { self.u.s.as_ref() };
        s.function
    }

    pub fn get_plist(self) -> LispObject {
        let s = unsafe { self.u.s.as_ref() };
        s.plist
    }

    pub fn set_plist(&mut self, plist: LispObject) {
        let s = unsafe { self.u.s.as_mut() };
        s.plist = plist;
    }

    pub fn set_function(&mut self, function: LispObject) {
        let s = unsafe { self.u.s.as_mut() };
        s.function = function;
    }

    pub unsafe fn get_value(self) -> LispObject {
        let s = self.u.s.as_ref();
        s.val.value
    }

    pub const fn iter(self) -> LispSymbolIter {
        LispSymbolIter { current: self }
    }

    pub fn get_next(self) -> Option<Self> {
        // `iter().next()` returns the _current_ symbol: we want
        // another `next()` on the iterator to really get the next
        // symbol. we use `nth(1)` as a shortcut here.
        self.iter().nth(1)
    }

    pub fn set_next(mut self, next: Option<Self>) {
        let mut s = unsafe { self.u.s.as_mut() };
        s.next = match next {
            Some(sym) => sym.as_ptr() as *mut Lisp_Symbol,
            None => ptr::null_mut(),
        };
    }
}

impl From<LispObject> for LispSymbolRef {
    fn from(o: LispObject) -> Self {
        if let Some(sym) = o.as_symbol() {
            sym
        } else {
            wrong_type!(Qsymbolp, o)
        }
    }
}

impl From<LispSymbolRef> for LispObject {
    fn from(mut s: LispSymbolRef) -> Self {
        unsafe { make_lisp_symbol(s.as_mut()) }
    }
}

impl From<LispObject> for Option<LispSymbolRef> {
    fn from(o: LispObject) -> Self {
        if o.is_symbol() {
            Some(LispSymbolRef::new(o.symbol_ptr_value() as *mut Lisp_Symbol))
        } else {
            None
        }
    }
}

// Symbol support (LispType == Lisp_Symbol == 0)
impl LispObject {
    pub fn is_symbol(self) -> bool {
        self.get_type() == Lisp_Type::Lisp_Symbol
    }

    pub fn force_symbol(self) -> LispSymbolRef {
        LispSymbolRef::new(self.symbol_ptr_value() as *mut Lisp_Symbol)
    }

    pub fn as_symbol(self) -> Option<LispSymbolRef> {
        self.into()
    }

    fn symbol_ptr_value(self) -> EmacsInt {
        let ptr_value = if USE_LSB_TAG {
            self.to_C()
        } else {
            self.get_untaggedptr() as EmacsInt
        };

        let lispsym_offset = unsafe { &lispsym as *const _ as EmacsInt };
        ptr_value + lispsym_offset
    }
}

pub struct LispSymbolIter {
    current: LispSymbolRef,
}

impl Iterator for LispSymbolIter {
    type Item = LispSymbolRef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let sym = self.current;
            let s = unsafe { sym.u.s.as_ref() };
            self.current = LispSymbolRef::new(s.next);
            Some(sym)
        }
    }
}
