//! symbols support

pub type LispSymbolRef = ExternalPtr<Lisp_Symbol>;
use crate::lisp::{ExternalPtr, LispObject};
use crate::remacs_sys::{
    indirect_function, lispsym, make_lisp_symbol, EmacsInt, Lisp_Symbol, Lisp_Type, Qsymbolp,
    USE_LSB_TAG,
};

impl LispSymbolRef {
    pub fn get_function(self) -> LispObject {
        let s = unsafe { self.u.s.as_ref() };
        s.function
    }

    pub fn get_indirect_function(self) -> LispObject {
        let obj = self.get_function();

        match obj.as_symbol() {
            None => obj,
            Some(_) => unsafe { indirect_function(obj) },
        }
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

// include!(concat!(env!("OUT_DIR"), "/symbols_exports.rs"));
