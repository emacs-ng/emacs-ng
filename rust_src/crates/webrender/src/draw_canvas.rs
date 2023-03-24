use crate::emacs::lglyph::LGlyph;
use crate::emacs::lglyph::LGlyphString;
use crate::frame::LispFrameWindowSystemExt;
use crate::image::ImageExt;
use crate::image::ImageRef;
use emacs::lisp::LispObject;
use euclid::Scale;
use std::cmp::min;
use webrender::api::ImageKey;

use webrender::{self, api::units::*, api::*};

use crate::{frame::LispFrameExt, fringe::FringeBitmap};

use super::{
    color::{color_to_pixel, pixel_to_color},
    font::{WRFont, WRFontRef},
    util::HandyDandyRectBuilder,
};

use emacs::{
    bindings::{
        draw_glyphs_face, face as Face, face_box_type::FACE_NO_BOX, face_underline_type,
        get_glyph_string_clip_rect, glyph_type, prepare_face_for_display, Emacs_Rectangle,
    },
    frame::LispFrameRef,
    glyph::GlyphStringRef,
};

pub trait Renderer {
    fn draw_glyph_string(&mut self, s: GlyphStringRef);

    fn draw_glyph_string_background(&mut self, s: GlyphStringRef, force_p: bool);

    fn draw_char_glyph_string_foreground(&mut self, s: GlyphStringRef);

    fn draw_stretch_glyph_string_foreground(&mut self, s: GlyphStringRef);

    fn draw_glyphless_glyph_string_foreground(&mut self, s: GlyphStringRef);

    fn draw_image_glyph(&mut self, s: GlyphStringRef);

    fn draw_image(
        &mut self,
        image_key: Option<ImageKey>,
        background_color: Option<ColorF>,
        bounds: LayoutRect,
        clip_bounds: Option<LayoutRect>,
    );

    fn draw_composite_glyph_string_foreground(&mut self, s: GlyphStringRef);

    fn draw_underline(
        builder: &mut DisplayListBuilder,
        s: GlyphStringRef,
        font: WRFontRef,
        foreground_color: ColorF,
        face: *mut Face,
        space_and_clip: SpaceAndClipInfo,
        scale: f32,
    );

    fn draw_fringe_bitmap(
        &mut self,
        pos: LayoutPoint,
        image: Option<FringeBitmap>,
        bitmap_color: ColorF,
        background_color: ColorF,
        image_clip_rect: LayoutRect,
        clear_rect: LayoutRect,
        row_rect: LayoutRect,
    );

    fn draw_vertical_window_border(&mut self, face: Option<*mut Face>, x: i32, y0: i32, y1: i32);

    fn draw_window_divider(
        &mut self,
        color: u64,
        color_first: u64,
        color_last: u64,
        x0: i32,
        x1: i32,
        y0: i32,
        y1: i32,
    );

    fn draw_rectangle(&mut self, clear_color: ColorF, rect: LayoutRect);

    fn clear_area(&mut self, clear_color: ColorF, x: i32, y: i32, width: i32, height: i32);

    fn scroll(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        from_y: i32,
        to_y: i32,
        scroll_height: i32,
    );
    fn draw_hollow_box_cursor(&mut self, cursor_rect: LayoutRect, clip_rect: LayoutRect);
    fn draw_bar_cursor(&mut self, face: &Face, x: i32, y: i32, width: i32, height: i32);
}

impl Renderer for LispFrameRef {
    fn draw_glyph_string(&mut self, mut s: GlyphStringRef) {
        unsafe { prepare_face_for_display(s.f, s.face) };

        match s.hl {
            draw_glyphs_face::DRAW_NORMAL_TEXT
            | draw_glyphs_face::DRAW_INVERSE_VIDEO
            | draw_glyphs_face::DRAW_MOUSE_FACE
            | draw_glyphs_face::DRAW_IMAGE_RAISED
            | draw_glyphs_face::DRAW_IMAGE_SUNKEN => {
                let face = unsafe { &*s.face };
                s.gc = face.gc;
                s.set_stippled_p(face.stipple != 0);
            }

            draw_glyphs_face::DRAW_CURSOR => {
                let face = unsafe { &*s.face };
                let frame: LispFrameRef = (*s).f.into();
                let mut dpyinfo = frame.display_info();

                let mut foreground = face.background;
                let mut background = color_to_pixel(frame.cursor_color());

                // If the glyph would be invisible, try a different foreground.
                if foreground == background {
                    foreground = face.foreground;
                }

                if foreground == background {
                    foreground = color_to_pixel(frame.cursor_foreground_color());
                }

                if foreground == background {
                    foreground = face.foreground;
                }

                // Make sure the cursor is distinct from text in this face.
                if foreground == face.foreground && background == face.background {
                    foreground = face.background;
                    background = face.foreground;
                }

                let gc = &mut dpyinfo.get_inner().scratch_cursor_gc;
                gc.foreground = foreground;
                gc.background = background;
                s.gc = gc.as_mut();

                s.set_stippled_p(false);
            }
            _ => log::warn!("invalid draw_glyphs_face {:?}", s.hl),
        }

        let type_ = s.first_glyph().type_();

        match type_ {
            glyph_type::CHAR_GLYPH => {
                if s.for_overlaps() != 0 {
                    s.set_background_filled_p(true);
                } else {
                    self.draw_glyph_string_background(s, false);
                }
                self.draw_char_glyph_string_foreground(s)
            }
            glyph_type::STRETCH_GLYPH => self.draw_stretch_glyph_string_foreground(s),
            glyph_type::IMAGE_GLYPH => self.draw_image_glyph(s),
            glyph_type::COMPOSITE_GLYPH => {
                if s.for_overlaps() != 0 || s.cmp_from > 0 && s.automatic_composite_p() {
                    s.set_background_filled_p(true);
                } else {
                    self.draw_glyph_string_background(s, true);
                }
                self.draw_composite_glyph_string_foreground(s)
            }
            glyph_type::XWIDGET_GLYPH => {
                log::warn!("TODO unimplemented! glyph_type::XWIDGET_GLYPH\n")
            }
            glyph_type::GLYPHLESS_GLYPH => {
                if s.for_overlaps() != 0 {
                    s.set_background_filled_p(true);
                } else {
                    self.draw_glyph_string_background(s, true);
                }

                self.draw_glyphless_glyph_string_foreground(s);
            }
            _ => {}
        }
    }

    // Draw the background of glyph_string S.  If S->background_filled_p
    // is non-zero don't draw it.  FORCE_P non-zero means draw the
    // background even if it wouldn't be drawn normally.  This is used
    // when a string preceding S draws into the background of S, or S
    // contains the first component of a composition.
    fn draw_glyph_string_background(&mut self, mut s: GlyphStringRef, force_p: bool) {
        // Nothing to do if background has already been drawn or if it
        // shouldn't be drawn in the first place.
        if s.background_filled_p() {
            return;
        }
        let box_line_width = std::cmp::max(unsafe { (*s.face).box_horizontal_line_width }, 0);

        if s.stippled_p() {
            // Fill background with a stipple pattern.
            // fill_background (s, s.x, s.y + box_line_width,
            //     s.background_width,
            //     s.height - 2 * box_line_width);
            s.set_background_filled_p(true);
        } else if s.font().font.height < s.height - 2 * box_line_width
	    /* When xdisp.c ignores FONT_HEIGHT, we cannot trust
	    font dimensions, since the actual glyphs might be
	    much smaller.  So in that case we always clear the
	    rectangle with background color.  */
	    || s.font().too_high_p()
            || s.font_not_found_p()
            || s.extends_to_end_of_line_p() || force_p
        {
            let gc = s.gc;
            let background_color = pixel_to_color(unsafe { (*gc).background } as u64);
            self.clear_area(
                background_color,
                s.x,
                s.y + box_line_width,
                s.background_width,
                s.height - 2 * box_line_width,
            );

            s.set_background_filled_p(true);
        }
    }

    fn draw_char_glyph_string_foreground(&mut self, s: GlyphStringRef) {
        let font = s.font();

        let gc = s.gc;

        let x = s.x;
        let y = s.y;

        let face = s.face;

        let visible_height = s.visible_height();

        // draw background
        let background_color = pixel_to_color(unsafe { (*gc).background } as u64);
        self.clear_area(background_color, x, y, s.background_width, visible_height);

        self.canvas().display(|builder, space_and_clip, scale| {
            let foreground_color = pixel_to_color(unsafe { (*gc).foreground });

            // draw underline
            if unsafe { (*face).underline() != face_underline_type::FACE_NO_UNDERLINE } {
                Self::draw_underline(
                    builder,
                    s,
                    font,
                    foreground_color,
                    face,
                    space_and_clip,
                    scale,
                );
            }

            let glyph_instances = s.scaled_glyph_instances(scale);
            // draw foreground
            if !glyph_instances.is_empty() {
                let font_instance_key = s.font_instance_key();
                let visible_rect = (x, y).by(s.width as i32, visible_height, scale);

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

    fn draw_stretch_glyph_string_foreground(&mut self, mut s: GlyphStringRef) {
        if s.background_filled_p() {
            return;
        }

        let visible_height = s.visible_height();
        let background_width = if s.hl == draw_glyphs_face::DRAW_CURSOR {
            let frame: LispFrameRef = s.f.into();

            min(frame.column_width, s.background_width)
        } else {
            s.background_width
        };

        let background_color = pixel_to_color(unsafe { (*s.gc).background } as u64);
        self.clear_area(background_color, s.x, s.y, background_width, visible_height);

        s.set_background_filled_p(true);
    }

    fn draw_image_glyph(&mut self, mut s: GlyphStringRef) {
        let image: ImageRef = s.img.into();
        let frame: LispFrameRef = s.f.into();
        let image_key = image.image_key(frame);

        // clear area
        let x = s.x;
        let y = s.y;
        let gc = s.gc;
        let visible_height = s.visible_height();
        let background_color = pixel_to_color(unsafe { (*gc).background } as u64);
        self.clear_area(background_color, x, y, s.background_width, visible_height);

        let mut clip_rect = Emacs_Rectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };

        unsafe { get_glyph_string_clip_rect(s.as_mut(), &mut clip_rect) };

        let face = unsafe { &*s.face };

        let background_color = pixel_to_color(face.background);
        let scale = s.frame().canvas().scale();
        let clip_bounds =
            (clip_rect.x, clip_rect.y).by(clip_rect.width as i32, clip_rect.height as i32, scale);
        let bounds = (s.x, s.y).by(s.slice.width() as i32, s.slice.height() as i32, scale);

        self.draw_image(image_key, Some(background_color), bounds, Some(clip_bounds));
    }

    fn draw_glyphless_glyph_string_foreground(&mut self, s: GlyphStringRef) {
        let _x = s.x();
        //TODO
    }

    fn draw_image(
        &mut self,
        image_key: Option<ImageKey>,
        background_color: Option<ColorF>,
        bounds: LayoutRect,
        clip_bounds: Option<LayoutRect>,
    ) {
        let clip_bounds = clip_bounds.unwrap_or(bounds);
        let background_rect = bounds.intersection(&clip_bounds);

        // render background
        if let Some(background_rect) = background_rect {
            let background_color = background_color.unwrap_or(ColorF::TRANSPARENT);
            self.draw_rectangle(background_color, background_rect);
        }

        if let Some(image_key) = image_key {
            self.canvas().display(|builder, space_and_clip, _scale| {
                // render image
                builder.push_image(
                    &CommonItemProperties::new(clip_bounds, space_and_clip),
                    bounds,
                    ImageRendering::Auto,
                    AlphaType::Alpha,
                    image_key,
                    ColorF::WHITE,
                );
            });
        } else {
            log::error!("image key {:?}", image_key);
        }
    }

    fn draw_composite_glyph_string_foreground(&mut self, s: GlyphStringRef) {
        // S is a glyph string for a composition.  S->cmp_from is the index
        // of the first character drawn for glyphs of this composition.
        // S->cmp_from == 0 means we are drawing the very first character of
        // this composition

        // Draw a rectangle for the composition if the font for the very
        // first character of the composition could not be loaded.
        if s.font_not_found_p() {
            if s.cmp_from == 0 {
                self.clear_area(self.cursor_color(), s.x, s.y, s.width, s.height);
            }
        } else {
            let font = s.font();

            let face = s.face;

            let gc = s.gc;

            let visible_height = s.visible_height();

            let x = s.x;
            let y = s.y;
            let background_color = pixel_to_color(unsafe { (*gc).background } as u64);
            self.clear_area(background_color, x, y, s.background_width, visible_height);
            self.canvas().display(|builder, space_and_clip, scale| {
                let s = s.clone();

                let foreground_color = pixel_to_color(unsafe { (*gc).foreground });

                // draw underline
                if unsafe { (*face).underline() != face_underline_type::FACE_NO_UNDERLINE } {
                    Self::draw_underline(
                        builder,
                        s,
                        font,
                        foreground_color,
                        face,
                        space_and_clip,
                        scale,
                    );
                }

                let visible_rect = (x, y).by(s.width, visible_height, scale);

                let glyph_instances = s.scaled_glyph_instances(scale);
                // draw foreground
                if !glyph_instances.is_empty() {
                    let font_instance_key = s.font_instance_key();
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
    }

    fn draw_underline(
        builder: &mut DisplayListBuilder,
        s: GlyphStringRef,
        font: WRFontRef,
        foreground_color: ColorF,
        face: *mut Face,
        space_and_clip: SpaceAndClipInfo,
        scale: f32,
    ) {
        let x = s.x;
        let y = s.y;

        let underline_color = if unsafe { (*face).underline_defaulted_p() } {
            foreground_color
        } else {
            pixel_to_color(unsafe { (*face).underline_color })
        };

        let thickness = if font.font.underline_thickness > 0 {
            font.font.underline_thickness
        } else if unsafe { (*face).underline() } == face_underline_type::FACE_UNDER_WAVE {
            2
        } else {
            1
        };

        let position = if font.font.underline_position > 0 {
            font.font.underline_position
        } else {
            y + s.height - thickness
        };

        let line_type = if unsafe { (*face).underline() } == face_underline_type::FACE_UNDER_WAVE {
            LineStyle::Wavy
        } else {
            LineStyle::Solid
        };

        let visible_height = s.visible_height();

        let info = CommonItemProperties::new(
            (x, y).by(s.width as i32, visible_height, scale),
            space_and_clip,
        );

        let visible_rect = (x, position).by(s.width as i32, thickness, scale);

        builder.push_line(
            &info,
            &visible_rect,
            1.0,
            LineOrientation::Horizontal,
            &underline_color,
            line_type,
        );
    }

    fn draw_fringe_bitmap(
        &mut self,
        pos: LayoutPoint,
        image: Option<FringeBitmap>,
        bitmap_color: ColorF,
        background_color: ColorF,
        image_clip_rect: LayoutRect,
        clear_rect: LayoutRect,
        row_rect: LayoutRect,
    ) {
        // Fixed clear_rect
        let clear_rect = clear_rect
            .union(&image_clip_rect)
            .intersection(&row_rect)
            .unwrap_or_else(|| LayoutRect::zero());

        // Fixed image_clip_rect
        let image_clip_rect = image_clip_rect
            .intersection(&row_rect)
            .unwrap_or_else(|| LayoutRect::zero());

        // clear area
        self.draw_rectangle(background_color, clear_rect);

        self.canvas().display(|builder, space_and_clip, scale| {
            if let Some(image) = &image {
                let image_display_rect = LayoutRect::new(
                    pos,
                    LayoutPoint::new(image.width as f32, image.height as f32),
                ) * Scale::new(scale);
                // render image
                builder.push_image(
                    &CommonItemProperties::new(image_clip_rect, space_and_clip),
                    image_display_rect,
                    ImageRendering::Auto,
                    AlphaType::Alpha,
                    image.image_key,
                    bitmap_color,
                );
            }
        });
    }

    fn draw_vertical_window_border(&mut self, face: Option<*mut Face>, x: i32, y0: i32, y1: i32) {
        // Fix the border height
        // Don't known why the height is short than expected.
        let y1 = y1 + 1;

        let color = match face {
            Some(f) => pixel_to_color(unsafe { (*f).foreground }),
            None => ColorF::BLACK,
        };

        let scale = self.canvas().scale();
        let visible_rect = (x, y0).by(1, y1 - y0, scale);
        self.draw_rectangle(color, visible_rect);
    }

    fn draw_window_divider(
        &mut self,
        color: u64,
        color_first: u64,
        color_last: u64,
        x0: i32,
        x1: i32,
        y0: i32,
        y1: i32,
    ) {
        let scale = self.canvas().scale();
        let (first, middle, last) = if (y1 - y0 > x1 - x0) && (x1 - x0 >= 3) {
            // A vertical divider, at least three pixels wide: Draw first and
            // last pixels differently.

            let first = (x0, y0).to(x0 + 1, y1, scale);
            let middle = (x0 + 1, y0).to(x1 - 1, y1, scale);
            let last = (x1 - 1, y0).to(x1, y1, scale);
            (Some(first), Some(middle), Some(last))
        } else if (x1 - x0 > y1 - y0) && (y1 - y0 >= 3) {
            // A horizontal divider, at least three pixels high: Draw first and
            // last pixels differently.

            let first = (x0, y0).to(x1, 1, scale);
            let middle = (x0, y0 + 1).to(x1, y1 - 1, scale);
            let last = (x0, y1 - 1).to(x1, y1, scale);
            (Some(first), Some(middle), Some(last))
        } else {
            // In any other case do not draw the first and last pixels
            // differently.
            let visible_rect = (x0, y0).to(x1, y1, scale);
            (None, Some(visible_rect), None)
        };
        if let Some(first) = first {
            self.draw_rectangle(pixel_to_color(color_first), first);
        }
        if let Some(middle) = middle {
            self.draw_rectangle(pixel_to_color(color), middle);
        }
        if let Some(last) = last {
            self.draw_rectangle(pixel_to_color(color_last), last);
        }
    }

    fn draw_rectangle(&mut self, clear_color: ColorF, rect: LayoutRect) {
        self.canvas().display(|builder, space_and_clip, _| {
            builder.push_rect(
                &CommonItemProperties::new(rect, space_and_clip),
                rect,
                clear_color,
            );
        });
    }

    fn clear_area(&mut self, clear_color: ColorF, x: i32, y: i32, width: i32, height: i32) {
        let scale = self.canvas().scale();
        let rect = (x, y).by(width, height, scale);
        self.draw_rectangle(clear_color, rect);
    }

    fn scroll(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        from_y: i32,
        to_y: i32,
        scroll_height: i32,
    ) {
        let bottom_y = y + height;

        let height = if to_y < from_y {
            // Scrolling up.  Make sure we don't copy part of the mode
            // line at the bottom.
            if (from_y + scroll_height) > bottom_y {
                bottom_y - from_y
            } else {
                scroll_height
            }
        } else {
            // Scrolling down.  Make sure we don't copy over the mode line.
            // at the bottom.
            if (to_y + scroll_height) > bottom_y {
                bottom_y - to_y
            } else {
                scroll_height
            }
        };

        // flush all content to screen before coping screen pixels
        self.canvas().flush();

        let diff_y = to_y - from_y;
        let frame_size = self.logical_size();

        if let Some(image_key) = self.canvas().get_previous_frame() {
            self.canvas().display(|builder, space_and_clip, scale| {
                let viewport = (x, to_y).by(width, height, scale);
                let new_frame_position =
                    (0, 0 + diff_y).by(frame_size.width as i32, frame_size.height as i32, scale);
                builder.push_image(
                    &CommonItemProperties::new(viewport, space_and_clip),
                    new_frame_position,
                    ImageRendering::Auto,
                    AlphaType::PremultipliedAlpha,
                    image_key,
                    ColorF::WHITE,
                );
            });
        }
    }

    fn draw_hollow_box_cursor(&mut self, cursor_rect: LayoutRect, clip_rect: LayoutRect) {
        let cursor_color = self.cursor_color();

        let border_widths = LayoutSideOffsets::new_all_same(1.0);

        let border_side = BorderSide {
            color: cursor_color,
            style: BorderStyle::Solid,
        };

        let border_details = BorderDetails::Normal(NormalBorder {
            top: border_side,
            right: border_side,
            bottom: border_side,
            left: border_side,
            radius: BorderRadius::uniform(0.0),
            do_aa: true,
        });

        self.canvas().display(|builder, space_and_clip, _scale| {
            builder.push_border(
                &CommonItemProperties::new(clip_rect, space_and_clip),
                cursor_rect,
                border_widths,
                border_details,
            );
        });
    }

    fn draw_bar_cursor(&mut self, face: &Face, x: i32, y: i32, width: i32, height: i32) {
        let cursor_color = if pixel_to_color(face.background) == self.cursor_color() {
            pixel_to_color(face.foreground)
        } else {
            self.cursor_color()
        };
        let scale = self.canvas().scale();
        let bounds = (x, y).by(width, height, scale);

        self.draw_rectangle(cursor_color, bounds);
    }
}

pub trait WrGlyph {
    fn x(self) -> i32;
    fn box_line_width(self) -> i32;
    fn font(self) -> WRFontRef;
    fn font_instance_key(self) -> FontInstanceKey;
    fn image(self) -> ImageRef;
    fn type_(self) -> glyph_type::Type;
    fn composite_p(self) -> bool;
    fn automatic_composite_p(self) -> bool;
    fn visible_height(self) -> i32;
    fn frame(self) -> LispFrameRef;
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

    fn frame(self) -> LispFrameRef {
        self.f.into()
    }

    fn font(self) -> WRFontRef {
        WRFontRef::new(self.font as *mut WRFont)
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
        let scale = self.frame().canvas().scale();
        self.frame()
            .canvas()
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

        for (i, index) in glyph_indices.into_iter().enumerate() {
            let previous_char_width = if i == 0 {
                0.0
            } else {
                let dimension = glyph_dimensions[i - 1];
                match dimension {
                    Some(d) => d as f32,
                    None => 0.0,
                }
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
                if self.composite_glyph(n as usize) == <u8 as Into<i64>>::into(b'\t') {
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
            let c = lglyph.char();
            if let Some(index) = self.font().glyph_for_char(c) {
                let glyph_instance = GlyphInstance {
                    index,
                    point: LayoutPoint::new(x as f32, y as f32),
                };
                log::warn!("automatic composite glyph instance {glyph_instance:?}");
                instances.push(glyph_instance);
            }
        };

        let mut x = self.x() as u32;

        let y = self.ybase;
        let mut width = 0;

        let cmp_from = self.cmp_from;
        let cmp_to = self.cmp_to;

        let mut i = cmp_from;
        let mut j = cmp_from;

        for n in cmp_from..cmp_to {
            let lglyph = lgstring.lglyph(n as u32);
            if lglyph.adjustment().is_nil() {
                width += lglyph.width();
            } else {
                if j < i {
                    composite_lglyph(lglyph, x as i32, y);
                    x += width as u32;
                }
                let xoff = lglyph.xoff() as u32;
                let yoff = lglyph.yoff() as u32;
                let wadjust = lglyph.wadjust() as u32;
                composite_lglyph(lglyph, x as i32 + xoff as i32, y + yoff as i32);

                x += wadjust;
                j = i + 1;
                width = 0;
            }
            i = i + 1;
        }
        if j < i {
            for n in j..i {
                let lglyph = lgstring.lglyph(n as u32);
                composite_lglyph(lglyph, x as i32, y);
                x += width as u32;
            }
        }

        instances
    }
}
