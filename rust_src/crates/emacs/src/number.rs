//! Functions operating on numbers.

use crate::{
    bindings::{Lisp_Bits, Lisp_Type, INTTYPEBITS},
    definitions::{EmacsInt, EmacsUint, EMACS_INT_MAX, USE_LSB_TAG},
    globals::{Qintegerp, Qwholenump},
    lisp::LispObject,
};

// Largest and smallest numbers that can be represented as fixnums in
// Emacs lisp.
pub const INTMASK: EmacsInt = EMACS_INT_MAX >> (INTTYPEBITS - 1);
pub const MOST_POSITIVE_FIXNUM: EmacsInt = EMACS_INT_MAX >> INTTYPEBITS as u32;
pub const MOST_NEGATIVE_FIXNUM: EmacsInt = -1 - MOST_POSITIVE_FIXNUM;

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

    pub fn is_fixnum(self) -> bool {
        let ty = self.get_type();
        (ty as u8 & ((Lisp_Type::Lisp_Int0 as u8) | !(Lisp_Type::Lisp_Int1 as u8)))
            == Lisp_Type::Lisp_Int0 as u8
    }

    pub fn force_fixnum(self) -> EmacsInt {
        unsafe { self.to_fixnum_unchecked() }
    }

    pub fn as_fixnum(self) -> Option<EmacsInt> {
        if self.is_fixnum() {
            Some(unsafe { self.to_fixnum_unchecked() })
        } else {
            None
        }
    }

    pub fn as_fixnum_or_error(self) -> EmacsInt {
        if self.is_fixnum() {
            unsafe { self.to_fixnum_unchecked() }
        } else {
            wrong_type!(Qintegerp, self)
        }
    }
}

impl LispObject {
    /// Convert a positive integer into its LispObject representation.
    ///
    /// This is also the function to use when translating `XSETFASTINT`
    /// from Emacs C.
    // TODO: the C claims that make_natnum is faster, but it does the same
    // thing as make_number when USE_LSB_TAG is 1, which it is for us. We
    // should remove this in favour of make_number.
    pub fn from_natnum(n: EmacsUint) -> Self {
        debug_assert!(n <= (MOST_POSITIVE_FIXNUM as EmacsUint));
        Self::from_fixnum_truncated(n as EmacsInt)
    }

    pub fn is_natnum(self) -> bool {
        self.as_fixnum().map_or(false, |i| i >= 0)
    }

    pub fn as_natnum(self) -> Option<EmacsUint> {
        if self.is_natnum() {
            Some(unsafe { self.to_fixnum_unchecked() as EmacsUint })
        } else {
            None
        }
    }

    pub fn as_natnum_or_error(self) -> EmacsUint {
        self.as_natnum()
            .unwrap_or_else(|| wrong_type!(Qwholenump, self))
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
