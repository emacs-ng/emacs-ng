use crate::number::LNumber;

use crate::bindings::lglyph_indices;
use crate::bindings::make_nil_vector;
use crate::bindings::make_vector;
use crate::bindings::pvec_type;
use crate::bindings::Fcopy_sequence;
use crate::bindings::ASET;
use crate::definitions::EmacsInt;
use crate::globals::Qnil;
use crate::lisp::LispObject;
use crate::vector::LVector;

impl LispObject {
    pub fn is_lgstring(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_NORMAL_VECTOR))
    }
    pub fn lgstring_header(self) -> LispObject {
        self.aref(0)
    }

    pub fn set_lgstring_header(self, header: impl Into<LispObject>) {
        self.aset(0, header.into());
    }
    pub fn lgstring_font(self) -> LispObject {
        self.lgstring_header().aref(0)
    }
    pub fn lgstring_char(self, i: u32) -> LispObject {
        self.lgstring_header().aref(i + 1)
    }
    pub fn lgstring_char_len(self) -> u32 {
        self.lgstring_header().asize() - 1
    }
    pub fn lgstring_shaped_p(self) -> bool {
        self.aref(1).is_t()
    }
    pub fn set_lgstring_id(self, id: impl Into<LispObject>) {
        self.aset(1, id.into())
    }
    pub fn lgstring_glyph(self, i: u32) -> LispObject {
        self.aref(i + 2)
    }
    pub fn lgstring_glyph_len(self) -> u32 {
        self.asize() - 2
    }
    pub fn set_lgstring_glyph(self, i: u32, lglyph: impl Into<LispObject>) {
        self.aset(i + 2, lglyph.into())
    }
}

impl LispObject {
    pub fn lglyph_new() -> LispObject {
        unsafe { make_nil_vector(lglyph_indices::LGLYPH_SIZE.try_into().unwrap()) }
    }
    pub fn lglyph_from(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_FROM)
    }
    pub fn lglyph_to(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_TO)
    }
    pub fn lglyph_char(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_CHAR)
    }
    // pub fn lglyph_char(self) -> char {
    //     let c_u32: u32 = self.lchar().xfixnum().try_into().unwrap();
    //     char::from_u32(c_u32).unwrap()
    // }
    pub fn lglyph_code(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_CODE)
    }
    pub fn lglyph_width(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_WIDTH)
    }
    pub fn lglyph_lbearing(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_LBEARING)
    }
    pub fn lglyph_rbearing(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_RBEARING)
    }
    pub fn lglyph_ascent(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_ASCENT)
    }
    pub fn lglyph_descent(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_DESCENT)
    }
    pub fn lglyph_adjustment(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_ADJUSTMENT)
    }
    pub fn set_lglyph_from_to(self, from: impl Into<LispObject>, to: impl Into<LispObject>) {
        self.aset(lglyph_indices::LGLYPH_IX_FROM, from.into());
        self.aset(lglyph_indices::LGLYPH_IX_TO, to.into());
    }
    pub fn set_lglyph_char(self, c: impl Into<LispObject>) {
        self.aset(lglyph_indices::LGLYPH_IX_CHAR, c.into());
    }
    pub fn set_lglyph_code(self, code: impl Into<LispObject>) {
        self.aset(lglyph_indices::LGLYPH_IX_CODE, code.into());
    }
    pub fn set_lglyph_width(self, width: impl Into<LispObject>) {
        self.aset(lglyph_indices::LGLYPH_IX_WIDTH, width.into());
    }
    pub fn set_lglyph_lbearing(self, val: impl Into<LispObject>) {
        self.aset(lglyph_indices::LGLYPH_IX_LBEARING, val.into());
    }
    pub fn set_lglyph_rbearing(self, val: impl Into<LispObject>) {
        self.aset(lglyph_indices::LGLYPH_IX_RBEARING, val.into());
    }
    pub fn set_lglyph_descent(self, val: impl Into<LispObject>) {
        self.aset(lglyph_indices::LGLYPH_IX_DESCENT, val.into());
    }
    pub fn set_lglyph_ascent(self, val: impl Into<LispObject>) {
        self.aset(lglyph_indices::LGLYPH_IX_ASCENT, val.into());
    }
    pub fn set_lglyph_adjustment(
        self,
        xoff: impl Into<LispObject>,
        yoff: impl Into<LispObject>,
        wadjust: impl Into<LispObject>,
    ) {
        let result = unsafe { make_vector(3, Qnil) };
        unsafe {
            ASET(result, 0, xoff.into());
            ASET(result, 1, yoff.into());
            ASET(result, 2, wadjust.into())
        };
        self.aset(lglyph_indices::LGLYPH_IX_ADJUSTMENT, result);
    }

    pub fn lglyph_copy(self) -> Self {
        unsafe { Fcopy_sequence(self) }
    }
    pub fn lglyph_xoff(self) -> EmacsInt {
        if self.lglyph_adjustment().vectorp() {
            return self.lglyph_adjustment().aref(0).xfixnum();
        }
        0
    }
    pub fn lglyph_yoff(self) -> EmacsInt {
        if self.lglyph_adjustment().vectorp() {
            return self.lglyph_adjustment().aref(1).xfixnum();
        }
        0
    }
    pub fn lglyph_wadjust(self) -> EmacsInt {
        if self.lglyph_adjustment().vectorp() {
            return self.lglyph_adjustment().aref(2).xfixnum();
        }
        0
    }
}
