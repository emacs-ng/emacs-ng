use crate::emacs::number::LNumber;
use crate::image::ImageRef;
use emacs::composite::LGlyph;
use emacs::composite::LGlyphString;
use emacs::lisp::LispObject;
use euclid::Scale;

use webrender::api::units::*;
use webrender::api::*;
use webrender::{self};

use crate::frame::FrameExtGlRendererCommon;

use font::FontInfo;
use font::FontInfoRef;

use emacs::bindings::face_box_type::FACE_NO_BOX;
use emacs::bindings::glyph_type;
use emacs::display_traits::GlyphStringRef;
use emacs::frame::FrameRef;

pub trait WrGlyph {
    fn x(self) -> i32;
    fn box_line_width(self) -> i32;
    fn font(self) -> FontInfoRef;
    fn font_instance_key(self) -> FontInstanceKey;
    fn image(self) -> ImageRef;
    fn type_(self) -> glyph_type::Type;
    fn composite_p(self) -> bool;
    fn automatic_composite_p(self) -> bool;
    fn visible_height(self) -> i32;
    fn frame(self) -> FrameRef;
    fn glyph_indices(self) -> Vec<u32>;
    fn scaled_glyph_instances(self, scale: f32) -> Vec<GlyphInstance>;
    fn glyph_instances(self) -> Vec<GlyphInstance>;
    fn char_glyph_instances(self) -> Vec<GlyphInstance>;
    fn composite_glyph_instances(self) -> Vec<GlyphInstance>;
    fn automatic_composite_glyph_instances(self) -> Vec<GlyphInstance>;
}

impl WrGlyph for GlyphStringRef {
    // If first glyph of S has a left box line, start drawing the text
    // of S to the right of that box line.
    fn x(self) -> i32 {
        if !self.face.is_null()
            && unsafe { (*self.face).box_() } != FACE_NO_BOX
            && unsafe { (*self.first_glyph).left_box_line_p() }
        {
            self.x + std::cmp::max(unsafe { (*self.face).box_vertical_line_width }, 0)
        } else {
            self.x
        }
    }

    fn box_line_width(self) -> i32 {
        std::cmp::max(unsafe { (*self.face).box_horizontal_line_width }, 0)
    }

    fn frame(self) -> FrameRef {
        self.f.into()
    }

    fn font(self) -> FontInfoRef {
        FontInfoRef::new(self.font as *mut FontInfo)
    }

    fn image(self) -> ImageRef {
        self.img.into()
    }

    fn type_(self) -> glyph_type::Type {
        self.first_glyph().type_()
    }

    fn composite_p(self) -> bool {
        self.type_() == glyph_type::COMPOSITE_GLYPH
    }

    fn automatic_composite_p(self) -> bool {
        self.composite_p() && unsafe { (*self.first_glyph).u.cmp.automatic() }
    }

    fn visible_height(self) -> i32 {
        if unsafe { (*self.row).mode_line_p() } {
            unsafe { (*self.row).height }
        } else {
            unsafe { (*self.row).visible_height }
        }
    }

    fn font_instance_key(self) -> FontInstanceKey {
        let font = self.font();
        let scale = self.frame().gl_renderer().scale();
        self.frame()
            .gl_renderer()
            .get_or_create_font_instance(font, font.font.pixel_size as f32 * scale)
    }

    fn glyph_indices(self) -> Vec<u32> {
        let from = 0 as usize;
        let to = self.nchars as usize;

        self.get_chars()[from..to]
            .iter()
            .map(|c| *c as u32)
            .collect()
    }

    fn scaled_glyph_instances(self, scale: f32) -> Vec<GlyphInstance> {
        let instances = self.glyph_instances();

        let face = self.face;
        let overstrike = unsafe { (*face).overstrike() };

        let mut scaled: Vec<GlyphInstance> = vec![];
        for instance in instances.iter() {
            let cur_point = instance.point;
            scaled.push(GlyphInstance {
                point: cur_point * Scale::new(scale),
                ..*instance
            });
            if overstrike {
                scaled.push(GlyphInstance {
                    point: LayoutPoint::new(cur_point.x + 1.0, cur_point.y) * Scale::new(scale),
                    ..*instance
                });
            }
        }
        scaled
    }

    fn glyph_instances(self) -> Vec<GlyphInstance> {
        let type_ = self.type_();

        match type_ {
            glyph_type::CHAR_GLYPH => self.char_glyph_instances(),
            glyph_type::COMPOSITE_GLYPH => {
                if self.automatic_composite_p() {
                    self.automatic_composite_glyph_instances()
                } else {
                    self.composite_glyph_instances()
                }
            }
            _ => vec![],
        }
    }

    fn char_glyph_instances(self) -> Vec<GlyphInstance> {
        let font = self.font();

        let x_start = self.x();
        let y_start = self.y + (font.font.ascent + (self.height - font.font.height) / 2);

        let glyph_indices = self.glyph_indices();

        let glyph_dimensions = font.get_glyph_advance_width(glyph_indices.clone());
        let mut glyph_instances: Vec<GlyphInstance> = vec![];
        // println!("indices: {:?}, dimensions: {:?}", glyph_indices.clone(), glyph_dimensions);

        for (i, index) in glyph_indices.into_iter().enumerate() {
            let previous_char_width = if i == 0 {
                0.0
            } else {
                glyph_dimensions[i - 1] as f32
            };

            let previous_char_start = if i == 0 {
                x_start as f32
            } else {
                glyph_instances[i - 1].point.x
            };

            let start = previous_char_start + previous_char_width;

            let glyph_instance = GlyphInstance {
                index,
                point: LayoutPoint::new(start, y_start as f32),
            };

            glyph_instances.push(glyph_instance);
        }
        glyph_instances
    }

    fn composite_glyph_instances(self) -> Vec<GlyphInstance> {
        let font = self.font();

        let x = self.x();

        let y_start = self.y + (font.font.ascent + (self.height - font.font.height) / 2);

        let offsets = self.composite_offsets();

        let glyph_instances: Vec<GlyphInstance> = self
            .composite_chars()
            .into_iter()
            .enumerate()
            .filter_map(|(n, glyph)| {
                // TAB in a composition means display glyphs with padding
                // space on the left or right.
                if self.composite_glyph(n as usize).is_some()
                    && self.composite_glyph(n as usize).unwrap() == <u8 as Into<i64>>::into(b'\t')
                {
                    return None;
                }

                let xx = x + offsets[n as usize * 2] as i32;
                let yy = y_start - offsets[n as usize * 2 + 1] as i32;

                let glyph_instance = GlyphInstance {
                    index: *glyph,
                    point: LayoutPoint::new(xx as f32, yy as f32),
                };

                Some(glyph_instance)
            })
            .collect();
        glyph_instances
    }

    fn automatic_composite_glyph_instances(self) -> Vec<GlyphInstance> {
        let mut instances: Vec<GlyphInstance> = vec![];
        let lgstring = self.get_lgstring();
        let mut composite_lglyph = |lglyph: LispObject, x: i32, y: i32| {
            let code = lglyph.lglyph_code().as_fixnum_or_error();
            let index: webrender::api::GlyphIndex = code.try_into().unwrap();
            let glyph_instance = GlyphInstance {
                index,
                point: LayoutPoint::new(x as f32, y as f32),
            };
            log::warn!("automatic composite glyph instance {glyph_instance:?}");
            instances.push(glyph_instance);
        };

        let mut x = self.x() as u32;

        let y = self.ybase;
        let mut width = 0;

        let cmp_from = self.cmp_from;
        let cmp_to = self.cmp_to;

        let mut i = cmp_from;
        let mut j = cmp_from;

        for n in cmp_from..cmp_to {
            let lglyph = lgstring.lgstring_glyph(n as u32);
            if lglyph.lglyph_adjustment().is_nil() {
                width += lglyph.lglyph_width().xfixnum();
            } else {
                if j < i {
                    composite_lglyph(lglyph, x as i32, y);
                    x += width as u32;
                }
                let xoff = lglyph.lglyph_xoff() as u32;
                let yoff = lglyph.lglyph_yoff() as u32;
                let wadjust = lglyph.lglyph_wadjust() as u32;
                composite_lglyph(lglyph, x as i32 + xoff as i32, y + yoff as i32);

                x += wadjust;
                j = i + 1;
                width = 0;
            }
            i = i + 1;
        }
        if j < i {
            for n in j..i {
                let lglyph = lgstring.lgstring_glyph(n as u32);
                composite_lglyph(lglyph, x as i32, y);
                x += width as u32;
            }
        }

        instances
    }
}
