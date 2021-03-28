//! Operations on lists.

use libc::c_void;

use crate::{
    bindings::{Fcons, CHECK_IMPURE},
    bindings::{Lisp_Cons, Lisp_Type},
    globals::{Qconsp, Qlistp},
    lisp::LispObject,
    number::MOST_POSITIVE_FIXNUM,
    sys::circular_list,
};

// Cons support (LispType == 6 | 3)

/// A newtype for objects we know are conses.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct LispCons(LispObject);

impl LispObject {
    pub fn check_list(self) {
        if !self.is_list() {
            wrong_type!(Qlistp, self);
        }
    }

    pub fn check_list_end(self, list: Self) {
        if !self.is_nil() {
            wrong_type!(Qlistp, list);
        }
    }

    pub fn is_cons(self) -> bool {
        self.get_type() == Lisp_Type::Lisp_Cons
    }

    pub const fn force_cons(self) -> LispCons {
        LispCons(self)
    }

    pub fn as_cons(self) -> Option<LispCons> {
        if self.is_cons() {
            Some(LispCons(self))
        } else {
            None
        }
    }
}

impl LispObject {
    pub fn cons(car: impl Into<Self>, cdr: impl Into<Self>) -> Self {
        unsafe { Fcons(car.into(), cdr.into()) }
    }

    pub fn is_list(self) -> bool {
        self.is_cons() || self.is_nil()
    }

    /// Iterate over all tails of self.  self should be a list, i.e. a chain
    /// of cons cells ending in nil.
    /// wrong-type-argument error will be signaled if END_CHECKS is 'on'.
    pub fn iter_tails(
        self,
        end_checks: LispConsEndChecks,
        circular_checks: LispConsCircularChecks,
    ) -> TailsIter {
        TailsIter::new(self, Qlistp, end_checks, circular_checks)
    }

    /// Iterate over the car cells of a list.
    pub fn iter_cars(
        self,
        end_checks: LispConsEndChecks,
        circular_checks: LispConsCircularChecks,
    ) -> CarIter {
        CarIter::new(TailsIter::new(self, Qlistp, end_checks, circular_checks))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LispConsEndChecks {
    off, // no checks
    on,  // error when the last item inspected is not a valid cons cell.
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LispConsCircularChecks {
    off,  // no checks
    safe, // checked, exits when a circular list is found.
    on,   // raises error when a circular list is found.
}

/// From `FOR_EACH_TAIL_INTERNAL` in `lisp.h`
pub struct TailsIter {
    list: LispObject,
    tail: LispObject,
    tortoise: LispObject,
    errsym: Option<LispObject>,
    circular_checks: LispConsCircularChecks,
    max: isize,
    n: isize,
    q: u16,
}

impl TailsIter {
    pub fn new(
        list: LispObject,
        ty: LispObject,
        end_checks: LispConsEndChecks,
        circular_checks: LispConsCircularChecks,
    ) -> Self {
        let errsym = match end_checks {
            LispConsEndChecks::on => Some(ty),
            _ => None,
        };

        Self {
            list,
            tail: list,
            tortoise: list,
            errsym,
            circular_checks,
            max: 2,
            n: 0,
            q: 2,
        }
    }

    pub const fn rest(&self) -> LispObject {
        // This is kind of like Peekable but even when None is returned there
        // might still be a valid item in self.tail.
        self.tail
    }

    // This function must only be called when LispConsCircularCheck is either on or safe.
    fn check_circular(&mut self, cons: LispCons) -> Option<LispCons> {
        self.q = self.q.wrapping_sub(1);
        if self.q != 0 {
            if self.tail == self.tortoise {
                match self.circular_checks {
                    LispConsCircularChecks::on => unsafe { circular_list(self.tail) },
                    _ => return None,
                }
            }
        } else {
            self.n = self.n.wrapping_sub(1);
            if self.n > 0 {
                if self.tail == self.tortoise {
                    match self.circular_checks {
                        LispConsCircularChecks::on => unsafe { circular_list(self.tail) },
                        _ => return None,
                    }
                }
            } else {
                self.max <<= 1;
                self.q = self.max as u16;
                self.n = self.max >> 16;
                self.tortoise = self.tail;
            }
        }

        Some(cons)
    }
}

impl Iterator for TailsIter {
    type Item = LispCons;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tail.as_cons() {
            None => {
                if self.tail.is_not_nil() {
                    if let Some(errsym) = self.errsym {
                        wrong_type!(errsym, self.list);
                    }
                }
                None
            }
            Some(cons) => {
                self.tail = cons.cdr();
                match self.circular_checks {
                    // when off we do not checks at all. When 'safe' the checks are performed
                    // and the iteration exits but no errors are raised.
                    LispConsCircularChecks::off => Some(cons),
                    _ => self.check_circular(cons),
                }
            }
        }
    }
}

pub struct CarIter(TailsIter);

impl CarIter {
    pub const fn new(tails: TailsIter) -> Self {
        Self(tails)
    }

    pub const fn rest(&self) -> LispObject {
        self.0.tail
    }
}

impl Iterator for CarIter {
    type Item = LispObject;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(LispCons::car)
    }
}

impl From<LispObject> for LispCons {
    fn from(o: LispObject) -> Self {
        o.as_cons().unwrap_or_else(|| wrong_type!(Qconsp, o))
    }
}

impl From<LispObject> for Option<LispCons> {
    fn from(o: LispObject) -> Self {
        if o.is_list() {
            Some(LispCons::from(o))
        } else {
            None
        }
    }
}

impl From<LispCons> for LispObject {
    fn from(c: LispCons) -> Self {
        c.0
    }
}

impl<S: Into<LispObject>, T: Into<LispObject>> From<(S, T)> for LispObject {
    fn from(t: (S, T)) -> Self {
        Self::cons(t.0, t.1)
    }
}

impl From<LispCons> for (LispObject, LispObject) {
    fn from(c: LispCons) -> Self {
        (c.car(), c.cdr())
    }
}

impl From<LispObject> for (LispObject, LispObject) {
    fn from(o: LispObject) -> Self {
        LispCons::from(o).into()
    }
}

impl From<LispObject> for Option<(LispObject, LispObject)> {
    fn from(o: LispObject) -> Self {
        if o.is_cons() {
            Some(o.into())
        } else {
            None
        }
    }
}

impl LispCons {
    fn _extract(self) -> *mut Lisp_Cons {
        self.0.get_untaggedptr() as *mut Lisp_Cons
    }

    /// Return the car (first cell).
    pub fn car(self) -> LispObject {
        unsafe { (*self._extract()).u.s.as_ref().car }
    }

    /// Return the cdr (second cell).
    pub fn cdr(self) -> LispObject {
        unsafe { (*self._extract()).u.s.as_ref().u.cdr }
    }

    /// Set the car of the cons cell.
    pub fn set_car(self, n: impl Into<LispObject>) {
        unsafe {
            (*self._extract()).u.s.as_mut().car = n.into();
        }
    }

    /// Set the cdr of the cons cell.
    pub fn set_cdr(self, n: impl Into<LispObject>) {
        unsafe {
            (*self._extract()).u.s.as_mut().u.cdr = n.into();
        }
    }

    /// Check that "self" is an impure (i.e. not readonly) cons cell.
    pub fn check_impure(self) {
        unsafe {
            CHECK_IMPURE(self.0, self._extract() as *mut c_void);
        }
    }

    pub fn length(self) -> usize {
        let len = self
            .0
            .iter_tails(LispConsEndChecks::on, LispConsCircularChecks::on)
            .count();
        if len > MOST_POSITIVE_FIXNUM as usize {
            error!("List too long");
        }
        len
    }

    pub fn iter_tails(
        self,
        end_checks: LispConsEndChecks,
        circular_checks: LispConsCircularChecks,
    ) -> TailsIter {
        TailsIter::new(self.0, Qlistp, end_checks, circular_checks)
    }

    pub fn iter_cars(
        self,
        end_checks: LispConsEndChecks,
        circular_checks: LispConsCircularChecks,
    ) -> CarIter {
        CarIter::new(TailsIter::new(self.0, Qlistp, end_checks, circular_checks))
    }
}
