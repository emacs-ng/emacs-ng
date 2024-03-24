use crate::frame::FrameExtWrCommon;
use crate::util::HandyDandyRectBuilder;
use emacs_sys::display_traits::GlyphRowArea;
use emacs_sys::display_traits::GlyphRowRef;
use emacs_sys::display_traits::GlyphType;
use emacs_sys::window::WindowRef;

pub fn draw_filled_cursor(window: WindowRef, row: GlyphRowRef) {
    window.draw_phys_cursor_glyph(row);
}

pub fn draw_hollow_box_cursor(window: WindowRef, row: GlyphRowRef) {
    let geometry = window.phys_cursor_geometry(row);
    if geometry.is_none() {
        return;
    }
    let (x, y, height) = geometry.unwrap();
    let width = window.phys_cursor_width;

    let mut frame = window.get_frame();
    let scale = frame.gl_renderer().scale();
    let cursor_rect = (x, y).by(width, height, scale);

    let window_rect = {
        let (x, y, width, height) = window.area_box(GlyphRowArea::Any);
        (x, y).by(width, height, scale)
    };

    frame.draw_hollow_box_cursor(cursor_rect, window_rect);
}

pub fn draw_bar_cursor(mut window: WindowRef, row: GlyphRowRef, cursor_width: i32, is_hbar: bool) {
    let mut frame = window.get_frame();

    let cursor_glyph = window.phys_cursor_glyph();

    if cursor_glyph.is_null() {
        return;
    }

    let glyph_type = cursor_glyph.glyph_type();
    if glyph_type == GlyphType::Xwidget || glyph_type == GlyphType::Image {
        return;
    }

    let face = frame.face_from_id(cursor_glyph.face_id2());

    let cursor_glyph_width = cursor_glyph.pixel_width as i32;

    let (x, y, width, height) = if !is_hbar {
        let mut x = window.text_to_frame_pixel_x(window.phys_cursor.x);
        let y = window.frame_pixel_y(window.phys_cursor.y);

        let width = if cursor_width < 0 {
            frame.cursor_width
        } else {
            cursor_width
        };

        let width = std::cmp::min(cursor_glyph_width, width);

        window.phys_cursor_width = width;
        // If the character under cursor is R2L, draw the bar cursor
        //  on the right of its glyph, rather than on the left.
        if (cursor_glyph.resolved_level() & 1) != 0 {
            x += cursor_glyph_width - width;
        }

        let height = row.height;

        (x, y, width, height)
    } else {
        let row_height = row.height as i32;
        let mut x = window.text_to_frame_pixel_x(window.phys_cursor.x);

        let height = if cursor_width < 0 {
            row.height
        } else {
            cursor_width
        };

        let height = std::cmp::min(row_height, height);

        if (cursor_glyph.resolved_level() & 1) != 0
            && cursor_glyph_width > window.phys_cursor_width - 1
        {
            x += cursor_glyph_width - window.phys_cursor_width + 1;
        }

        let y = window.frame_pixel_y(window.phys_cursor.y + row_height - height);

        let width = window.phys_cursor_width - 1;

        (x, y, width, height)
    };

    frame.draw_bar_cursor(face, x, y, width, height);
}
