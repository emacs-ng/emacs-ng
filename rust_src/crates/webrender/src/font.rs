use std::mem::ManuallyDrop;

use fontdb::{Family, Query, Stretch, Style, Weight};
use lazy_static::lazy_static;

use webrender::api::*;

use emacs::{
    bindings::{
        font, font_driver, font_make_entity, font_make_object, font_metrics, font_property_index,
        font_style_to_value, frame, glyph_string, Fassoc, Fcdr, Fcons, Fmake_symbol, Fnreverse,
        FONT_INVALID_CODE,
    },
    frame::LispFrameRef,
    globals::{
        Qbold, Qextra_bold, Qextra_light, Qiso10646_1, Qitalic, Qlight, Qnil, Qnormal, Qoblique,
        Qsemi_bold, Qultra_bold, Qwr,
    },
    lisp::{ExternalPtr, LispObject},
    multibyte::LispStringRef,
    symbol::LispSymbolRef,
};

use crate::{font_db::FontDB, frame::LispFrameExt, output::OutputRef};

pub type FontRef = ExternalPtr<font>;

pub struct FontDriver(pub font_driver);
unsafe impl Sync for FontDriver {}

lazy_static! {
    pub static ref FONT_DRIVER: FontDriver = {
        let mut font_driver = font_driver::default();

        font_driver.type_ = Qwr;
        font_driver.case_sensitive = true;
        font_driver.get_cache = Some(get_cache);
        font_driver.list = Some(list);
        font_driver.match_ = Some(match_);
        font_driver.list_family = Some(list_family);
        font_driver.open_font = Some(open_font);
        font_driver.close_font = Some(close_font);
        font_driver.encode_char = Some(encode_char);
        font_driver.has_char = Some(has_char);
        font_driver.text_extents = Some(text_extents);
        font_driver.draw = Some(draw);

        FontDriver(font_driver)
    };
    static ref FONT_DB: FontDB = FontDB::new();
}

/// A newtype for objects we know are font_spec.
#[derive(Clone, Copy)]
pub struct LispFontLike(LispObject);

impl LispFontLike {
    fn aref(&self, index: font_property_index::Type) -> LispObject {
        let vl = self.0.as_vectorlike().unwrap();
        let v = unsafe { vl.as_vector_unchecked() };
        unsafe { v.get_unchecked(index as usize) }
    }

    fn get_family(&self) -> Option<String> {
        let tem = self.aref(font_property_index::FONT_FAMILY_INDEX);

        if tem.is_nil() {
            None
        } else {
            let symbol_or_string = tem.as_symbol_or_string();
            let string: LispStringRef = symbol_or_string.into();
            return Some(string.to_string().replace("-", "\\-"));
        }
    }

    fn get_slant(&self) -> Option<Style> {
        let slant = self.aref(font_property_index::FONT_SLANT_INDEX);

        if slant.is_nil() {
            None
        } else {
            let symbol_or_string = slant.as_symbol_or_string();
            let string: LispStringRef = symbol_or_string.into();
            match string.to_string().as_ref() {
                "Qnormal" => Some(Style::Normal),
                "Qitalic" => Some(Style::Italic),
                "Qoblique" => Some(Style::Oblique),
                _ => Some(Style::Normal),
            }
        }
    }

    fn aset(&self, index: font_property_index::Type, val: LispObject) {
        let vl = self.0.as_vectorlike().unwrap();
        let mut v = unsafe { vl.as_vector_unchecked() };
        unsafe { v.set_unchecked(index as usize, val) };
    }

    fn set_style(&self, index: font_property_index::Type, val: LispObject) {
        let value = unsafe { font_style_to_value(index, val, true) };

        self.aset(index, LispObject::from(value));
    }

    fn as_lisp_object(self) -> LispObject {
        self.0
    }
}

impl From<LispObject> for LispFontLike {
    fn from(v: LispObject) -> LispFontLike {
        LispFontLike(v)
    }
}

extern "C" fn get_cache(f: *mut frame) -> LispObject {
    let frame = LispFrameRef::new(f);
    let mut dpyinfo = frame.wr_display_info();

    dpyinfo.get_raw().name_list_element
}

extern "C" fn draw(
    _s: *mut glyph_string,
    _from: i32,
    _to: i32,
    _x: i32,
    _y: i32,
    _with_background: bool,
) -> i32 {
    0
}

extern "C" fn list(frame: *mut frame, font_spec: LispObject) -> LispObject {
    // FIXME: implment the real list in future
    match_(frame, font_spec)
}

extern "C" fn match_(_f: *mut frame, spec: LispObject) -> LispObject {
    let font_spec = LispFontLike(spec);
    let family = font_spec.get_family();

    let fonts = if let Some(family) = family {
        let family = match family.as_ref() {
            "Serif" => Family::Serif,
            "Sans Serif" => Family::SansSerif,
            "Monospace" => Family::Monospace,
            "Cursive" => Family::Cursive,
            "Fantasy" => Family::Fantasy,
            f => Family::Name(f),
        };

        FONT_DB.select_family(&family)
    } else {
        FONT_DB.all_fonts()
    };

    let mut list = Qnil;

    for f in fonts {
        let entity: LispFontLike = unsafe { font_make_entity() }.into();

        // set type
        entity.aset(font_property_index::FONT_TYPE_INDEX, Qwr);

        let family_name: &str = &f.family;
        // set family
        entity.aset(font_property_index::FONT_FAMILY_INDEX, unsafe {
            Fmake_symbol(LispObject::from(family_name))
        });

        let weight = f.weight;

        let weight = if weight <= Weight::EXTRA_LIGHT {
            Qextra_light
        } else if weight <= Weight::LIGHT {
            Qlight
        } else if weight <= Weight::NORMAL {
            Qnormal
        } else if weight <= Weight::MEDIUM {
            Qsemi_bold
        } else if weight <= Weight::SEMIBOLD {
            Qsemi_bold
        } else if weight <= Weight::BOLD {
            Qbold
        } else if weight <= Weight::EXTRA_BOLD {
            Qextra_bold
        } else if weight <= Weight::BLACK {
            Qultra_bold
        } else {
            Qultra_bold
        };

        // set weight
        entity.set_style(font_property_index::FONT_WEIGHT_INDEX, weight);

        let slant = match f.style {
            Style::Normal => Qnormal,
            Style::Italic => Qitalic,
            Style::Oblique => Qoblique,
        };

        // set slant
        entity.set_style(font_property_index::FONT_SLANT_INDEX, slant);

        // set width
        entity.set_style(font_property_index::FONT_WIDTH_INDEX, (0 as usize).into());

        // set size
        entity.aset(font_property_index::FONT_SIZE_INDEX, (0 as usize).into());

        // set registry
        entity.aset(font_property_index::FONT_REGISTRY_INDEX, Qiso10646_1);

        let postscript_name: &str = &f.post_script_name;
        // set name
        entity.aset(font_property_index::FONT_EXTRA_INDEX, unsafe {
            Fcons(
                Fcons(":postscript-name".into(), LispObject::from(postscript_name)),
                Qnil,
            )
        });

        list = unsafe { Fcons(entity.as_lisp_object(), list) }
    }

    unsafe { Fnreverse(list) }
}

extern "C" fn list_family(_f: *mut frame) -> LispObject {
    let mut list = Qnil;

    if let Some(families) = FONT_DB.all_families() {
        for f in families {
            list = LispObject::cons(LispObject::from(&f as &str), list);
        }
    }

    list
}

#[repr(C)]
pub struct WRFont {
    // extend basic font
    pub font: font,

    pub font_id: fontdb::ID,

    pub font_instance_key: FontInstanceKey,

    pub font_bytes: ManuallyDrop<Vec<u8>>,

    pub font_index: u32,

    pub output: OutputRef,
}

impl WRFont {
    pub fn glyph_for_char(&self, character: char) -> Option<u32> {
        let face = ttf_parser::Face::from_slice(&self.font_bytes, self.font_index).ok()?;

        face.glyph_index(character).map(|c| c.0 as u32)
    }

    pub fn get_glyph_advance_width(&self, glyph_indices: Vec<GlyphIndex>) -> Vec<Option<i32>> {
        let face = ttf_parser::Face::from_slice(&self.font_bytes, self.font_index);

        if face.is_err() {
            return Vec::new();
        }

        let face = face.unwrap();

        let pixel_size = self.font.pixel_size;
        let units_per_em = face.units_per_em().unwrap();

        let scale = pixel_size as f32 / units_per_em as f32;

        glyph_indices
            .into_iter()
            .map(|i| {
                face.glyph_hor_advance(ttf_parser::GlyphId(i as u16))
                    .map(|a| (a as f32 * scale).round() as i32)
            })
            .collect()
    }
}

pub type WRFontRef = ExternalPtr<WRFont>;

extern "C" fn open_font(frame: *mut frame, font_entity: LispObject, pixel_size: i32) -> LispObject {
    let font_entity: LispFontLike = font_entity.into();

    let frame: LispFrameRef = frame.into();
    let mut output = frame.wr_output();

    let mut pixel_size = pixel_size as i64;

    if pixel_size == 0 {
        pixel_size = if !output.font.is_null() {
            output.font.pixel_size as i64
        } else {
            15
        };
    }

    let font_object: LispFontLike = unsafe {
        font_make_object(
            vecsize!(WRFont) as i32,
            font_entity.as_lisp_object(),
            pixel_size as i32,
        )
    }
    .into();

    // set type
    font_object.aset(font_property_index::FONT_TYPE_INDEX, Qwr);

    // set name
    font_object.aset(
        font_property_index::FONT_NAME_INDEX,
        LispSymbolRef::from(font_entity.aref(font_property_index::FONT_FAMILY_INDEX)).symbol_name(),
    );

    // Get postscript name form font_entity.
    let font_extra = font_entity.aref(font_property_index::FONT_EXTRA_INDEX);

    let val = unsafe { Fassoc(":postscript-name".into(), font_extra, Qnil) };

    let font = if val.is_nil() {
        let family = font_entity.get_family().unwrap();

        let slant = font_entity.get_slant().unwrap();

        let family = match family.as_ref() {
            "Serif" => Family::Serif,
            "Sans Serif" => Family::SansSerif,
            "Monospace" => Family::Monospace,
            "Cursive" => Family::Cursive,
            "Fantasy" => Family::Fantasy,
            f => Family::Name(f),
        };

        FONT_DB
            .query(&Query {
                families: &[family],
                stretch: Stretch::Normal,
                weight: Weight::NORMAL,
                style: slant,
            })
            .unwrap()
    } else {
        let postscript_name = unsafe { Fcdr(val) }.as_string().unwrap().to_string();
        FONT_DB.select_postscript(&postscript_name).unwrap()
    };

    let mut wr_font = WRFontRef::new(
        font_object
            .as_lisp_object()
            .as_font()
            .unwrap()
            .as_font_mut() as *mut WRFont,
    );

    wr_font.output = output;
    wr_font.font_id = font.id;

    // Create font key in webrender.
    let font_key = output.add_font(font);
    wr_font.font_instance_key = output.add_font_instance(font_key, pixel_size as i32);

    FONT_DB
        .db
        .with_face_data(font.id, |font_data, face_index| {
            wr_font.font_bytes = ManuallyDrop::new(font_data.to_vec());
            wr_font.font_index = face_index;

            let face = ttf_parser::Face::from_slice(font_data, face_index).unwrap();

            let units_per_em = face.units_per_em().unwrap();

            let underline_metrics = face.underline_metrics().unwrap();

            let ascent = face.ascender();
            let descent = face.descender();

            let average_width = face.glyph_hor_advance(ttf_parser::GlyphId(0)).unwrap();

            let scale = pixel_size as f32 / units_per_em as f32;

            wr_font.font.pixel_size = pixel_size as i32;
            wr_font.font.average_width = (average_width as f32 * scale) as i32;
            wr_font.font.ascent = (scale * ascent as f32).round() as i32;
            wr_font.font.descent = (-scale * descent as f32).round() as i32;
            wr_font.font.space_width = wr_font.font.average_width;
            wr_font.font.max_width = wr_font.font.average_width;
            wr_font.font.underline_thickness = (scale * underline_metrics.thickness as f32) as i32;
            wr_font.font.underline_position = (scale * underline_metrics.position as f32) as i32;

            wr_font.font.height = (scale * (ascent - descent) as f32).round() as i32;
        })
        .unwrap();

    wr_font.font.baseline_offset = 0;

    wr_font.font.driver = &FONT_DRIVER.0;

    font_object.as_lisp_object()
}

extern "C" fn close_font(_font: *mut font) {}

extern "C" fn encode_char(font: *mut font, c: i32) -> u32 {
    let font = WRFontRef::new(font as *mut WRFont);

    std::char::from_u32(c as u32)
        .and_then(|c| font.glyph_for_char(c))
        .unwrap_or(FONT_INVALID_CODE)
}

extern "C" fn has_char(font: LispObject, c: i32) -> i32 {
    if font.is_font_entity() {
        let font_entity: LispFontLike = font.into();

        // Get postscript name form font_entity.
        let font_extra = font_entity.aref(font_property_index::FONT_EXTRA_INDEX);

        let val = unsafe { Fassoc(":postscript-name".into(), font_extra, Qnil) };

        if val.is_nil() {
            return -1;
        }

        let postscript_name = unsafe { Fcdr(val) }.as_string().unwrap().to_string();

        let c = std::char::from_u32(c as u32);

        if c.is_none() {
            return 0;
        }

        let c = c.unwrap();

        FONT_DB
            .select_postscript(&postscript_name)
            .and_then(|font| {
                FONT_DB
                    .db
                    .with_face_data(font.id, |font_data, face_index| {
                        ttf_parser::Face::from_slice(font_data, face_index)
                            .ok()
                            .and_then(|face| face.glyph_index(c))
                    })
                    .flatten()
            })
            .is_some() as i32
    } else {
        let font = font.as_font().unwrap().as_font_mut();

        (encode_char(font, c) != FONT_INVALID_CODE) as i32
    }
}

#[allow(unused_variables)]
extern "C" fn text_extents(
    font: *mut font,
    code: *const u32,
    nglyphs: i32,
    metrics: *mut font_metrics,
) {
    let font = WRFontRef::new(font as *mut WRFont);

    let glyph_indices: Vec<u32> = unsafe { std::slice::from_raw_parts(code, nglyphs as usize) }
        .iter()
        .copied()
        .collect();

    let width: i32 = font
        .get_glyph_advance_width(glyph_indices.clone())
        .into_iter()
        .filter_map(|w| w)
        .sum();

    unsafe {
        (*metrics).lbearing = 0;
        (*metrics).rbearing = width as i16;
        (*metrics).width = width as i16;
        (*metrics).ascent = font.font.ascent as i16;
        (*metrics).descent = font.font.descent as i16;
    }
}
