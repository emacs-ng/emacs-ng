use winit::window::CursorIcon;

use emacs_sys::bindings::winit_output;
use emacs_sys::bindings::Emacs_Cursor;

pub fn winit_to_emacs_cursor(cursor: CursorIcon) -> Emacs_Cursor {
    // 0 for No_Cursor
    let emacs_cursor = match cursor {
        CursorIcon::Default => 1,
        CursorIcon::Crosshair => 2,
        CursorIcon::Pointer => 3,
        CursorIcon::Move => 4,
        CursorIcon::Text => 5,
        CursorIcon::Wait => 6,
        CursorIcon::Help => 7,
        CursorIcon::Progress => 8,
        CursorIcon::NotAllowed => 9,
        CursorIcon::ContextMenu => 10,
        CursorIcon::Cell => 11,
        CursorIcon::VerticalText => 12,
        CursorIcon::Alias => 13,
        CursorIcon::Copy => 14,
        CursorIcon::NoDrop => 15,
        CursorIcon::Grab => 16,
        CursorIcon::Grabbing => 17,
        CursorIcon::AllScroll => 18,
        CursorIcon::ZoomIn => 19,
        CursorIcon::ZoomOut => 20,
        CursorIcon::EResize => 21,
        CursorIcon::NResize => 22,
        CursorIcon::NeResize => 23,
        CursorIcon::NwResize => 24,
        CursorIcon::SResize => 25,
        CursorIcon::SeResize => 26,
        CursorIcon::SwResize => 27,
        CursorIcon::WResize => 28,
        CursorIcon::EwResize => 29,
        CursorIcon::NsResize => 30,
        CursorIcon::NeswResize => 31,
        CursorIcon::NwseResize => 32,
        CursorIcon::ColResize => 33,
        CursorIcon::RowResize => 34,
        _ => todo!(),
    };

    emacs_cursor as *mut ::libc::c_int
}

pub fn emacs_to_winit_cursor(cursor: Emacs_Cursor) -> CursorIcon {
    let cursor = cursor as *const _ as usize;

    // 0 for No_Cursor
    match cursor {
        1 => CursorIcon::Default,
        2 => CursorIcon::Crosshair,
        3 => CursorIcon::Pointer,
        4 => CursorIcon::Move,
        5 => CursorIcon::Text,
        6 => CursorIcon::Wait,
        7 => CursorIcon::Help,
        8 => CursorIcon::Progress,
        9 => CursorIcon::NotAllowed,
        10 => CursorIcon::ContextMenu,
        11 => CursorIcon::Cell,
        12 => CursorIcon::VerticalText,
        13 => CursorIcon::Alias,
        14 => CursorIcon::Copy,
        15 => CursorIcon::NoDrop,
        16 => CursorIcon::Grab,
        17 => CursorIcon::Grabbing,
        18 => CursorIcon::AllScroll,
        19 => CursorIcon::ZoomIn,
        20 => CursorIcon::ZoomOut,
        21 => CursorIcon::EResize,
        22 => CursorIcon::NResize,
        23 => CursorIcon::NeResize,
        24 => CursorIcon::NwResize,
        25 => CursorIcon::SResize,
        26 => CursorIcon::SeResize,
        27 => CursorIcon::SwResize,
        28 => CursorIcon::WResize,
        29 => CursorIcon::EwResize,
        30 => CursorIcon::NsResize,
        31 => CursorIcon::NeswResize,
        32 => CursorIcon::NwseResize,
        33 => CursorIcon::ColResize,
        34 => CursorIcon::RowResize,
        _ => {
            log::error!("Unhandled Emacs_Cursor {cursor:?}");
            CursorIcon::Default
        }
    }
}

pub fn build_mouse_cursors(output: &mut winit_output) {
    output.text_cursor = winit_to_emacs_cursor(CursorIcon::Text);
    output.nontext_cursor = winit_to_emacs_cursor(CursorIcon::Default);
    output.modeline_cursor = winit_to_emacs_cursor(CursorIcon::Pointer);
    output.hand_cursor = winit_to_emacs_cursor(CursorIcon::Pointer);
    output.hourglass_cursor = winit_to_emacs_cursor(CursorIcon::Progress);

    output.horizontal_drag_cursor = winit_to_emacs_cursor(CursorIcon::ColResize);
    output.vertical_drag_cursor = winit_to_emacs_cursor(CursorIcon::RowResize);

    output.left_edge_cursor = winit_to_emacs_cursor(CursorIcon::WResize);
    output.right_edge_cursor = winit_to_emacs_cursor(CursorIcon::EResize);
    output.top_edge_cursor = winit_to_emacs_cursor(CursorIcon::NResize);
    output.bottom_edge_cursor = winit_to_emacs_cursor(CursorIcon::SResize);

    output.top_left_corner_cursor = winit_to_emacs_cursor(CursorIcon::NwResize);
    output.top_right_corner_cursor = winit_to_emacs_cursor(CursorIcon::NeResize);

    output.bottom_left_corner_cursor = winit_to_emacs_cursor(CursorIcon::SwResize);
    output.bottom_right_corner_cursor = winit_to_emacs_cursor(CursorIcon::SeResize);
}
