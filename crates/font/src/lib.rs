#![feature(concat_idents)]
#![feature(lazy_cell)]

#[macro_use]
extern crate emacs_sys;
extern crate lisp_macros;
#[macro_use]
extern crate lisp_util;

mod cache;
mod fns;

use emacs_sys::bindings::assq_no_quit;
use emacs_sys::bindings::font_put_extra;
use emacs_sys::bindings::globals;
use emacs_sys::bindings::make_fixnum;
use emacs_sys::bindings::AREF;
use emacs_sys::bindings::CONSP;
use emacs_sys::bindings::SYMBOLP;
use emacs_sys::bindings::XCAR;
use emacs_sys::bindings::XCDR;
use emacs_sys::bindings::XFIXNUM;
use emacs_sys::globals::QCfont_entity;
use emacs_sys::globals::QClang;
use emacs_sys::globals::QCotf;
use emacs_sys::globals::QCscript;
use emacs_sys::globals::Qdefault;
use emacs_sys::globals::Qfixed;
use emacs_sys::globals::Qmonospace;
use font_index::Font;
use font_index::FontEntry;
pub use font_index::FontId;
use font_index::FontIndex;
use std::mem::ManuallyDrop;
use std::sync::LazyLock;
use std::sync::Mutex;
use swash::shape::ShapeContext;
use swash::text::Language;
use swash::text::Script;
use swash::Attributes;
use swash::ObliqueAngle;
use swash::Stretch;
use swash::Style;
use swash::Weight;

use std::ptr;

use crate::cache::TerminalExtFontIndex;
use webrender_api::GlyphIndex;

use emacs_sys::bindings::font;
use emacs_sys::bindings::font_driver;
use emacs_sys::bindings::font_make_entity;
use emacs_sys::bindings::font_make_object;
use emacs_sys::bindings::font_metrics;
use emacs_sys::bindings::font_property_index;
use emacs_sys::bindings::font_style_to_value;
use emacs_sys::bindings::frame;
use emacs_sys::bindings::glyph_string;
use emacs_sys::bindings::intern;
use emacs_sys::bindings::register_font_driver;
use emacs_sys::bindings::Fcons;
use emacs_sys::bindings::Fmake_symbol;
use emacs_sys::bindings::Fnreverse;
use emacs_sys::bindings::FONT_INVALID_CODE;
use emacs_sys::frame::FrameRef;
use emacs_sys::globals::Qiso10646_1;
use emacs_sys::globals::Qnil;
use emacs_sys::globals::Qswash;
use emacs_sys::lisp::ExternalPtr;
use emacs_sys::lisp::LispObject;
use emacs_sys::symbol::LispSymbolRef;

pub type FontRef = ExternalPtr<font>;

pub struct FontDriver(pub font_driver);
unsafe impl Sync for FontDriver {}

static FONT_DRIVER: LazyLock<FontDriver> = LazyLock::new(|| {
    log::trace!("FONT_DRIVER is being created...");
    let mut font_driver = font_driver::default();

    font_driver.type_ = Qswash;
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
    font_driver.shape = Some(shape);

    FontDriver(font_driver)
});
//shapecontext per thread
static SHAPE_CONTEXT: LazyLock<Mutex<ShapeContext>> =
    LazyLock::new(|| Mutex::new(ShapeContext::new()));
impl FontDriver {
    fn global() -> &'static FontDriver {
        &FONT_DRIVER
    }
}

/// A newtype for objects we know are font_spec.
#[derive(Clone, Copy)]
pub struct LispFontLike(LispObject);
pub struct LispFontWeight(LispObject);
pub struct LispFontStyle(LispObject);

impl LispFontLike {
    fn aref(&self, index: font_property_index::Type) -> LispObject {
        let vl = self.0.as_vectorlike().unwrap();
        let v = unsafe { vl.as_vector_unchecked() };
        unsafe { v.get_unchecked(index as usize) }
    }

    fn family_name(&self) -> Option<String> {
        let mut tem = self.aref(font_property_index::FONT_FAMILY_INDEX);

        if tem.is_nil() {
            None
        } else {
            if tem.eq(Qmonospace) || tem.eq(Qfixed) || tem.eq(Qdefault) {
                tem = Qmonospace
            }
            let string: String = tem.into();
            Some(string.to_string().replace("-", "\\-"))
        }
    }

    fn aset(&self, index: font_property_index::Type, val: LispObject) {
        let vl = self.0.as_vectorlike().unwrap();
        let mut v = unsafe { vl.as_vector_unchecked() };
        unsafe { v.set_unchecked(index as usize, val) };
    }

    fn set_property(&self, index: font_property_index::Type, val: LispObject) {
        let value = unsafe { font_style_to_value(index, val, true) };

        self.aset(index, LispObject::from(value));
    }

    fn as_lisp_object(self) -> LispObject {
        self.0
    }
}

pub trait LispFont {
    fn from(font: FontEntry<'_>) -> Self;
}

impl From<LispObject> for LispFontLike {
    fn from(v: LispObject) -> LispFontLike {
        LispFontLike(v)
    }
}
impl From<LispObject> for LispFontWeight {
    fn from(v: LispObject) -> LispFontWeight {
        LispFontWeight(v)
    }
}
impl From<LispObject> for LispFontStyle {
    fn from(v: LispObject) -> LispFontStyle {
        LispFontStyle(v)
    }
}

pub type Slant = Option<Style>;

impl From<LispFontStyle> for Slant {
    fn from(v: LispFontStyle) -> Slant {
        if v.0.is_nil() {
            return Some(Style::Normal);
        }
        let slant = unsafe { XFIXNUM(v.0) };
        match slant {
            100 => Some(Style::Normal),
            200 => Some(Style::Italic),
            210 => Some(Style::Oblique(ObliqueAngle::from_degrees(14.))),
            _ => {
                log::error!("Swash doesn't support reverse-italic/oblique.");
                None
            }
        }
    }
}

impl From<LispFontWeight> for Weight {
    fn from(v: LispFontWeight) -> Weight {
        if v.0.is_nil() {
            return Weight::NORMAL;
        }

        let weight = unsafe { XFIXNUM(v.0) };
        // per font-weight-table
        if weight == 0 {
            Weight::THIN
        } else if weight <= 40 {
            Weight::EXTRA_LIGHT
        } else if weight <= 50 {
            Weight::LIGHT
        } else if weight <= 80 {
            Weight::NORMAL
        } else if weight <= 100 {
            Weight::MEDIUM
        } else if weight <= 180 {
            Weight::SEMI_BOLD
        } else if weight <= 200 {
            Weight::BOLD
        } else if weight <= 205 {
            Weight::EXTRA_BOLD
        } else if weight <= 210 {
            Weight::BLACK
        } else {
            Weight::BLACK
        }
    }
}

impl Into<LispFontStyle> for Style {
    fn into(self) -> LispFontStyle {
        let slant = match self {
            Style::Normal => 100,
            Style::Italic => 200,
            Style::Oblique(_) => 210,
        };
        LispFontStyle(unsafe { make_fixnum(slant) })
    }
}

impl Into<LispFontWeight> for Weight {
    fn into(self) -> LispFontWeight {
        // per font-weight-table
        let weight = if self.0 <= Weight::THIN.0 {
            0
        } else if self.0 <= Weight::EXTRA_LIGHT.0 {
            40
        } else if self.0 <= Weight::LIGHT.0 {
            50
        } else if self.0 <= Weight::NORMAL.0 {
            80
        } else if self.0 <= Weight::MEDIUM.0 {
            100
        } else if self.0 <= Weight::SEMI_BOLD.0 {
            180
        } else if self.0 <= Weight::BOLD.0 {
            200
        } else if self.0 <= Weight::EXTRA_BOLD.0 {
            205
        } else if self.0 <= Weight::BLACK.0 {
            210
        } else {
            250
        };

        let weight = unsafe { make_fixnum(weight) };
        LispFontWeight(weight)
    }
}

pub type OptionalAttributes = Option<Attributes>;

impl From<LispFontLike> for OptionalAttributes {
    fn from(spec_or_entity: LispFontLike) -> OptionalAttributes {
        let slant = Slant::from(spec_or_entity);
        let weight = Weight::from(spec_or_entity);

        let stretch = Stretch::NORMAL;
        if let Some(style) = slant {
            return Some(Attributes::new(stretch, weight, style));
        }

        None
    }
}

impl From<LispFontLike> for Slant {
    fn from(spec_or_entity: LispFontLike) -> Slant {
        let slant = spec_or_entity.aref(font_property_index::FONT_SLANT_INDEX);
        Slant::from(LispFontStyle(slant))
    }
}

impl From<LispFontLike> for Weight {
    fn from(spec_or_entity: LispFontLike) -> Weight {
        let weight = spec_or_entity.aref(font_property_index::FONT_WIDTH_INDEX);
        Weight::from(LispFontWeight(weight))
    }
}

pub type OptionalScript = Option<Script>;

impl From<LispFontLike> for OptionalScript {
    fn from(spec_or_entity: LispFontLike) -> OptionalScript {
        let find_otf_script = || -> Option<String> {
            let mut extra = spec_or_entity.aref(font_property_index::FONT_EXTRA_INDEX);

            unsafe {
                while CONSP(extra) {
                    let tmp = XCAR(extra);
                    if tmp.is_cons() {
                        let key = XCAR(tmp);
                        let val = XCDR(tmp);
                        if key.eq(QCscript) && SYMBOLP(val) {
                            let otf_script = Fscript_to_otf(val);
                            if otf_script.is_not_nil() {
                                return Some(otf_script.into());
                            }
                        }
                        if key.eq(QClang) && SYMBOLP(val) {
                            let lang: String = val.into();
                            if let Some(lang) = Language::parse(lang.as_str()) {
                                if let Some(otf_script) = lang.script() {
                                    return Some(otf_script.to_string());
                                }
                            }
                        }
                        if key.eq(QCotf) && CONSP(val) && SYMBOLP(XCAR(val)) {
                            let otf_script = XCAR(val);
                            return Some(otf_script.into());
                        }
                    }
                    extra = XCDR(extra);
                }
            }

            let reg = spec_or_entity.aref(font_property_index::FONT_REGISTRY_INDEX);
            if reg.is_symbol() {
                let script = Fregistry_to_script(reg);
                let otf_script = Fscript_to_otf(script);
                return Some(otf_script.into());
            }

            return None;
        };
        if let Some(otf_script) = find_otf_script() {
            return Script::from_opentype(swash::tag_from_str_lossy(otf_script.as_str()));
        }

        return None;
    }
}

impl From<FontEntry<'_>> for LispFontLike {
    fn from(font: FontEntry<'_>) -> LispFontLike {
        let entity: LispFontLike = unsafe { font_make_entity() }.into();

        // set type
        entity.aset(font_property_index::FONT_TYPE_INDEX, Qswash);

        // set family
        let family_name: LispObject = font.family_name().to_string().into();
        let family_name = unsafe { Fmake_symbol(family_name) };
        entity.aset(font_property_index::FONT_FAMILY_INDEX, family_name);

        let (_stretch, weight, style) = font.attributes().parts();

        let weight: LispFontWeight = weight.into();
        // set weight
        entity.set_property(font_property_index::FONT_WEIGHT_INDEX, weight.0);

        let slant: LispFontStyle = style.into();
        // set slant
        entity.set_property(font_property_index::FONT_SLANT_INDEX, slant.0);

        // // set spacing
        // entity.set_style(font_property_index::FONT_SPACING_INDEX, spacing);

        // set width
        entity.set_property(font_property_index::FONT_WIDTH_INDEX, (0 as usize).into());

        // set size
        entity.aset(font_property_index::FONT_SIZE_INDEX, (0 as usize).into());

        // set registry
        entity.aset(font_property_index::FONT_REGISTRY_INDEX, Qiso10646_1);
        let font_id = unsafe { make_fixnum(font.id().to_usize() as i64) };
        let extra = unsafe { Fcons(font_id, Qnil) };
        unsafe { font_put_extra(entity.0, QCfont_entity, extra) };

        entity
    }
}

extern "C" fn get_cache(f: *mut frame) -> LispObject {
    let frame = FrameRef::new(f);
    let dpyinfo = frame.display_info();

    dpyinfo.name_list_element
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

extern "C" fn list(frame: *mut frame, spec: LispObject) -> LispObject {
    // println!("list font with spec: {:?}", spec);
    //TODO handle generial font family name
    // script -> charset language
    // webrender using font-index, add font using FontID
    log::trace!("list font {:?}", spec);
    if let Some(script) = OptionalScript::from(LispFontLike(spec)) {
        log::trace!("{:?}", script);
    };

    let mut list = Qnil;

    let entity = match_(frame, spec);

    if entity.is_not_nil() {
        list = unsafe { Fcons(entity, list) }
    }

    // println!("listed font {:?}", list);
    unsafe { Fnreverse(list) }
}

extern "C" fn match_(_f: *mut frame, spec: LispObject) -> LispObject {
    let spec = LispFontLike(spec);
    let family_name = spec.family_name();
    if family_name.is_none() {
        return Qnil;
    }
    let family_name = family_name.unwrap();
    let attributes = OptionalAttributes::from(spec);
    if attributes.is_none() {
        return Qnil;
    }
    let attributes = attributes.unwrap();
    log::info!(
        "match font spec: {:?} family: {:?} attributes: {:?}",
        spec.0,
        family_name,
        attributes
    );
    if let Some(font) = FontIndex::global().query(family_name.as_str(), attributes) {
        log::debug!(
            "matched font {:?} {:?}",
            font.family_name(),
            font.attributes()
        );
        return LispFontLike::from(font).as_lisp_object();
    }
    log::error!(
        "No font for: {:?} family: {:?} attributes: {:?}",
        spec.0,
        family_name,
        attributes
    );
    Qnil
}

extern "C" fn list_family(_f: *mut frame) -> LispObject {
    let mut list = Qnil;

    FontIndex::global().families.iter().for_each(|family_data| {
        let f = family_data.name.to_string();
        list = LispObject::cons(unsafe { intern(f.as_ptr() as *const ::libc::c_char) }, list);
    });

    list
}

#[repr(C)]
pub struct FontInfo {
    // extend basic font
    pub font: font,

    pub base: ManuallyDrop<Font>,

    pub id: FontId,

    pub frame: *mut frame,
}

impl FontInfo {
    pub fn too_high_p(&self) -> bool {
        let pixel_size = self.font.pixel_size;
        let ascent = self.font.ascent;
        let descent = self.font.descent;

        pixel_size > 0 && (ascent + descent) > 3 * pixel_size
    }

    pub fn glyph_for_char(&self, c: char) -> u16 {
        self.base.charmap().map(c)
    }

    pub fn get_glyph_advance_width(&self, glyph_indices: Vec<GlyphIndex>) -> Vec<i32> {
        let glyph_metrics = self
            .base
            .glyph_metrics(&[])
            .scale(self.font.pixel_size as f32);
        return glyph_indices
            .into_iter()
            .map(|i| glyph_metrics.advance_width(i as u16).ceil() as i32)
            .collect();
    }
}

pub type FontInfoRef = ExternalPtr<FontInfo>;

extern "C" fn open_font(frame: *mut frame, font_entity: LispObject, pixel_size: i32) -> LispObject {
    log::trace!("open font: {:?}", pixel_size);
    let extra = unsafe {
        assq_no_quit(
            QCfont_entity,
            AREF(font_entity, font_property_index::FONT_EXTRA_INDEX as isize),
        )
    };
    if !extra.is_cons() {
        return Qnil;
    }
    let extra = unsafe { XCDR(extra) };
    let font_id = unsafe { XCAR(extra) };
    let font_id = FontId(unsafe { XFIXNUM(font_id) } as u32);

    let mut frame: FrameRef = frame.into();
    let font = frame.terminal().font_cache().get(font_id);
    if font.is_none() {
        return Qnil;
    }
    let font = font.unwrap();

    let pixel_size = if pixel_size == 0 {
        // pixel_size here reflects to DPR 1 for webrender display, we have scale_factor from winit.
        // while pgtk/ns/w32 reflects to actual DPR on device by setting resx/resy to display
        if !frame.font().is_null() {
            frame.font().pixel_size as i64
        } else {
            // fallback font size
            16
        }
    } else {
        // prefer elisp specific font size
        pixel_size as i64
    };

    let font_object: LispFontLike =
        unsafe { font_make_object(vecsize!(FontInfo) as i32, font_entity, pixel_size as i32) }
            .into();

    // set type
    font_object.aset(font_property_index::FONT_TYPE_INDEX, Qswash);

    // set name
    font_object.aset(
        font_property_index::FONT_NAME_INDEX,
        LispSymbolRef::from(LispFontLike(font_entity).aref(font_property_index::FONT_FAMILY_INDEX))
            .symbol_name(),
    );

    let mut font_info = FontInfoRef::new(
        font_object
            .as_lisp_object()
            .as_font()
            .unwrap()
            .as_font_mut() as *mut FontInfo,
    );
    let metrics = font.metrics(&[]).scale(pixel_size as f32);

    font_info.font.pixel_size = pixel_size as i32;
    font_info.font.average_width = metrics.average_width.ceil() as i32;
    font_info.font.ascent = metrics.ascent.ceil() as i32;
    font_info.font.descent = metrics.descent.ceil() as i32;
    font_info.font.space_width = font
        .glyph_metrics(&[])
        .scale(pixel_size as f32)
        .advance_width(font.charmap().map(' '))
        .ceil() as i32;
    font_info.font.max_width = metrics.max_width.ceil() as i32;
    font_info.font.underline_thickness = metrics.stroke_size.ceil() as i32;
    font_info.font.underline_position = metrics.underline_offset.ceil() as i32;

    font_info.font.height = (metrics.ascent + metrics.descent + metrics.leading).ceil() as i32;
    font_info.font.baseline_offset = 0;

    font_info.id = font_id;
    font_info.base = ManuallyDrop::new(font);

    let driver = FontDriver::global();
    font_info.font.driver = &driver.0;
    font_info.frame = frame.as_mut();

    log::trace!("open font done: {:?}", pixel_size);
    font_object.as_lisp_object()
}

extern "C" fn close_font(f: *mut font) {
    let mut font = FontInfoRef::new(f as *mut FontInfo);
    unsafe { ManuallyDrop::drop(&mut font.base) };
}

extern "C" fn encode_char(font: *mut font, c: i32) -> u32 {
    let font = FontInfoRef::new(font as *mut FontInfo);

    // 0 is returned for FONT_INVALID_CODE
    // https://developer.apple.com/fonts/TrueType-Reference-Manual/RM07/appendixB.html
    // The first glyph (glyph index 0) must be the MISSING CHARACTER GLYPH. This glyph must have a visible appearance and non-zero advance width.
    let idx = std::char::from_u32(c as u32)
        .and_then(|c| Some(font.base.charmap().map(c) as u32))
        .unwrap_or(FONT_INVALID_CODE);
    if idx == 0 {
        return FONT_INVALID_CODE;
    }
    idx
}

extern "C" fn has_char(_font: LispObject, _c: i32) -> i32 {
    -1
}

#[allow(unused_variables)]
extern "C" fn text_extents(
    font: *mut font,
    code: *const u32,
    nglyphs: i32,
    metrics: *mut font_metrics,
) {
    let font_info = FontInfoRef::new(font as *mut FontInfo);

    let glyph_indices: Vec<u32> = unsafe { std::slice::from_raw_parts(code, nglyphs as usize) }
        .iter()
        .copied()
        .collect();

    let glyph_metrics = font_info
        .base
        .glyph_metrics(&[])
        .scale(font_info.font.pixel_size as f32);

    let width: i32 = font_info
        .get_glyph_advance_width(glyph_indices.clone())
        .into_iter()
        .sum();

    let mut total_width = 0;
    for (i, x) in glyph_indices.into_iter().enumerate() {
        let lbearing = glyph_metrics.lsb(x as u16).ceil() as i16;
        let rbearing: i16 = 0;
        let width = glyph_metrics.advance_width(x as u16).ceil() as i16;

        if i == 0 {
            (unsafe { *metrics }).lbearing = lbearing;
            // FIXME swash doesn't have rsb for rbearing
            (unsafe { *metrics }).rbearing = rbearing;
        }
        if { unsafe { *metrics } }.lbearing > width + lbearing {
            (unsafe { *metrics }).lbearing = width + lbearing;
        }
        if { unsafe { *metrics } }.rbearing < width + rbearing {
            (unsafe { *metrics }).rbearing = width + rbearing;
        }

        total_width += width;
    }

    unsafe {
        (*metrics).width = total_width as i16;
    }
}

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn otf_capability(_font: *mut font) -> LispObject {
    todo!()
}

/// Swash implementation of shape for font backend.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn shape(lgstring: LispObject, direction: LispObject) -> LispObject {
    use core::ops::Range;
    use emacs_sys::bindings::font_metrics as FontMetrics;
    use emacs_sys::bindings::CHECK_FONT_GET_OBJECT;
    use emacs_sys::globals::QL2R;
    use emacs_sys::globals::QR2L;
    use emacs_sys::number::LNumber;
    use emacs_sys::thread::ThreadState;
    use swash::shape::cluster::Glyph;
    use swash::shape::cluster::GlyphCluster;
    use swash::shape::Direction;

    let font = lgstring.lgstring_font();
    let font = unsafe { CHECK_FONT_GET_OBJECT(font) };
    let mut font_info = FontInfoRef::new(font as *mut FontInfo);
    let frame: FrameRef = font_info.frame.into();
    let font = frame.terminal().font_cache().get(font_info.id).unwrap();
    let glyph_len = 0;
    let text_len = lgstring.lgstring_glyph_len();
    let mut chars: Vec<char> = Vec::with_capacity(text_len.try_into().unwrap());

    for i in 0..text_len {
        let g = lgstring.lgstring_glyph(i);
        if g.is_nil() {
            break;
        }
        let c = g.lglyph_char();
        let c = char::from_u32(c.xfixnum().try_into().unwrap()).unwrap();
        chars.push(c);
    }
    let source: String = chars.into_iter().collect();
    if source.len() == 0 {
        return Qnil;
    };
    let source = source.as_str();

    let mut context = SHAPE_CONTEXT.lock().expect("SHAPE_CONTEXT lock() failed");
    let mut shaper_builder = context.builder(&font);

    /* If the caller didn't provide a meaningful DIRECTION, let Swash
    guess it. */
    if !direction.is_nil()
        /* If they bind bidi-display-reordering to nil, the DIRECTION
	they provide is meaningless, and we should let Swash guess
	the real direction.  */
        && !ThreadState::current_buffer_unchecked().bidi_display_reordering_.is_nil()
    {
        if direction.eq(QR2L) {
            shaper_builder = shaper_builder.direction(Direction::RightToLeft);
        } else if direction.eq(QL2R) {
            shaper_builder = shaper_builder.direction(Direction::LeftToRight);
        }
    }

    /* Leave the script determination to HarfBuzz, until Emacs has a
    better idea of the script of LGSTRING.  FIXME. */
    // builder.script(Script::Latin)

    /* FIXME: This can only handle the single global language, which
    normally comes from the locale.  In addition, if
    current-iso639-language is a list, we arbitrarily use the first
    one.  We should instead have a notion of the language of the text
    being shaped.  */

    let mut lang = unsafe { globals.Vcurrent_iso639_language };
    if lang.is_cons() {
        lang = lang.force_cons().car();
    }
    if lang.is_symbol() {
        lang = lang.force_symbol().symbol_name();
        let lang: String = lang.into();
        shaper_builder = shaper_builder.language(Language::parse(lang.as_str()));
    }

    //TODO use script
    //TODO use features
    //TODO use variations
    let mut shaper = shaper_builder
        .size(font_info.font.pixel_size as f32)
        // .features(&[("dlig", 1)])
        // .variations(&[("wght", 520.5)])
        .build();

    shaper.add_str(source);

    let mut i = 0;
    let mut metrics = FontMetrics::default();

    // Not sure what to do with this.
    // https://docs.rs/swash/latest/swash/shape/index.html#collecting-the-prize
    // Please note that, unlike HarfBuzz, this shaper does not reverse runs that are in right-to-left order. The reasoning is that, for correctness, line breaking must be done in logical order and reversing runs should occur during bidi reordering.

    // Also pertinent to right-to-left runs: you’ll need to ensure that you reverse clusters and not glyphs. Intra-cluster glyphs must remain in logical order for proper mark placement.

    shaper.shape_with(
        |GlyphCluster {
             source: source_range,
             glyphs,
             ..
         }| {
            let Range { start, end } = source_range.to_range();

            /* FROM is the index of the first character that contributed
            to this cluster.  */
            let from = source[0..start].char_indices().count();

            /* TO is the index of the last character that contributed to
            this cluster.  */
            let to = source[0..end].char_indices().count() - 1;
            let mut chars = source[start..end].chars();
            /* Not every glyph in a cluster maps directly to a single
            character; in general, N characters can yield M glyphs, where
            M could be smaller or greater than N.  However, in many cases
            there is a one-to-one correspondence, and it would be a pity
            to lose that information, even if it's sometimes inaccurate.  */
            for Glyph {
                id, x, y, advance, ..
            } in glyphs.iter()
            {
                if i > lgstring.lgstring_glyph_len() {
                    break;
                }

                let mut lglyph = lgstring.lgstring_glyph(i);
                if lglyph.is_nil() {
                    lglyph = LispObject::lglyph_new();
                    lgstring.set_lgstring_glyph(i, lglyph);
                }

                /* All the glyphs in a cluster have the same values of FROM and TO.  */
                lglyph.set_lglyph_from_to(from, to);

                if let Some(char) = chars.next() {
                    lglyph.set_lglyph_char(char as u32);
                }
                lglyph.set_lglyph_code(*id);

                let code = *id as u32;
                text_extents(font_info.as_mut() as *mut font, &code, 1, &mut metrics);

                let xoff = x.ceil() as i16;
                let yoff = y.ceil() as i16;
                let advance = advance.ceil() as i16;

                lglyph.set_lglyph_width(advance);
                lglyph.set_lglyph_lbearing(metrics.lbearing);
                lglyph.set_lglyph_rbearing(metrics.rbearing);
                lglyph.set_lglyph_ascent(metrics.ascent);
                lglyph.set_lglyph_descent(metrics.descent);

                if xoff != 0 || yoff != 0 || advance != metrics.width {
                    lglyph.set_lglyph_adjustment(xoff, yoff, advance);
                }

                i += 1;
            }
        },
    );
    if i > lgstring.lgstring_glyph_len() {
        return Qnil;
    }
    i.into()
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn register_swash_font_driver(f: *mut frame) {
    let driver = FontDriver::global();
    unsafe {
        register_font_driver(&driver.0, f);
    }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn syms_of_swash_font() {
    def_lisp_sym!(Qswash, "swash");
    def_lisp_sym!(Qmonospace, "monospace");
    def_lisp_sym!(Qfixed, "fixed");
    def_lisp_sym!(Qzh, "zh");

    #[rustfmt::skip]
    defvar_lisp!(Vregistry_script_alist, "registry-script-alist", Qnil);

    register_swash_font_driver(ptr::null_mut());
}

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
