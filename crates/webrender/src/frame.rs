use super::util::HandyDandyRectBuilder;
use crate::fringe::FringeBitmap;
use crate::glyph::GlyphStringExtWr;
use crate::glyph::WrGlyph;
use crate::image::ImageExt;
use crate::image::ImageRef;
use crate::output::GlRenderer;
use crate::output::GlRendererRef;
use emacs_sys::color::pixel_to_color;
use emacs_sys::display_traits::DrawGlyphsFace;
use emacs_sys::display_traits::FaceRef;
use emacs_sys::display_traits::GlyphStringRef;
use emacs_sys::display_traits::GlyphType;
use emacs_sys::frame::FrameRef;
use euclid::Scale;
use std::cmp::min;
use std::ptr;
use webrender::api::units::*;
use webrender::api::*;

pub trait FrameExtWrCommon {
    fn gl_renderer(&self) -> GlRendererRef;
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

    fn draw_vertical_window_border(&mut self, face: Option<FaceRef>, x: i32, y0: i32, y1: i32);

    fn draw_window_divider(
        &mut self,
        color: ColorF,
        color_first: ColorF,
        color_last: ColorF,
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
    fn draw_bar_cursor(&mut self, face: Option<FaceRef>, x: i32, y: i32, width: i32, height: i32);
}

impl FrameExtWrCommon for FrameRef {
    fn gl_renderer(&self) -> GlRendererRef {
        if self.output().gl_renderer.is_null() {
            log::debug!("gl renderer data empty");
            let data = Box::new(GlRenderer::build(self.clone()));
            self.output().gl_renderer = Box::into_raw(data) as *mut libc::c_void;
        }

        GlRendererRef::new(self.output().gl_renderer as *mut GlRenderer)
    }

    fn free_gl_renderer_resources(&mut self) {
        let _ = unsafe { Box::from_raw(self.output().gl_renderer as *mut GlRenderer) };
        self.output().gl_renderer = ptr::null_mut();
    }

    fn draw_glyph_string(&mut self, mut s: GlyphStringRef) {
        // wip
        s.set_gc();

        match s.glyph_type() {
            GlyphType::Char => {
                if s.for_overlaps() != 0 {
                    s.set_background_filled_p(true);
                } else {
                    self.draw_glyph_string_background(s, false);
                }
                self.draw_char_glyph_string_foreground(s)
            }
            GlyphType::Stretch => self.draw_stretch_glyph_string_foreground(s),
            GlyphType::Image => self.draw_image_glyph(s),
            GlyphType::Composite => {
                if s.for_overlaps() != 0 || s.cmp_from > 0 && s.automatic_composite_p() {
                    s.set_background_filled_p(true);
                } else {
                    self.draw_glyph_string_background(s, true);
                }
                self.draw_composite_glyph_string_foreground(s)
            }
            GlyphType::Xwidget => {
                log::warn!("TODO unimplemented! GlyphType::XWIDGET_GLYPH\n")
            }
            GlyphType::Glyphless => {
                if s.for_overlaps() != 0 {
                    s.set_background_filled_p(true);
                } else {
                    self.draw_glyph_string_background(s, true);
                }

                self.draw_glyphless_glyph_string_foreground(s);
            }
        }

        if !s.is_for_overlaps() {
            // Draw underline
            s.draw_underline();

            // Draw overline
            if s.face().overline_p() {
                s.draw_overline();
            }

            /* Draw strike-through.  */
            if s.face().strike_through_p() {
                s.draw_strike_through();
            }
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
        let box_line_width = std::cmp::max(s.face().box_horizontal_line_width, 0);

        if s.stippled_p() {
            // Fill background with a stipple pattern.
            // fill_background (s, s.x, s.y + box_line_width,
            //     s.background_width,
            //     s.height - 2 * box_line_width);
            s.set_background_filled_p(true);
        } else if s.font_info().font.height < s.height - 2 * box_line_width
	    /* When xdisp.c ignores FONT_HEIGHT, we cannot trust
	    font dimensions, since the actual glyphs might be
	    much smaller.  So in that case we always clear the
	    rectangle with background color.  */
	    || s.font_info().too_high_p()
            || s.font_not_found_p()
            || s.extends_to_end_of_line_p() || force_p
        {
            let background_color = pixel_to_color(s.gc().background as u64);
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
        let x = s.x;
        let y = s.y;

        let visible_height = s.visible_height();

        // draw background
        let background_color = pixel_to_color(s.gc().background as u64);
        self.clear_area(background_color, x, y, s.background_width, visible_height);

        self.gl_renderer()
            .display(|builder, space_and_clip, scale| {
                let foreground_color = pixel_to_color(s.gc().foreground);

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
        let background_width = if s.hl() == DrawGlyphsFace::Cursor {
            let frame: FrameRef = s.f.into();

            min(frame.column_width, s.background_width)
        } else {
            s.background_width
        };

        let background_color = s.bg_color();
        self.clear_area(background_color, s.x, s.y, background_width, visible_height);

        s.set_background_filled_p(true);
    }

    fn draw_image_glyph(&mut self, mut s: GlyphStringRef) {
        // clear area
        let x = s.x;
        let y = s.y;
        let visible_height = s.visible_height();
        let background_color = s.bg_color();
        self.clear_area(background_color, x, y, s.background_width, visible_height);
        let clip_rect = s.clip_rect();

        let background_color = s.face().bg_color();
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
        println!("draw glyphless glyph string forground");
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
            let visible_height = s.visible_height();

            let x = s.x;
            let y = s.y;
            let background_color = pixel_to_color(s.gc().background as u64);
            self.clear_area(background_color, x, y, s.background_width, visible_height);
            self.gl_renderer()
                .display(|builder, space_and_clip, scale| {
                    let s = s.clone();

                    let foreground_color = pixel_to_color(s.gc().foreground);

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

    fn draw_vertical_window_border(&mut self, face: Option<FaceRef>, x: i32, y0: i32, y1: i32) {
        // Fix the border height
        // Don't known why the height is short than expected.
        let y1 = y1 + 1;

        let color = match face {
            Some(f) => f.fg_color(),
            None => ColorF::BLACK,
        };

        let scale = self.gl_renderer().scale();
        let visible_rect = (x, y0).by(1, y1 - y0, scale);
        self.draw_rectangle(color, visible_rect);
    }

    fn draw_window_divider(
        &mut self,
        color: ColorF,
        color_first: ColorF,
        color_last: ColorF,
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
            self.draw_rectangle(color_first, first);
        }
        if let Some(middle) = middle {
            self.draw_rectangle(color, middle);
        }
        if let Some(last) = last {
            self.draw_rectangle(color_last, last);
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

    fn draw_bar_cursor(&mut self, face: Option<FaceRef>, x: i32, y: i32, width: i32, height: i32) {
        let cursor_color = match face {
            Some(face) if face.bg_color() == self.cursor_color() => face.fg_color(),
            _ => self.cursor_color(),
        };

        let scale = self.gl_renderer().scale();
        let bounds = (x, y).by(width, height, scale);

        self.draw_rectangle(cursor_color, bounds);
    }
}
