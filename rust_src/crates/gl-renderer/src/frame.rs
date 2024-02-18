use crate::display_info::DisplayInfoExtGlRenderer;
use crate::image::ImageExt;
use crate::image::ImageRef;
use euclid::Scale;
use std::cmp::min;
use std::ptr;

use webrender::{self, api::units::*, api::*};

use crate::fringe::FringeBitmap;

use super::util::HandyDandyRectBuilder;
use emacs::color::{color_to_pixel, pixel_to_color};
use font::WRFontRef;

use emacs::{
    bindings::{
        draw_glyphs_face, face as Face, face_underline_type, get_glyph_string_clip_rect,
        glyph_type, prepare_face_for_display, Emacs_Rectangle,
    },
    glyph::GlyphStringRef,
};

use crate::gl::context::GLContextTrait;
use crate::glyph::WrGlyph;
use crate::output::GlRenderer;
use crate::output::GlRendererRef;
use emacs::frame::FrameRef;

pub trait FrameExtGlRenderer {
    fn cursor_color(&self) -> ColorF;
    fn cursor_foreground_color(&self) -> ColorF;
    fn scale_factor(&self) -> f64;
}

pub trait FrameExtGlRendererCommon {
    fn gl_renderer(&self) -> GlRendererRef;
    fn logical_size(&self) -> LayoutSize;
    fn physical_size(&self) -> DeviceIntSize;
    fn create_gl_context(&self) -> crate::gl::context::GLContext;
    fn free_gl_renderer_resources(&mut self);
    fn draw_glyph_string(&mut self, s: GlyphStringRef);

    fn draw_glyph_string_background(&mut self, s: GlyphStringRef, force_p: bool);

    fn draw_char_glyph_string_foreground(&mut self, s: GlyphStringRef);

    fn draw_stretch_glyph_string_foreground(&mut self, s: GlyphStringRef);

    fn draw_glyphless_glyph_string_foreground(&mut self, s: GlyphStringRef);

    fn draw_image_glyph(&mut self, s: GlyphStringRef);

    fn draw_image(
        &mut self,
        image_key: ImageKey,
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

impl FrameExtGlRendererCommon for FrameRef {
    fn logical_size(&self) -> LayoutSize {
        LayoutSize::new(self.pixel_width as f32, self.pixel_height as f32)
    }

    fn physical_size(&self) -> DeviceIntSize {
        let size = self.logical_size() * euclid::Scale::new(self.scale_factor() as f32);
        size.to_i32()
    }

    fn gl_renderer(&self) -> GlRendererRef {
        if self.output().gl_renderer.is_null() {
            log::debug!("gl renderer data empty");
            let data = Box::new(GlRenderer::build(self.clone()));
            self.output().gl_renderer = Box::into_raw(data) as *mut libc::c_void;
        }

        GlRendererRef::new(self.output().gl_renderer as *mut GlRenderer)
    }

    fn create_gl_context(&self) -> crate::gl::context::GLContext {
        crate::gl::context::GLContext::build(self)
    }

    fn free_gl_renderer_resources(&mut self) {
        let _ = unsafe { Box::from_raw(self.output().gl_renderer as *mut GlRenderer) };
        self.output().gl_renderer = ptr::null_mut();
    }

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
                let frame: FrameRef = (*s).f.into();
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

                let gc = &mut dpyinfo.gl_renderer_data().scratch_cursor_gc;
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

        self.gl_renderer()
            .display(|builder, space_and_clip, scale| {
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
            let frame: FrameRef = s.f.into();

            min(frame.column_width, s.background_width)
        } else {
            s.background_width
        };

        let background_color = pixel_to_color(unsafe { (*s.gc).background } as u64);
        self.clear_area(background_color, s.x, s.y, background_width, visible_height);

        s.set_background_filled_p(true);
    }

    fn draw_image_glyph(&mut self, mut s: GlyphStringRef) {
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
        let scale = s.frame().gl_renderer().scale();
        let clip_bounds =
            (clip_rect.x, clip_rect.y).by(clip_rect.width as i32, clip_rect.height as i32, scale);
        let bounds = (s.x, s.y).by(s.slice.width() as i32, s.slice.height() as i32, scale);

        // render background
        let background_rect = bounds.intersection(&clip_bounds);
        if let Some(background_rect) = background_rect {
            self.draw_rectangle(background_color, background_rect);
        }

        let image: ImageRef = s.img.into();
        let frame: FrameRef = s.f.into();
        if let Some((image_key, descriptor)) = image.meta(frame) {
            //s.img viewbox is the layout size we draw to
            //we don't crop image to fix into viewbox
            //So the width of image uploaded to WebRender could be bigger than viewbox
            //We do scaling here to avoid stretching
            let dwidth = s.slice.height() as f32 / descriptor.size.height as f32
                * descriptor.size.width as f32;
            let bounds = (s.x, s.y).by(dwidth as i32, s.slice.height() as i32, scale);
            self.draw_image(image_key, bounds, Some(clip_bounds));
        }
    }

    fn draw_glyphless_glyph_string_foreground(&mut self, s: GlyphStringRef) {
        let _x = s.x();
        //TODO
    }

    fn draw_image(
        &mut self,
        image_key: ImageKey,
        bounds: LayoutRect,
        clip_bounds: Option<LayoutRect>,
    ) {
        let clip_bounds = clip_bounds.unwrap_or(bounds);
        self.gl_renderer()
            .display(|builder, space_and_clip, _scale| {
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
            self.gl_renderer()
                .display(|builder, space_and_clip, scale| {
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

        self.gl_renderer()
            .display(|builder, space_and_clip, scale| {
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

        let scale = self.gl_renderer().scale();
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
        let scale = self.gl_renderer().scale();
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
        self.gl_renderer().display(|builder, space_and_clip, _| {
            builder.push_rect(
                &CommonItemProperties::new(rect, space_and_clip),
                rect,
                clear_color,
            );
        });
    }

    fn clear_area(&mut self, clear_color: ColorF, x: i32, y: i32, width: i32, height: i32) {
        let scale = self.gl_renderer().scale();
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
        self.gl_renderer().flush();

        let diff_y = to_y - from_y;
        let frame_size = self.logical_size();

        if let Some(image_key) = self.gl_renderer().get_previous_frame() {
            self.gl_renderer()
                .display(|builder, space_and_clip, scale| {
                    let viewport = (x, to_y).by(width, height, scale);
                    let new_frame_position = (0, 0 + diff_y).by(
                        frame_size.width as i32,
                        frame_size.height as i32,
                        scale,
                    );
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

        self.gl_renderer()
            .display(|builder, space_and_clip, _scale| {
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
        let scale = self.gl_renderer().scale();
        let bounds = (x, y).by(width, height, scale);

        self.draw_rectangle(cursor_color, bounds);
    }
}
