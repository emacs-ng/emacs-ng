use crate::{frame::LispFrameExt, util::HandyDandyRectBuilder};

use winit::window::CursorIcon;

use emacs::{
    bindings::{
        draw_glyphs_face, draw_phys_cursor_glyph, get_phys_cursor_geometry, get_phys_cursor_glyph,
        glyph_row, glyph_row_area, glyph_type, Emacs_Cursor,
    },
    window::LispWindowRef,
};

pub fn draw_filled_cursor(mut window: LispWindowRef, row: *mut glyph_row) {
    unsafe { draw_phys_cursor_glyph(window.as_mut(), row, draw_glyphs_face::DRAW_CURSOR) };
}

pub fn draw_hollow_box_cursor(mut window: LispWindowRef, row: *mut glyph_row) {
    let cursor_glyph = unsafe { get_phys_cursor_glyph(window.as_mut()) };

    if cursor_glyph.is_null() {
        return;
    }

    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut height: i32 = 0;
    unsafe {
        get_phys_cursor_geometry(
            window.as_mut(),
            row,
            cursor_glyph,
            &mut x,
            &mut y,
            &mut height,
        )
    };
    let width = window.phys_cursor_width;

    let frame = window.get_frame();
    let output = frame.wr_output();

    let cursor_rect = (x, y).by(width, height);

    let window_rect = {
        let (x, y, width, height) = window.area_box(glyph_row_area::ANY_AREA);
        (x, y).by(width, height)
    };

    output
        .canvas()
        .draw_hollow_box_cursor(cursor_rect, window_rect);
}

pub fn draw_bar_cursor(
    mut window: LispWindowRef,
    row: *mut glyph_row,
    cursor_width: i32,
    is_hbar: bool,
) {
    let frame = window.get_frame();
    let output = frame.wr_output();

    let cursor_glyph = unsafe { get_phys_cursor_glyph(window.as_mut()) };

    if cursor_glyph.is_null() {
        return;
    }

    if unsafe {
        (*cursor_glyph).type_() == glyph_type::XWIDGET_GLYPH
            || (*cursor_glyph).type_() == glyph_type::IMAGE_GLYPH
    } {
        return;
    }

    let face = unsafe {
        let face_id = (*cursor_glyph).face_id();
        let face_id = std::mem::transmute::<u32, emacs::bindings::face_id>(face_id);

        &*frame.face_from_id(face_id).unwrap()
    };

    let (x, y, width, height) = if !is_hbar {
        let mut x = window.text_to_frame_pixel_x(window.phys_cursor.x);
        let y = window.frame_pixel_y(window.phys_cursor.y);

        let width = if cursor_width < 0 {
            frame.cursor_width
        } else {
            cursor_width
        };

        let width = std::cmp::min(unsafe { (*cursor_glyph).pixel_width } as i32, width);

        window.phys_cursor_width = width;
        // If the character under cursor is R2L, draw the bar cursor
        //  on the right of its glyph, rather than on the left.
        if (unsafe { (*cursor_glyph).resolved_level() } & 1) != 0 {
            x += unsafe { (*cursor_glyph).pixel_width } as i32 - width;
        }

        let height = unsafe { (*row).height };

        (x, y, width, height)
    } else {
        let row_height = unsafe { (*row).height } as i32;
        let mut x = window.text_to_frame_pixel_x(window.phys_cursor.x);

        let height = if cursor_width < 0 {
            unsafe { (*row).height }
        } else {
            cursor_width
        };

        let height = std::cmp::min(row_height, height);

        if (unsafe { (*cursor_glyph).resolved_level() } & 1) != 0
            && unsafe { (*cursor_glyph).pixel_width } as i32 > window.phys_cursor_width - 1
        {
            x += unsafe { (*cursor_glyph).pixel_width } as i32 - window.phys_cursor_width + 1;
        }

        let y = window.frame_pixel_y(window.phys_cursor.y + row_height - height);

        let width = window.phys_cursor_width - 1;

        (x, y, width, height)
    };

    output.canvas().draw_bar_cursor(face, x, y, width, height);
}

pub fn winit_to_emacs_cursor(cursor: CursorIcon) -> Emacs_Cursor {
    // 0 for No_Cursor
    let emacs_cursor = match cursor {
        CursorIcon::Default => 1,
        CursorIcon::Crosshair => 2,
        CursorIcon::Hand => 3,
        CursorIcon::Arrow => 4,
        CursorIcon::Move => 5,
        CursorIcon::Text => 6,
        CursorIcon::Wait => 7,
        CursorIcon::Help => 8,
        CursorIcon::Progress => 9,
        CursorIcon::NotAllowed => 10,
        CursorIcon::ContextMenu => 11,
        CursorIcon::Cell => 12,
        CursorIcon::VerticalText => 13,
        CursorIcon::Alias => 14,
        CursorIcon::Copy => 15,
        CursorIcon::NoDrop => 16,
        CursorIcon::Grab => 17,
        CursorIcon::Grabbing => 18,
        CursorIcon::AllScroll => 19,
        CursorIcon::ZoomIn => 20,
        CursorIcon::ZoomOut => 21,
        CursorIcon::EResize => 22,
        CursorIcon::NResize => 23,
        CursorIcon::NeResize => 24,
        CursorIcon::NwResize => 25,
        CursorIcon::SResize => 26,
        CursorIcon::SeResize => 27,
        CursorIcon::SwResize => 28,
        CursorIcon::WResize => 29,
        CursorIcon::EwResize => 30,
        CursorIcon::NsResize => 31,
        CursorIcon::NeswResize => 32,
        CursorIcon::NwseResize => 33,
        CursorIcon::ColResize => 34,
        CursorIcon::RowResize => 35,
    };

    emacs_cursor as *mut ::libc::c_int
}

pub fn emacs_to_winit_cursor(cursor: Emacs_Cursor) -> CursorIcon {
    let cursor = cursor as *const _ as usize;

    // 0 for No_Cursor
    match cursor {
        1 => CursorIcon::Default,
        2 => CursorIcon::Crosshair,
        3 => CursorIcon::Hand,
        4 => CursorIcon::Arrow,
        5 => CursorIcon::Move,
        6 => CursorIcon::Text,
        7 => CursorIcon::Wait,
        8 => CursorIcon::Help,
        9 => CursorIcon::Progress,
        10 => CursorIcon::NotAllowed,
        11 => CursorIcon::ContextMenu,
        12 => CursorIcon::Cell,
        13 => CursorIcon::VerticalText,
        14 => CursorIcon::Alias,
        15 => CursorIcon::Copy,
        16 => CursorIcon::NoDrop,
        17 => CursorIcon::Grab,
        18 => CursorIcon::Grabbing,
        19 => CursorIcon::AllScroll,
        20 => CursorIcon::ZoomIn,
        21 => CursorIcon::ZoomOut,
        22 => CursorIcon::EResize,
        23 => CursorIcon::NResize,
        24 => CursorIcon::NeResize,
        25 => CursorIcon::NwResize,
        26 => CursorIcon::SResize,
        27 => CursorIcon::SeResize,
        28 => CursorIcon::SwResize,
        29 => CursorIcon::WResize,
        30 => CursorIcon::EwResize,
        31 => CursorIcon::NsResize,
        32 => CursorIcon::NeswResize,
        33 => CursorIcon::NwseResize,
        34 => CursorIcon::ColResize,
        35 => CursorIcon::RowResize,
        _ => panic!("Not invalie Emacs_Cursor"),
    }
}
