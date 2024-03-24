use crate::display_info::DisplayInfoExtWr;
use crate::frame::FrameExtWrCommon;
use crate::image::ImageRef;
use crate::util::HandyDandyRectBuilder;
use emacs_sys::bindings::face_box_type::FACE_NO_BOX;
use emacs_sys::bindings::face_box_type::{self};
use emacs_sys::bindings::globals;
use emacs_sys::color::color_to_pixel;
use emacs_sys::display_traits::DrawGlyphsFace;
use emacs_sys::display_traits::FaceUnderlineType;
use emacs_sys::display_traits::GlyphStringRef;
use emacs_sys::display_traits::GlyphType;
use emacs_sys::frame::FrameRef;
use emacs_sys::lisp::LispObject;
use emacs_sys::number::LNumber;
use euclid::Scale;
use font::FontInfo;
use font::FontInfoRef;
use std::cmp::max;
use webrender::api::units::*;
use webrender::api::*;
use webrender::{self};
// TODO: maybe configurable from lisp world
const WAVY_LINE_THICKNESS: i32 = 1;

pub trait WrGlyph {
    fn x(&self) -> i32;
    fn box_line_width(&self) -> i32;
    fn underline_area(&self) -> LayoutRect;
    fn underwave_area(&self) -> LayoutRect;
    fn font_info(&self) -> FontInfoRef;
    fn font_instance_key(&self) -> FontInstanceKey;
    fn image(&self) -> ImageRef;
    fn composite_p(&self) -> bool;
    fn automatic_composite_p(&self) -> bool;
    fn visible_height(&self) -> i32;
    fn frame(&self) -> FrameRef;
    fn scale_factor(&self) -> f32;
    fn glyph_indices(&self) -> Vec<u32>;
    fn scaled_glyph_instances(&self, scale: f32) -> Vec<GlyphInstance>;
    fn glyph_instances(&self) -> Vec<GlyphInstance>;
    fn char_glyph_instances(&self) -> Vec<GlyphInstance>;
    fn composite_glyph_instances(&self) -> Vec<GlyphInstance>;
    fn automatic_composite_glyph_instances(&self) -> Vec<GlyphInstance>;
}

impl WrGlyph for GlyphStringRef {
    // If first glyph of S has a left box line, start drawing the text
    // of S to the right of that box line.
    fn x(&self) -> i32 {
        if !self.face.is_null()
            && unsafe { (*self.face).box_() } != FACE_NO_BOX
            && unsafe { (*self.first_glyph).left_box_line_p() }
        {
            self.x + std::cmp::max(unsafe { (*self.face).box_vertical_line_width }, 0)
        } else {
            self.x
        }
    }

    fn box_line_width(&self) -> i32 {
        std::cmp::max(unsafe { (*self.face).box_horizontal_line_width }, 0)
    }

    fn underwave_area(&self) -> LayoutRect {
        let wave_height = 3;
        // let wave_length = 2; // Webrender internals
        (self.x, self.ybase - wave_height + 3).by(
            self.width as i32,
            wave_height,
            self.scale_factor(),
        )
    }

    fn underline_area(&self) -> LayoutRect {
        assert_ne!(self.face().underline_type(), FaceUnderlineType::Wave);
        let underline_size = || {
            if let Some(prev) = self.prev().filter(|s| {
                s.face().underline_type() == FaceUnderlineType::Line
                    && (s.face().underline_at_descent_line_p()
                        == self.face().underline_at_descent_line_p())
                    && (s.face().underline_pixels_above_descent_line
                        == self.face().underline_pixels_above_descent_line)
            }) {
                return (prev.underline_thickness, prev.underline_position);
            } else {
                let font = self.font_for_underline_metrics();
                /* Get the underline thickness.  Default is 1 pixel.  */
                let thickness = font
                    .filter(|font| font.underline_thickness > 0)
                    .map(|font| font.underline_thickness)
                    .unwrap_or(1);
                if unsafe { globals.Vx_underline_at_descent_line }
                    || self.face().underline_at_descent_line_p()
                {
                    let position = (self.height - thickness)
                        - (self.ybase - self.y)
                        - self.face().underline_pixels_above_descent_line;
                    return (thickness, position);
                } else {
                    //  Get the underline position.  This is the recommended
                    // vertical offset in pixels from the baseline to the top of
                    // the underline.  This is a signed value according to the
                    // specs, and its default is

                    // ROUND ((maximum descent) / 2), with
                    // ROUND(x) = floor (x + 0.5)
                    let position = match font {
                        Some(font)
                            if unsafe { globals.Vx_use_underline_position_properties }
                                && font.underline_position >= 0 =>
                        {
                            font.underline_position
                        }
                        Some(font) => (font.descent + 1) / 2,
                        _ => unsafe { globals.underline_minimum_offset.try_into().unwrap() },
                    };
                    return (thickness, position);
                };
            }
        };

        let (mut thickness, mut position) = underline_size();
        /* Ignore minimum_offset if the amount of pixels was
        explicitly specified.  */
        if self.face().underline_pixels_above_descent_line != 0 {
            position = max(position, unsafe {
                globals.underline_minimum_offset.try_into().unwrap()
            });
        }
        /* Check the sanity of thickness and position.  We should
        avoid drawing underline out of the current line area.  */
        if self.y + self.height <= self.ybase + position {
            position = (self.height - 1) - (self.ybase - self.y);
        }
        if self.y + self.height < self.ybase + position + thickness {
            thickness = (self.y + self.height) - (self.ybase + position);
        }
        self.clone().underline_thickness = thickness;
        self.clone().underline_position = position;
        let y = self.ybase + position;
        (self.x, y).by(self.width as i32, thickness, self.scale_factor())
    }

    fn frame(&self) -> FrameRef {
        self.f.into()
    }

    fn scale_factor(&self) -> f32 {
        self.frame().scale_factor() as f32
    }

    fn font_info(&self) -> FontInfoRef {
        FontInfoRef::new(self.font as *mut FontInfo)
    }

    fn image(&self) -> ImageRef {
        self.img.into()
    }

    fn composite_p(&self) -> bool {
        self.glyph_type() == GlyphType::Composite
    }

    fn automatic_composite_p(&self) -> bool {
        self.composite_p() && unsafe { (*self.first_glyph).u.cmp.automatic() }
    }

    fn visible_height(&self) -> i32 {
        if unsafe { (*self.row).mode_line_p() } {
            unsafe { (*self.row).height }
        } else {
            unsafe { (*self.row).visible_height }
        }
    }

    fn font_instance_key(&self) -> FontInstanceKey {
        let font_info = self.font_info();
        let scale = self.frame().gl_renderer().scale();
        self.frame()
            .gl_renderer()
            .get_or_create_font_instance(font_info, font_info.font.pixel_size as f32 * scale)
    }

    fn glyph_indices(&self) -> Vec<u32> {
        let from = 0 as usize;
        let to = self.nchars as usize;

        self.get_chars()[from..to]
            .iter()
            .map(|c| *c as u32)
            .collect()
    }

    fn scaled_glyph_instances(&self, scale: f32) -> Vec<GlyphInstance> {
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

    fn glyph_instances(&self) -> Vec<GlyphInstance> {
        let glyph_type = self.glyph_type();

        match glyph_type {
            GlyphType::Char => self.char_glyph_instances(),
            GlyphType::Composite => {
                if self.automatic_composite_p() {
                    self.automatic_composite_glyph_instances()
                } else {
                    self.composite_glyph_instances()
                }
            }
            _ => vec![],
        }
    }

    fn char_glyph_instances(&self) -> Vec<GlyphInstance> {
        let font_info = self.font_info();

        let x_start = self.x();
        let y_start = self.y + (font_info.font.ascent + (self.height - font_info.font.height) / 2);

        let glyph_indices = self.glyph_indices();

        let glyph_dimensions = font_info.get_glyph_advance_width(glyph_indices.clone());
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

    fn composite_glyph_instances(&self) -> Vec<GlyphInstance> {
        let font_info = self.font_info();

        let x = self.x();

        let y_start = self.y + (font_info.font.ascent + (self.height - font_info.font.height) / 2);

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

    fn automatic_composite_glyph_instances(&self) -> Vec<GlyphInstance> {
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

        let mut x = self.x();

        let y = self.ybase;

        let cmp_from = self.cmp_from;
        let cmp_to = self.cmp_to;

        for n in cmp_from..cmp_to {
            let lglyph = lgstring.lgstring_glyph(n as u32);
            if lglyph.lglyph_adjustment().is_nil() {
                composite_lglyph(lglyph, x, y);
                if self.face().overstrike() {
                    composite_lglyph(lglyph, x + 1, y);
                }
                let glyph_pixel_width = lglyph.lglyph_width().xfixnum() as i32;
                x += glyph_pixel_width;
            } else {
                let xoff = lglyph.lglyph_xoff() as i32;
                let yoff = lglyph.lglyph_yoff() as i32;
                let wadjust = lglyph.lglyph_wadjust() as i32;
                if self.face().overstrike() {
                    composite_lglyph(lglyph, x + xoff + 1, y + yoff);
                }

                x += wadjust;
            }
        }
        instances
    }
}

pub trait GlyphStringExtWr {
    fn set_gc(&mut self);
    fn set_cursor_gc(&mut self);
    fn set_mouse_gc(&mut self);
    fn set_clipping(&mut self);
    fn set_clipping_exactly(&mut self, dist: Self);
    fn draw(&mut self);
    fn draw_stretch(&mut self);
    fn fill_rectangle(
        &mut self,
        color: u64,
        layout_rect: LayoutRect,
        respect_alpha_background: bool,
    );
    fn fill_background(&mut self, x: i32, y: i32, width: i32, height: i32);
    fn draw_box(&mut self);
    fn draw_line(
        &self,
        style: LineStyle,
        color: ColorF,
        rect: LayoutRect,
        orientation: LineOrientation,
    );
    fn draw_underline(&self);
    fn draw_overline(&mut self);
    fn draw_strike_through(&mut self);
    fn draw_image(&mut self);
    fn draw_xwidget(&mut self);
    fn draw_background(&mut self, is_force: bool);
    fn draw_foreground(&mut self);
    fn draw_composite_foreground(&mut self);
    fn draw_glyphless_foreground(&mut self);
    fn clear_area(&mut self, clear_color: ColorF, x: i32, y: i32, width: i32, height: i32);
    fn clear_rect(&mut self, clear_color: ColorF, x: i32, y: i32, width: i32, height: i32);
    fn draw_rectangle(&mut self, clear_color: ColorF, rect: LayoutRect);
}

impl GlyphStringExtWr for GlyphStringRef {
    // Set S->gc of glyph string S to a GC suitable for drawing a mode line.
    // Faces to use in the mode line have already been computed when the
    // matrix was built, so there isn't much to do, here.
    fn set_gc(&mut self) {
        self.prepare_face_for_display();
        match self.hl() {
            DrawGlyphsFace::Cursor => self.set_cursor_gc(),
            DrawGlyphsFace::Mouse => self.set_mouse_gc(),
            DrawGlyphsFace::NormalText
            | DrawGlyphsFace::InverseVideo
            | DrawGlyphsFace::ImageRaised
            | DrawGlyphsFace::ImageSunken => {
                self.gc = self.face().gc;
                let is_stippled = self.face().stipple != 0;
                self.set_stippled_p(is_stippled);
            }
        }
    }

    fn set_cursor_gc(&mut self) {
        let face = self.face();
        let f = self.frame();
        if self.font() == f.font()
            && face.background == f.background_pixel
            && face.foreground == f.foreground_pixel
            && !self.cmp.is_null()
        {
            // winit specific
            // self.frame().winit_data().map(|d| {
            //     self.gc().background = color_to_pixel(d.cursor_color);
            //     self.gc().foreground = color_to_pixel(d.cursor_foreground_color);
            // });
        } else {
            /* Cursor on non-default face: must merge.  */
            // FIXME not sure the logic below is aligned with x_set_cursor_gc
            // needs to check
            let mut dpyinfo = f.display_info();

            let mut foreground = face.background;
            let mut background = color_to_pixel(f.cursor_color());

            // If the glyph would be invisible, try a different foreground.
            if foreground == background {
                foreground = face.foreground;
            }

            if foreground == background {
                foreground = color_to_pixel(f.cursor_foreground_color());
            }

            if foreground == background {
                foreground = face.foreground;
            }

            // Make sure the cursor is distinct from text in this face.
            if foreground == face.foreground && background == face.background {
                foreground = face.background;
                background = face.foreground;
            }

            let gc = &mut dpyinfo.gl_renderer_data().scratch_cursor_gc;
            gc.foreground = foreground;
            gc.background = background;
            self.gc = gc.as_mut();

            self.set_stippled_p(false);
        }
    }
    fn set_mouse_gc(&mut self) {
        if self.font() == self.face().font() {
            self.gc = self.face().gc;
        } else {
            log::error!("unimplemented code path set_mouse_gc, ref x_set_mouse_face_gc");
        }
    }

    fn set_clipping(&mut self) {
        log::error!("unimplemented set clipping ref: x_set_glyph_string_clipping");
    }
    fn set_clipping_exactly(&mut self, _dist: Self) {
        log::error!("unimplemented set clipping ref: x_set_glyph_string_clipping");
    }

    fn draw(&mut self) {
        let mut is_relief_drawn = false;
        // If S draws into the background of its successors, draw the
        // background of the successors first so that S can draw into it.
        // This makes S->next use XDrawString instead of XDrawImageString.
        let right_overhang = self.right_overhang;
        if right_overhang != 0 && !self.is_for_overlaps() && self.next().is_some() {
            let mut width = 0;
            for (_, mut s) in self.next().unwrap().into_iter().enumerate() {
                if width >= right_overhang {
                    break;
                }

                let glyph_type = s.glyph_type();
                if glyph_type != GlyphType::Image {
                    s.set_gc();
                    s.set_clipping();
                } else if glyph_type == GlyphType::Stretch {
                    s.draw_stretch();
                } else {
                    s.draw_background(false);
                }
                width += s.width;
            }
        }

        // Set up S->gc, set clipping and draw S.
        self.set_gc();

        // Draw relief (if any) in advance for char/composition so that the
        // glyph string can be drawn over it.
        if !self.is_for_overlaps()
            && self.face().box_() != face_box_type::FACE_NO_BOX
            && (self.glyph_type() == GlyphType::Char || self.glyph_type() == GlyphType::Composite)
        {
            self.set_clipping();
            self.draw_background(true);
            self.draw_box();
            self.set_clipping();
            is_relief_drawn = true;
        } else if self.clip_head().is_none() // draw_glyphs didn't specify a clip mask.
            && self.clip_tail().is_none()
            && (self.prev().map(|s| s.hl() != self.hl() && self.left_overhang != 0)
                .unwrap_or(false)
                || self.next().map(|s| s.hl() != self.hl() && self.right_overhang != 0)
                    .unwrap_or(false))
        {
            // We must clip just this glyph.  left_overhang part has already
            // drawn when s->prev was drawn, and right_overhang part will be
            // drawn later when s->next is drawn.
            self.set_clipping_exactly(self.clone());
        } else {
            self.set_clipping();
        }

        match self.glyph_type() {
            GlyphType::Image => self.draw_image(),
            GlyphType::Xwidget => self.draw_xwidget(),
            GlyphType::Stretch => self.draw_stretch(),
            GlyphType::Char => {
                if self.for_overlaps() != 0 {
                    self.set_background_filled_p(true);
                } else {
                    self.draw_background(false);
                }
                self.draw_foreground();
            }
            GlyphType::Composite => {
                if self.for_overlaps() != 0
                    || (self.cmp_from > 0 && !self.is_automatic_composition())
                {
                    self.set_background_filled_p(true);
                } else {
                    self.draw_background(true);
                }
                self.draw_composite_foreground();
            }
            GlyphType::Glyphless => {
                if self.for_overlaps() != 0 {
                    self.set_background_filled_p(true);
                } else {
                    self.draw_background(true);
                }
                self.draw_glyphless_foreground()
            }
        }

        if !self.is_for_overlaps() {
            // Draw relief if not yet drawn.
            if !is_relief_drawn && self.face().box_() != face_box_type::FACE_NO_BOX {
                self.draw_box();
            }

            // Draw underline
            // match self.face().underline_type() {
            //     FaceUnderlineType::Wave => {
            //         self.draw_underwave(self.underline_color());
            //     }
            //     FaceUnderlineType::Line => {
            //         let layout_rect = self.underline_layout_area();
            //         self.fill_rectangle(self.underline_color(), layout_rect, false);
            //     }
            //     FaceUnderlineType::None => todo!(),
            // }

            // Draw overline
            if self.face().overline_p() {
                // let dy = 0;
                // let h = 1;
                // let layout_rect = (self.x, self.y + dy).by(self.width, h, self.scale_factor());

                // self.fill_rectangle(self.overline_color(), layout_rect, false);
            }

            /* Draw strike-through.  */
            if self.face().strike_through_p() {
                // /* Y-coordinate and height of the glyph string's first
                // glyph.  We cannot use s->y and s->height because those
                // could be larger if there are taller display elements
                // (e.g., characters displayed with a larger font) in the
                // same glyph row.  */
                // let glyph_y = self.ybase - self.first_glyph().ascent as i32;
                // let glyph_height = self.first_glyph().ascent + self.first_glyph().descent;
                // /* Strike-through width and offset from the glyph string's
                // top edge.  */
                // let h = 1;
                // let dy = (glyph_height - h) / 2;
                // let layout_rect =
                //     (self.x, glyph_y + dy as i32).by(self.width, h as i32, self.scale_factor());
                // self.fill_rectangle(self.strike_through_color(), layout_rect, false);
            }

            if self.prev().is_some() {
                // todo!()
                /* As prev was drawn while clipped to its own area, we
                must draw the right_overhang part using s->hl now.  */
            }
            if self.next().is_some() {
                // self.next().unwrap().into_iter().for_each(|s| {
                //     //todo!()
                // });
            }
        }
        /* TODO: figure out in which cases the stipple is actually drawn on
        WR.  */
        match self.row() {
            Some(mut row) => {
                if !row.stipple_p() {
                    row.set_stipple_p(self.face().stipple != 0);
                }
            }
            _ => {}
        }

        /* Reset clipping.  */
        // pgtk_end_cr_clip (s->f);
        self.num_clips = 0;
    }

    fn draw_stretch(&mut self) {
        todo!()
    }

    fn fill_rectangle(
        &mut self,
        _color: u64,
        _layout_rect: LayoutRect,
        _respect_alpha_background: bool,
    ) {
        // let scale_factor = self.scale_factor();
        todo!();
    }

    fn fill_background(&mut self, _x: i32, _y: i32, _width: i32, _height: i32) {
        // let scale_factor = self.scale_factor();
        todo!();
    }

    fn draw_box(&mut self) {
        todo!()
    }

    fn draw_image(&mut self) {
        todo!()
    }

    // underline/wave overline strike-through etc
    // webrender support 4 line styles
    fn draw_line(
        &self,
        style: LineStyle,
        color: ColorF,
        area: LayoutRect,
        orientation: LineOrientation,
    ) {
        let x = self.x;
        let y = self.y;

        let visible_height = self.visible_height();
        self.frame()
            .gl_renderer()
            .display(|builder, space_and_clip, scale| {
                let common = CommonItemProperties::new(
                    (x, y).by(self.width as i32, visible_height, scale),
                    space_and_clip,
                );

                builder.push_line(
                    &common,
                    &area,
                    WAVY_LINE_THICKNESS as f32,
                    orientation,
                    &color,
                    style,
                );
            });
    }

    fn draw_underline(&self) {
        let color = self.underline_color();

        if let Some(style) = self.face().underline_style() {
            let area = match style {
                LineStyle::Solid | LineStyle::Dotted | LineStyle::Dashed => self.underline_area(),
                LineStyle::Wavy => self.underwave_area(),
            };
            self.draw_line(style, color, area, LineOrientation::Horizontal);
        }
    }

    fn draw_overline(&mut self) {
        assert_eq!(self.face().overline_p(), true);
        let dy = 0;
        let h = 1;
        let area = (self.x, self.y + dy).by(self.width, h, self.scale_factor());
        self.draw_line(
            LineStyle::Solid,
            self.overline_color(),
            area,
            LineOrientation::Horizontal,
        );
    }

    fn draw_strike_through(&mut self) {
        assert_eq!(self.face().strike_through_p(), true);
        /* Y-coordinate and height of the glyph string's first
        glyph.  We cannot use s->y and s->height because those
        could be larger if there are taller display elements
        (e.g., characters displayed with a larger font) in the
        same glyph row.  */
        let glyph_y = self.ybase - self.first_glyph().ascent as i32;
        let glyph_height = self.first_glyph().ascent + self.first_glyph().descent;
        /* Strike-through width and offset from the glyph string's
        top edge.  */
        let h = 1;
        let dy = (glyph_height - h) / 2;
        let area = (self.x, glyph_y + dy as i32).by(self.width, h as i32, self.scale_factor());
        self.draw_line(
            LineStyle::Solid,
            self.strike_through_color(),
            area,
            LineOrientation::Horizontal,
        );
    }

    fn draw_xwidget(&mut self) {
        todo!()
    }

    // Draw the background of glyph_string S.  If S->background_filled_p
    // is non-zero don't draw it.  FORCE_P non-zero means draw the
    // background even if it wouldn't be drawn normally.  This is used
    // when a string preceding S draws into the background of S, or S
    // contains the first component of a composition.
    fn draw_background(&mut self, is_force: bool) {
        // Nothing to do if background has already been drawn or if it
        // shouldn't be drawn in the first place.
        if self.background_filled_p() {
            return;
        }
        let box_line_width = std::cmp::max(self.face().box_horizontal_line_width, 0);

        if self.stippled_p() {
            // Fill background with a stipple pattern.
            self.fill_background(
                self.x,
                self.y + box_line_width,
                self.background_width,
                self.height - 2 * box_line_width,
            );
            self.set_background_filled_p(true);
        } else if self.font_info().font.height < self.height - 2 * box_line_width
	    /* When xdisp.c ignores FONT_HEIGHT, we cannot trust
	    font dimensions, since the actual glyphs might be
	    much smaller.  So in that case we always clear the
	    rectangle with background color.  */
	    || self.font_info().too_high_p()
            || self.font_not_found_p()
            || self.extends_to_end_of_line_p() || is_force
        {
            let background_color = self.bg_color();
            self.clear_rect(
                background_color,
                self.x,
                self.y + box_line_width,
                self.background_width,
                self.height - 2 * box_line_width,
            );

            self.set_background_filled_p(true);
        }
    }

    // Draw the foreground of glyph string S.
    fn draw_foreground(&mut self) {
        let x = self.x;
        let y = self.y;

        let visible_height = self.visible_height();

        // draw background
        let background_color = self.bg_color();
        self.clear_area(
            background_color,
            x,
            y,
            self.background_width,
            visible_height,
        );

        self.frame()
            .gl_renderer()
            .display(|builder, space_and_clip, scale| {
                let foreground_color = self.fg_color();

                // // draw underline
                // if face.underline() != face_underline_type::FACE_NO_UNDERLINE {
                //     self.draw_underline(
                //         builder,
                //         s,
                //         font_info,
                //         foreground_color,
                //         face,
                //         space_and_clip,
                //         scale,
                //     );
                // }

                let glyph_instances = self.scaled_glyph_instances(scale);
                // draw foreground
                if !glyph_instances.is_empty() {
                    let font_instance_key = self.font_instance_key();
                    let visible_rect = (x, y).by(self.width as i32, visible_height, scale);

                    builder.push_text(
                        &CommonItemProperties::new(visible_rect, space_and_clip),
                        visible_rect,
                        &glyph_instances,
                        font_instance_key,
                        foreground_color,
                        None,
                    );
                }
            });
    }

    fn draw_composite_foreground(&mut self) {
        todo!()
    }

    fn draw_glyphless_foreground(&mut self) {
        todo!()
    }

    fn clear_area(&mut self, clear_color: ColorF, x: i32, y: i32, width: i32, height: i32) {
        let scale = self.scale_factor();
        let rect = (x, y).by(width, height, scale);
        self.draw_rectangle(clear_color, rect);
    }

    fn clear_rect(&mut self, clear_color: ColorF, x: i32, y: i32, width: i32, height: i32) {
        self.clear_area(clear_color, x, y, width, height);
    }

    fn draw_rectangle(&mut self, clear_color: ColorF, rect: LayoutRect) {
        self.frame()
            .gl_renderer()
            .display(|builder, space_and_clip, _| {
                builder.push_rect(
                    &CommonItemProperties::new(rect, space_and_clip),
                    rect,
                    clear_color,
                );
            });
    }
}
