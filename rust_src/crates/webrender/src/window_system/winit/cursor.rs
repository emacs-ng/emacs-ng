use crate::window_system::api::window::CursorIcon;

use emacs::bindings::{winit_output, Emacs_Cursor};

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
        #[cfg(use_tao)]
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
        _ => {
            log::error!("Unhandled Emacs_Cursor {cursor:?}");
            CursorIcon::Default
        }
    }
}

pub fn build_mouse_cursors(output: &mut winit_output) {
    output.text_cursor = winit_to_emacs_cursor(CursorIcon::Text);
    output.nontext_cursor = winit_to_emacs_cursor(CursorIcon::Arrow);
    output.modeline_cursor = winit_to_emacs_cursor(CursorIcon::Hand);
    output.hand_cursor = winit_to_emacs_cursor(CursorIcon::Hand);
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
