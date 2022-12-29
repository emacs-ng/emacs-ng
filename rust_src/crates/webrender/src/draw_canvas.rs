use std::cmp::min;

use webrender::{self, api::units::*, api::*};

use crate::{frame::LispFrameExt, fringe::FringeBitmap, image::WrPixmap};

use super::{
    color::{color_to_pixel, pixel_to_color},
    font::{WRFont, WRFontRef},
    output::OutputRef,
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

impl OutputRef {
    pub fn canvas(self) -> DrawCanvas {
        DrawCanvas::new(self)
    }
}

pub struct DrawCanvas {
    output: OutputRef,
}

impl DrawCanvas {
    pub fn new(output: OutputRef) -> DrawCanvas {
        DrawCanvas { output }
    }

    pub fn draw_glyph_string(&mut self, mut s: GlyphStringRef) {
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
                let output = frame.wr_output();
                let dpyinfo = output.display_info();

                let mut foreground = face.background;
                let mut background = color_to_pixel(output.cursor_color);

                // If the glyph would be invisible, try a different foreground.
                if foreground == background {
                    foreground = face.foreground;
                }

                if foreground == background {
                    foreground = color_to_pixel(output.cursor_foreground_color);
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

            _ => panic!("invalid draw_glyphs_face"),
        }

        let type_ = s.first_glyph().type_();

        match type_ {
            glyph_type::CHAR_GLYPH => self.draw_char_glyph_string(s),
            glyph_type::STRETCH_GLYPH => self.draw_stretch_glyph_string(s),
            glyph_type::IMAGE_GLYPH => self.draw_image_glyph(s),
            glyph_type::COMPOSITE_GLYPH => self.draw_composite_glyph_string(s),
            _ => {}
        }
    }

    fn draw_char_glyph_string(&mut self, s: GlyphStringRef) {
        let font = WRFontRef::new(s.font as *mut WRFont);

        let x_start = s.x;
        let y_start = s.y + (font.font.ascent + (s.height - font.font.height) / 2);

        let from = 0 as usize;
        let to = s.nchars as usize;

        let gc = s.gc;

        self.output.display(|builder, space_and_clip| {
            let glyph_indices: Vec<u32> =
                s.get_chars()[from..to].iter().map(|c| *c as u32).collect();

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

            let mut s = s.clone();

            let x = s.x;
            let y = s.y;

            let face = s.face;

            let visible_height = if unsafe { (*s.row).mode_line_p() } {
                unsafe { (*s.row).height }
            } else {
                unsafe { (*s.row).visible_height }
            };

            // draw background
            if !s.background_filled_p() {
                let background_bounds = (x, y).by(s.background_width as i32, visible_height);

                let background_color = pixel_to_color(unsafe { (*gc).background } as u64);

                builder.push_rect(
                    &CommonItemProperties::new(background_bounds, space_and_clip),
                    background_bounds,
                    background_color,
                );

                s.set_background_filled_p(true);
            }

            let foreground_color = pixel_to_color(unsafe { (*gc).foreground });

            // draw underline
            if unsafe { (*face).underline() != face_underline_type::FACE_NO_UNDERLINE } {
                Self::draw_underline(builder, s, font, foreground_color, face, space_and_clip);
            }

            // draw foreground
            if !glyph_instances.is_empty() {
                let visible_rect = (x, y).by(s.width as i32, visible_height);

                builder.push_text(
                    &CommonItemProperties::new(visible_rect, space_and_clip),
                    visible_rect,
                    &glyph_instances,
                    font.font_instance_key,
                    foreground_color,
                    None,
                );
            }
        });
    }

    fn draw_stretch_glyph_string(&mut self, mut s: GlyphStringRef) {
        if s.background_filled_p() {
            return;
        }

        let visible_height = unsafe { (*s.row).visible_height };
        let background_width = if s.hl == draw_glyphs_face::DRAW_CURSOR {
            let frame: LispFrameRef = s.f.into();

            min(frame.column_width, s.background_width)
        } else {
            s.background_width
        };

        let background_bounds = (s.x, s.y).by(background_width, visible_height);
        let background_color = pixel_to_color(unsafe { (*s.gc).background } as u64);

        self.output.display(|builder, space_and_clip| {
            builder.push_rect(
                &CommonItemProperties::new(background_bounds, space_and_clip),
                background_bounds,
                background_color,
            );
        });

        s.set_background_filled_p(true);
    }

    fn draw_image_glyph(&mut self, mut s: GlyphStringRef) {
        let wr_pixmap = unsafe { (*s.img).pixmap } as *mut WrPixmap;

        // TODO null pixmap? 0x0
        if wr_pixmap.is_null() {
            return;
        }

        let image_key = unsafe { (*wr_pixmap).image_key };

        let mut clip_rect = Emacs_Rectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };

        unsafe { get_glyph_string_clip_rect(s.as_mut(), &mut clip_rect) };

        let clip_bounds =
            (clip_rect.x, clip_rect.y).by(clip_rect.width as i32, clip_rect.height as i32);
        let bounds = (s.x, s.y).by(s.slice.width() as i32, s.slice.height() as i32);

        let face = unsafe { &*s.face };

        let background_color = pixel_to_color(face.background);

        let background_rect = bounds.intersection(&clip_bounds);

        self.output.display(|builder, space_and_clip| {
            if let Some(background_rect) = background_rect {
                // render background
                builder.push_rect(
                    &CommonItemProperties::new(background_rect, space_and_clip),
                    background_rect,
                    background_color,
                );
            }

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

    fn draw_composite_glyph_string(&mut self, s: GlyphStringRef) {
        // S is a glyph string for a composition.  S->cmp_from is the index
        // of the first character drawn for glyphs of this composition.
        // S->cmp_from == 0 means we are drawing the very first character of
        // this composition

        // Draw a rectangle for the composition if the font for the very
        // first character of the composition could not be loaded.
        if s.font_not_found_p() {
            if s.cmp_from == 0 {
                self.clear_area(self.output.cursor_color, s.x, s.y, s.width, s.height);
            }
        } else if !unsafe { (*s.first_glyph).u.cmp.automatic() } {
            let font = WRFontRef::new(s.font as *mut WRFont);

            let x = if !s.face.is_null()
                && unsafe { (*s.face).box_() } != FACE_NO_BOX
                && unsafe { (*s.first_glyph).left_box_line_p() }
            {
                s.x + std::cmp::max(unsafe { (*s.face).box_vertical_line_width }, 0)
            } else {
                s.x
            };

            let y_start = s.y + (font.font.ascent + (s.height - font.font.height) / 2);

            let offsets = s.composite_offsets();

            let glyph_instances: Vec<GlyphInstance> = s
                .composite_chars()
                .into_iter()
                .enumerate()
                .filter_map(|(n, glyph)| {
                    // TAB in a composition means display glyphs with padding
                    // space on the left or right.
                    if s.composite_glyph(n as usize) == <u8 as Into<i64>>::into(b'\t') {
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

            let face = s.face;

            let gc = s.gc;

            let visible_height = unsafe {
                if (*s.row).mode_line_p() {
                    (*s.row).height
                } else {
                    (*s.row).visible_height
                }
            };

            self.output.display(|builder, space_and_clip| {
                let mut s = s.clone();

                let x = s.x;
                let y = s.y;

                // draw background
                if !s.background_filled_p() {
                    let background_bounds = (x, y).by(s.background_width as i32, visible_height);

                    let background_color = pixel_to_color(unsafe { (*gc).background } as u64);

                    builder.push_rect(
                        &CommonItemProperties::new(background_bounds, space_and_clip),
                        background_bounds,
                        background_color,
                    );

                    s.set_background_filled_p(true);
                }

                let foreground_color = pixel_to_color(unsafe { (*gc).foreground });

                // draw underline
                if unsafe { (*face).underline() != face_underline_type::FACE_NO_UNDERLINE } {
                    Self::draw_underline(builder, s, font, foreground_color, face, space_and_clip);
                }

                let visible_rect = (x, y).by(s.width, visible_height);

                // draw foreground
                if !glyph_instances.is_empty() {
                    builder.push_text(
                        &CommonItemProperties::new(visible_rect, space_and_clip),
                        visible_rect,
                        &glyph_instances,
                        font.font_instance_key,
                        foreground_color,
                        None,
                    );
                }
            });
        } else {
            log::error!("TODO unimplemented! draw_composite_glyph_string.\n");
        }
    }

    fn draw_underline(
        builder: &mut DisplayListBuilder,
        s: GlyphStringRef,
        font: WRFontRef,
        foreground_color: ColorF,
        face: *mut Face,
        space_and_clip: SpaceAndClipInfo,
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

        let visible_height = unsafe { (*s.row).visible_height };

        let info =
            CommonItemProperties::new((x, y).by(s.width as i32, visible_height), space_and_clip);

        let visible_rect = (x, position).by(s.width as i32, thickness);

        builder.push_line(
            &info,
            &visible_rect,
            1.0,
            LineOrientation::Horizontal,
            &underline_color,
            line_type,
        );
    }

    pub fn draw_fringe_bitmap(
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

        self.output.display(|builder, space_and_clip| {
            // clear area
            builder.push_rect(
                &CommonItemProperties::new(clear_rect, space_and_clip),
                clear_rect,
                background_color,
            );

            if let Some(image) = &image {
                let image_display_rect = LayoutRect::new(
                    pos,
                    LayoutPoint::new(image.width as f32, image.height as f32),
                );
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

    pub fn draw_vertical_window_border(
        &mut self,
        face: Option<*mut Face>,
        x: i32,
        y0: i32,
        y1: i32,
    ) {
        // Fix the border height
        // Don't known why the height is short than expected.
        let y1 = y1 + 1;

        let visible_rect = (x, y0).by(1, y1 - y0);

        let color = match face {
            Some(f) => pixel_to_color(unsafe { (*f).foreground }),
            None => ColorF::BLACK,
        };

        self.output.display(|builder, space_and_clip| {
            builder.push_rect(
                &CommonItemProperties::new(visible_rect, space_and_clip),
                visible_rect,
                color,
            );
        });
    }

    pub fn draw_window_divider(
        &mut self,
        color: u64,
        color_first: u64,
        color_last: u64,
        x0: i32,
        x1: i32,
        y0: i32,
        y1: i32,
    ) {
        self.output.display(|builder, space_and_clip| {
            if (y1 - y0 > x1 - x0) && (x1 - x0 >= 3) {
                // A vertical divider, at least three pixels wide: Draw first and
                // last pixels differently.

                let first = (x0, y0).to(x0 + 1, y1);
                let middle = (x0 + 1, y0).to(x1 - 1, y1);
                let last = (x1 - 1, y0).to(x1, y1);

                builder.push_rect(
                    &CommonItemProperties::new(first, space_and_clip),
                    first,
                    pixel_to_color(color_first),
                );
                builder.push_rect(
                    &CommonItemProperties::new(middle, space_and_clip),
                    middle,
                    pixel_to_color(color),
                );
                builder.push_rect(
                    &CommonItemProperties::new(last, space_and_clip),
                    last,
                    pixel_to_color(color_last),
                );
            } else if (x1 - x0 > y1 - y0) && (y1 - y0 >= 3) {
                // A horizontal divider, at least three pixels high: Draw first and
                // last pixels differently.

                let first = (x0, y0).to(x1, 1);
                let middle = (x0, y0 + 1).to(x1, y1 - 1);
                let last = (x0, y1 - 1).to(x1, y1);

                builder.push_rect(
                    &CommonItemProperties::new(first, space_and_clip),
                    first,
                    pixel_to_color(color_first),
                );
                builder.push_rect(
                    &CommonItemProperties::new(middle, space_and_clip),
                    middle,
                    pixel_to_color(color),
                );
                builder.push_rect(
                    &CommonItemProperties::new(last, space_and_clip),
                    last,
                    pixel_to_color(color_last),
                );
            } else {
                // In any other case do not draw the first and last pixels
                // differently.
                let visible_rect = (x0, y0).to(x1, y1);
                builder.push_rect(
                    &CommonItemProperties::new(visible_rect, space_and_clip),
                    visible_rect,
                    pixel_to_color(color),
                );
            }
        });
    }

    pub fn clear_area(&mut self, clear_color: ColorF, x: i32, y: i32, width: i32, height: i32) {
        let visible_rect = (x, y).by(width, height);

        self.output.display(|builder, space_and_clip| {
            builder.push_rect(
                &CommonItemProperties::new(visible_rect, space_and_clip),
                visible_rect,
                clear_color,
            );
        });
    }

    pub fn scroll(
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
        self.output.flush();

        let viewport = (x, to_y).by(width, height);

        let diff_y = to_y - from_y;
        let frame_size = self.output.get_inner_size();

        let new_frame_position =
            (0, 0 + diff_y).by(frame_size.width as i32, frame_size.height as i32);

        if let Some(image_key) = self.output.get_previous_frame() {
            self.output.display(|builder, space_and_clip| {
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

    pub fn draw_hollow_box_cursor(&mut self, cursor_rect: LayoutRect, clip_rect: LayoutRect) {
        let cursor_color = self.output.cursor_color;

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

        self.output.display(|builder, space_and_clip| {
            builder.push_border(
                &CommonItemProperties::new(clip_rect, space_and_clip),
                cursor_rect,
                border_widths,
                border_details,
            );
        });
    }

    pub fn draw_bar_cursor(&mut self, face: &Face, x: i32, y: i32, width: i32, height: i32) {
        let cursor_color = if pixel_to_color(face.background) == self.output.cursor_color {
            pixel_to_color(face.foreground)
        } else {
            self.output.cursor_color
        };

        let bounds = (x, y).by(width, height);
        self.output.display(|builder, space_and_clip| {
            builder.push_rect(
                &CommonItemProperties::new(bounds, space_and_clip),
                bounds,
                cursor_color,
            );
        });
    }
}
