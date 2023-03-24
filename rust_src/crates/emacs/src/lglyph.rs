use crate::number::LNumber;

use crate::{
    bindings::{
        composition_gstring_from_id, lglyph_indices, make_vector, pvec_type, Fcopy_sequence, ASET,
    },
    definitions::EmacsInt,
    globals::Qnil,
    glyph::GlyphStringRef,
    lisp::LispObject,
    vector::LVector,
};

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
    fn header(self) -> LispObject;
    fn set_header(self, header: LispObject);
    fn font(self) -> LispObject;
    // fn lchar(self, i: u32) -> LispObject;
    // fn char(self, i: u32) -> u32;
    fn char_len(self) -> u32;
    fn shaped_p(self) -> bool;
    fn set_id(self, id: LispObject);
    fn lglyph(self, i: u32) -> LispObject;
    fn lglyph_len(self) -> u32;
    fn set_lglyph(self, i: u32, lglyph: LispObject);
}

impl LGlyphString for LispObject {
    fn is_lgstring(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_NORMAL_VECTOR))
    }
    fn header(self) -> LispObject {
        self.aref(0)
    }

    fn set_header(self, header: LispObject) {
        self.aset(0, header);
    }
    fn font(self) -> LispObject {
        self.header().aref(0)
    }
    // fn lchar(self, i: u32) -> LispObject {
    //     self.header().aref(i + 1)
    // }
    // fn char(self, i: u32) -> u32 {
    //     self.lchar(i).into()
    // }
    fn char_len(self) -> u32 {
        self.header().asize() - 1
    }
    fn shaped_p(self) -> bool {
        self.aref(1).is_t()
    }
    fn set_id(self, id: LispObject) {
        self.aset(1, id)
    }
    fn lglyph(self, i: u32) -> LispObject {
        self.aref(i + 2)
    }
    fn lglyph_len(self) -> u32 {
        self.asize() - 2
    }
    fn set_lglyph(self, i: u32, lglyph: LispObject) {
        self.aset(i + 2, lglyph)
    }
}

pub trait LGlyph {
    fn lfrom(self) -> LispObject;
    fn lto(self) -> LispObject;
    fn lchar(self) -> LispObject;
    fn char(self) -> char;
    fn lcode(self) -> LispObject;
    fn lwidth(self) -> LispObject;
    fn width(self) -> EmacsInt;
    fn llbearing(self) -> LispObject;
    fn lrbearing(self) -> LispObject;
    fn lascent(self) -> LispObject;
    fn ldescent(self) -> LispObject;
    fn adjustment(self) -> LispObject;
    fn set_from_to(self, from: LispObject, to: LispObject);
    fn set_char(self, c: LispObject);
    fn set_code(self, code: LispObject);
    fn set_width(self, width: LispObject);
    fn set_adjustment(self, xoff: LispObject, yoff: LispObject, wadjust: LispObject);
    // Return the shallow Copy of GLYPH.
    fn copy(self) -> Self;
    fn xoff(self) -> EmacsInt;
    fn yoff(self) -> EmacsInt;
    fn wadjust(self) -> EmacsInt;
}

impl LGlyph for LispObject {
    fn lfrom(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_FROM)
    }
    fn lto(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_TO)
    }
    fn lchar(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_CHAR)
    }
    fn char(self) -> char {
        let c_u32: u32 = self.lchar().xfixnum().try_into().unwrap();
        char::from_u32(c_u32).unwrap()
    }
    fn lcode(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_CODE)
    }
    fn lwidth(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_WIDTH)
    }
    fn width(self) -> EmacsInt {
        self.lwidth().xfixnum()
    }
    fn llbearing(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_LBEARING)
    }
    fn lrbearing(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_RBEARING)
    }
    fn lascent(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_ASCENT)
    }
    fn ldescent(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_DESCENT)
    }
    fn adjustment(self) -> LispObject {
        self.aref(lglyph_indices::LGLYPH_IX_ADJUSTMENT)
    }
    fn set_from_to(self, from: LispObject, to: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_FROM, from);
        self.aset(lglyph_indices::LGLYPH_IX_TO, to);
    }
    fn set_char(self, c: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_CHAR, c);
    }
    fn set_code(self, code: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_CODE, code);
    }
    fn set_width(self, width: LispObject) {
        self.aset(lglyph_indices::LGLYPH_IX_WIDTH, width);
    }

    fn set_adjustment(self, xoff: LispObject, yoff: LispObject, wadjust: LispObject) {
        let result = unsafe { make_vector(3, Qnil) };
        unsafe {
            ASET(result, 0, xoff);
            ASET(result, 1, yoff);
            ASET(result, 2, wadjust)
        };
        self.aset(lglyph_indices::LGLYPH_IX_ADJUSTMENT, result);
    }

    fn copy(self) -> Self {
        unsafe { Fcopy_sequence(self) }
    }
    fn xoff(self) -> EmacsInt {
        if self.adjustment().vectorp() {
            return self.adjustment().aref(0).xfixnum();
        }
        0
    }
    fn yoff(self) -> EmacsInt {
        if self.adjustment().vectorp() {
            return self.adjustment().aref(1).xfixnum();
        }
        0
    }
    fn wadjust(self) -> EmacsInt {
        if self.adjustment().vectorp() {
            return self.adjustment().aref(2).xfixnum();
        }
        0
    }
}
