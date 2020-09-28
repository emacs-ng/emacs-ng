//! Functions operating on numbers.

use crate::{
    lisp::{LispObject, INTMASK},
    remacs_sys::{
        EmacsInt, EmacsUint, Lisp_Bits, Lisp_Type, EMACS_INT_MAX, INTTYPEBITS, USE_LSB_TAG,
    },
};

// Largest and smallest numbers that can be represented as fixnums in
// Emacs lisp.
pub const MOST_POSITIVE_FIXNUM: EmacsInt = EMACS_INT_MAX >> INTTYPEBITS as u32;
pub(crate) const MOST_NEGATIVE_FIXNUM: EmacsInt = (-1 - MOST_POSITIVE_FIXNUM);

// Fixnum(Integer) support (LispType == Lisp_Int0 | Lisp_Int1 == 2 | 6(LSB) )

/// Fixnums are inline integers that fit directly into Lisp's tagged word.
/// There's two `LispType` variants to provide an extra bit.

/// Natnums(natural number) are the non-negative fixnums.
/// There were special branches in the original code for better performance.
/// However they are unified into the fixnum logic under LSB mode.
/// TODO: Recheck these logic in original C code.

impl LispObject {
    pub fn from_fixnum(n: EmacsInt) -> Self {
        debug_assert!(MOST_NEGATIVE_FIXNUM <= n && n <= MOST_POSITIVE_FIXNUM);
        Self::from_fixnum_truncated(n)
    }

    pub fn from_fixnum_truncated(n: EmacsInt) -> Self {
        let o = if USE_LSB_TAG {
            (n << INTTYPEBITS) as EmacsUint + Lisp_Type::Lisp_Int0 as EmacsUint
        } else {
            (n & INTMASK) as EmacsUint + ((Lisp_Type::Lisp_Int0 as EmacsUint) << Lisp_Bits::VALBITS)
        };
        Self::from_C(o as EmacsInt)
    }

    pub fn force_fixnum(self) -> EmacsInt {
        unsafe { self.to_fixnum_unchecked() }
    }
}

impl LispObject {
    pub unsafe fn to_fixnum_unchecked(self) -> EmacsInt {
        let raw = self.to_C();
        if USE_LSB_TAG {
            raw >> INTTYPEBITS
        } else {
            raw & INTMASK
        }
    }
}
