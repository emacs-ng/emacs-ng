use std::mem::ManuallyDrop;

use font_kit::{
    family_name::FamilyName,
    loaders::default::Font,
    metrics::Metrics,
    properties::{Properties, Style, Weight},
    source::{Source, SystemSource},
};
use lazy_static::lazy_static;

use webrender::api::*;

use super::output::OutputRef;

use emacs::{
    bindings::{
        font, font_driver, font_make_entity, font_make_object, font_metrics, font_property_index,
        font_style_to_value, frame, glyph_string, Fassoc, Fcdr, Fcons, Fmake_symbol, Fnreverse,
        FONT_INVALID_CODE,
    },
    frame::LispFrameRef,
    globals::{
        Qbold, Qextra_bold, Qextra_light, Qitalic, Qlight, Qnil, Qnormal, Qoblique, Qsemi_bold,
        Qultra_bold, Qwr,
    },
    lisp::{ExternalPtr, LispObject},
    multibyte::LispStringRef,
    symbol::LispSymbolRef,
};

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
        font_driver.text_extents = Some(text_extents);
        font_driver.draw = Some(draw);

        FontDriver(font_driver)
    };
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

    fn get_family(&self) -> Option<FamilyName> {
        let tem = self.aref(font_property_index::FONT_FAMILY_INDEX);

        if tem.is_nil() {
            None
        } else {
            let symbol_or_string = tem.as_symbol_or_string();
            let string: LispStringRef = symbol_or_string.into();
            match string.to_string().as_ref() {
                "Serif" => Some(FamilyName::Serif),
                "Sans Serif" => Some(FamilyName::SansSerif),
                "Monospace" => Some(FamilyName::Monospace),
                "Cursive" => Some(FamilyName::Cursive),
                "Fantasy" => Some(FamilyName::Fantasy),
                f => Some(FamilyName::Title(f.to_string().replace("-", "\\-"))),
            }
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
    let output: OutputRef = unsafe { frame.output_data.wr.into() };

    let mut dpyinfo = output.display_info();

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

    let fonts = family
        .and_then(|f| SystemSource::new().select_family_by_generic_name(&f).ok())
        .map(|f| {
            f.fonts()
                .iter()
                .filter_map(|f| f.load().ok())
                .collect::<Vec<Font>>()
        });

    match fonts {
        Some(fonts) => {
            let mut list = Qnil;

            for f in fonts {
                let entity: LispFontLike = unsafe { font_make_entity() }.into();

                // set type
                entity.aset(font_property_index::FONT_TYPE_INDEX, Qwr);

                let family_name: &str = &f.family_name();
                // set family
                entity.aset(font_property_index::FONT_FAMILY_INDEX, unsafe {
                    Fmake_symbol(LispObject::from(family_name))
                });

                let weight = f.properties().weight;

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

                let slant = match f.properties().style {
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

                if let Some(postscript_name) = f.postscript_name() {
                    let postscript_name: &str = &postscript_name;
                    // set name
                    entity.aset(font_property_index::FONT_EXTRA_INDEX, unsafe {
                        Fcons(
                            Fcons(":postscript-name".into(), LispObject::from(postscript_name)),
                            Qnil,
                        )
                    });
                }

                list = unsafe { Fcons(entity.as_lisp_object(), list) }
            }

            unsafe { Fnreverse(list) }
        }
        None => Qnil,
    }
}

extern "C" fn list_family(_f: *mut frame) -> LispObject {
    let mut list = Qnil;

    if let Some(families) = SystemSource::new().all_families().ok() {
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

    // font-kit font
    pub metrics: Metrics,

    pub font_backend: ManuallyDrop<Font>,

    pub font_instance_key: FontInstanceKey,

    pub output: OutputRef,
}

impl WRFont {
    pub fn glyph_for_char(&self, character: char) -> Option<u32> {
        self.font_backend.glyph_for_char(character)
    }

    pub fn get_glyph_advance_width(&self, glyph_indices: Vec<GlyphIndex>) -> Vec<Option<i32>> {
        let font_metrics = self.font_backend.metrics();

        let pixel_size = self.font.pixel_size;

        let scale = pixel_size as f32 / font_metrics.units_per_em as f32;

        glyph_indices
            .into_iter()
            .map(|i| {
                self.font_backend
                    .advance(i)
                    .map(|a| (a.x * scale).round() as i32)
                    .ok()
            })
            .collect()
    }
}

pub type WRFontRef = ExternalPtr<WRFont>;

extern "C" fn open_font(frame: *mut frame, font_entity: LispObject, pixel_size: i32) -> LispObject {
    let font_entity: LispFontLike = font_entity.into();

    let frame: LispFrameRef = frame.into();
    let output: OutputRef = unsafe { frame.output_data.wr.into() };

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

        SystemSource::new()
            .select_best_match(&[family], &Properties::new().style(slant))
            .unwrap()
    } else {
        let postscript_name = unsafe { Fcdr(val) }.as_string().unwrap().to_string();

        // load font by postscript_name.
        // The existing of font has been checked in `match` function.
        SystemSource::new()
            .select_by_postscript_name(&postscript_name)
            .unwrap()
    };

    let mut wr_font = WRFontRef::new(
        font_object
            .as_lisp_object()
            .as_font()
            .unwrap()
            .as_font_mut() as *mut WRFont,
    );

    wr_font.output = output;
    wr_font.font_backend = ManuallyDrop::new(font.load().unwrap());

    // Create font key in webrender.
    let font_key = output.add_font(&font);
    wr_font.font_instance_key = output.add_font_instance(font_key, pixel_size as i32);

    let font_metrics = wr_font.font_backend.metrics();
    let font_advance = wr_font.font_backend.advance(33).unwrap();

    let scale = pixel_size as f32 / font_metrics.units_per_em as f32;

    wr_font.font.pixel_size = pixel_size as i32;
    wr_font.font.average_width = (font_advance.x * scale) as i32;
    wr_font.font.ascent = (scale * font_metrics.ascent).round() as i32;
    wr_font.font.descent = (-scale * font_metrics.descent).round() as i32;
    wr_font.font.space_width = wr_font.font.average_width;
    wr_font.font.max_width = wr_font.font.average_width;
    wr_font.font.underline_thickness = (scale * font_metrics.underline_thickness) as i32;
    wr_font.font.underline_position = (scale * font_metrics.underline_position) as i32;

    wr_font.font.height = (scale * (font_metrics.ascent - font_metrics.descent)).round() as i32;

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
