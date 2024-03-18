use crate::number::LNumber;

use crate::bindings::composition_gstring_from_id;
use crate::bindings::lglyph_indices;
use crate::bindings::make_nil_vector;
use crate::bindings::make_vector;
use crate::bindings::pvec_type;
use crate::bindings::Fcopy_sequence;
use crate::bindings::ASET;
use crate::definitions::EmacsInt;
use crate::display_traits::GlyphStringRef;
use crate::globals::Qnil;
use crate::lisp::LispObject;
use crate::vector::LVector;

impl GlyphStringRef {
    pub fn is_automatic_composition(&self) -> bool {
        unsafe { (*self.first_glyph).u.cmp.automatic() }
    }
    pub fn get_lgstring(&self) -> LispObject {
        if !self.is_automatic_composition() {
            return Qnil;
        }
        unsafe { composition_gstring_from_id(self.cmp_id) }
    }
}

pub trait LGlyphString {
    fn is_lgstring(self) -> bool;
    fn lgstring_header(self) -> LispObject;
    fn set_lgstring_header(self, header: LispObject);
    fn lgstring_font(self) -> LispObject;
    fn lgstring_char(self, i: u32) -> LispObject;
    fn lgstring_char_len(self) -> u32;
    fn lgstring_shaped_p(self) -> bool;
    fn set_lgstring_id(self, id: LispObject);
    fn lgstring_glyph(self, i: u32) -> LispObject;
    fn lgstring_glyph_len(self) -> u32;
    fn set_lgstring_glyph(self, i: u32, lglyph: LispObject);
}

impl LGlyphString for LispObject {
    fn is_lgstring(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_NORMAL_VECTOR))
    }
    fn lgstring_header(self) -> LispObject {
        self.aref(0)
    }

    fn set_lgstring_header(self, header: LispObject) {
        self.aset(0, header);
    }
    fn lgstring_font(self) -> LispObject {
        self.lgstring_header().aref(0)
    }
    fn lgstring_char(self, i: u32) -> LispObject {
        self.lgstring_header().aref(i + 1)
    }
    fn lgstring_char_len(self) -> u32 {
        self.lgstring_header().asize() - 1
    }
    fn lgstring_shaped_p(self) -> bool {
        self.aref(1).is_t()
    }
    fn set_lgstring_id(self, id: LispObject) {
        self.aset(1, id)
    }
    fn lgstring_glyph(self, i: u32) -> LispObject {
        self.aref(i + 2)
    }
    fn lgstring_glyph_len(self) -> u32 {
        self.asize() - 2
    }
    fn set_lgstring_glyph(self, i: u32, lglyph: LispObject) {
        self.aset(i + 2, lglyph)
    }
}

pub trait LGlyph {
    fn lglyph_new() -> LispObject;
    fn lglyph_from(self) -> LispObject;
    fn lglyph_to(self) -> LispObject;
    fn lglyph_char(self) -> LispObject;
    fn lglyph_code(self) -> LispObject;
    fn lglyph_width(self) -> LispObject;
    fn lglyph_lbearing(self) -> LispObject;
    fn lglyph_rbearing(self) -> LispObject;
    fn lglyph_ascent(self) -> LispObject;
    fn lglyph_descent(self) -> LispObject;
    fn lglyph_adjustment(self) -> LispObject;
    fn set_lglyph_from_to(self, from: LispObject, to: LispObject);
    fn set_lglyph_char(self, c: LispObject);
    fn set_lglyph_code(self, code: LispObject);
    fn set_lglyph_width(self, width: LispObject);
    fn set_lglyph_lbearing(self, val: LispObject);
    fn set_lglyph_rbearing(self, val: LispObject);
    fn set_lglyph_descent(self, val: LispObject);
    fn set_lglyph_ascent(self, val: LispObject);
    fn set_lglyph_adjustment(self, xoff: LispObject, yoff: LispObject, wadjust: LispObject);
    // Return the shallow Copy of GLYPH.
    fn lglyph_copy(self) -> Self;
    fn lglyph_xoff(self) -> EmacsInt;
    fn lglyph_yoff(self) -> EmacsInt;
    fn lglyph_wadjust(self) -> EmacsInt;
}

impl LGlyph for LispObject {
    fn lglyph_new() -> LispObject {
        unsafe { make_nil_vector(lglyph_indices::LGLYPH_SIZE.try_into().unwrap()) }
    }
    fn lglyph_from(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_FROM)
    }
    fn lglyph_to(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_TO)
    }
    fn lglyph_char(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_CHAR)
    }
    // fn lglyph_char(self) -> char {
    //     let c_u32: u32 = self.lchar().xfixnum().try_into().unwrap();
    //     char::from_u32(c_u32).unwrap()
    // }
    fn lglyph_code(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_CODE)
    }
    fn lglyph_width(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_WIDTH)
    }
    fn lglyph_lbearing(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_LBEARING)
    }
    fn lglyph_rbearing(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_RBEARING)
    }
    fn lglyph_ascent(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_ASCENT)
    }
    fn lglyph_descent(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_DESCENT)
    }
    fn lglyph_adjustment(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_ADJUSTMENT)
    }
    fn set_lglyph_from_to(self, from: LispObject, to: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_FROM, from);
        self.aset(lglyph_indices::LGLYPH_IX_TO, to);
    }
    fn set_lglyph_char(self, c: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_CHAR, c);
    }
    fn set_lglyph_code(self, code: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_CODE, code);
    }
    fn set_lglyph_width(self, width: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_WIDTH, width);
    }
    fn set_lglyph_lbearing(self, val: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_LBEARING, val);
    }
    fn set_lglyph_rbearing(self, val: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_RBEARING, val);
    }
    fn set_lglyph_descent(self, val: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_DESCENT, val);
    }
    fn set_lglyph_ascent(self, val: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_ASCENT, val);
    }
    fn set_lglyph_adjustment(self, xoff: LispObject, yoff: LispObject, wadjust: LispObject) {
        let result = unsafe { make_vector(3, Qnil) };
        unsafe {
            ASET(result, 0, xoff);
            ASET(result, 1, yoff);
            ASET(result, 2, wadjust)
        };
        self.aset(lglyph_indices::LGLYPH_IX_ADJUSTMENT, result);
    }

    fn lglyph_copy(self) -> Self {
        unsafe { Fcopy_sequence(self) }
    }
    fn lglyph_xoff(self) -> EmacsInt {
        if self.lglyph_adjustment().vectorp() {
            return self.lglyph_adjustment().aref(0).xfixnum();
        }
        0
    }
    fn lglyph_yoff(self) -> EmacsInt {
        if self.lglyph_adjustment().vectorp() {
            return self.lglyph_adjustment().aref(1).xfixnum();
        }
        0
    }
    fn lglyph_wadjust(self) -> EmacsInt {
        if self.lglyph_adjustment().vectorp() {
            return self.lglyph_adjustment().aref(2).xfixnum();
        }
        0
    }
}
