use std::cmp::min;
use std::slice;

use crate::bindings::composition_hash_table;
use crate::bindings::composition_method;
use crate::bindings::glyph;
use crate::bindings::glyph_string;
use crate::bindings::hash_hash_t;
use crate::bindings::hash_lookup_get_hash;
use crate::bindings::XHASH_TABLE;
use crate::definitions::EmacsInt;
use crate::lisp::ExternalPtr;

pub type XChar2b = u32;

pub type GlyphStringRef = ExternalPtr<glyph_string>;

impl GlyphStringRef {
    pub fn get_chars(&self) -> &[XChar2b] {
        let len = self.nchars as usize;

        unsafe { slice::from_raw_parts(self.char2b, len) }
    }

    pub fn first_glyph(&self) -> GlyphRef {
        self.first_glyph.into()
    }

    pub fn composite_offsets(&self) -> &[i16] {
        let len = (self.nchars * 2) as usize;

        let offsets = unsafe { slice::from_raw_parts((*self.cmp).offsets, len) };

        let from = (self.cmp_from * 2) as usize;
        let to = min((self.cmp_to * 2) as usize, len);

        &offsets[from..to]
    }

    pub fn composite_chars(&self) -> &[XChar2b] {
        let from = self.cmp_from as usize;
        let to = min(self.cmp_to, self.nchars) as usize;

        &self.get_chars()[from..to]
    }

    pub fn composite_glyph(&self, n: usize) -> Option<EmacsInt> {
        let n = self.cmp_from as usize + n;

        let hash_table = unsafe { XHASH_TABLE(composition_hash_table) };

        let key_and_value = unsafe { *(*hash_table).key_and_value }.as_vector().unwrap();

        let key = unsafe { (*self.cmp).key };
        let hash_code: Box<hash_hash_t> = Box::new(0);
        let hash_index = unsafe { hash_lookup_get_hash(hash_table, key, Box::into_raw(hash_code)) };

        let composition_index = (hash_index * 2) as usize;
        let composition =
            unsafe { key_and_value.contents.as_slice(composition_index + 1) }[composition_index];
        if let Some(composition) = composition.as_vector() {
            let glyph_index = if unsafe { (*self.cmp).method }
                == composition_method::COMPOSITION_WITH_RULE_ALTCHARS
            {
                n * 2
            } else {
                n
            };

            let glyph = unsafe { composition.contents.as_slice(glyph_index + 1) }[glyph_index];

            Some(glyph.as_fixnum_or_error())
        } else {
            None
        }
    }
}

impl IntoIterator for GlyphStringRef {
    type Item = GlyphStringRef;
    type IntoIter = GlyphStringIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        GlyphStringIntoIterator {
            next_glyph_string: Some(self),
        }
    }
}

pub struct GlyphStringIntoIterator {
    next_glyph_string: Option<GlyphStringRef>,
}

impl Iterator for GlyphStringIntoIterator {
    type Item = GlyphStringRef;

    fn next(&mut self) -> Option<GlyphStringRef> {
        let new_next = self.next_glyph_string.and_then(|n| {
            if n.next.is_null() {
                None
            } else {
                Some(GlyphStringRef::from(n.next))
            }
        });

        let result = self.next_glyph_string;
        self.next_glyph_string = new_next;

        result
    }
}
