use std::ptr;

use lazy_static::lazy_static;

use super::display_info::{DisplayInfo, DisplayInfoRef};

use lisp::{
    keyboard::allocate_keyboard,
    lisp::{ExternalPtr, LispObject},
    remacs_sys::{
        create_terminal, current_kboard, frame_parm_handler, glyph_row, glyph_string,
        initial_kboard, output_method, redisplay_interface, terminal, text_cursor_kinds,
        xlispstrdup, Fcons, Lisp_Frame, Lisp_Window, Qnil, Qwr,
    },
    remacs_sys::{
        gui_clear_end_of_line, gui_clear_window_mouse_face, gui_fix_overlapping_area,
        gui_get_glyph_overhangs, gui_produce_glyphs, gui_set_font, gui_set_font_backend,
        gui_write_glyphs,
    },
};

pub type TerminalRef = ExternalPtr<terminal>;

fn get_frame_parm_handlers() -> [frame_parm_handler; 47] {
    // Keep this list in the same order as frame_parms in frame.c.
    // Use None for unsupported frame parameters.
    let handlers: [frame_parm_handler; 47] = [
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(gui_set_font),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(gui_set_font_backend),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ];

    handlers
}

struct RedisplayInterface(pub redisplay_interface);
unsafe impl Sync for RedisplayInterface {}

lazy_static! {
    static ref REDISPLAY_INTERFACE: RedisplayInterface = {
        let frame_parm_handlers = Box::new(get_frame_parm_handlers());

        let interface = redisplay_interface {
            frame_parm_handlers: (Box::into_raw(frame_parm_handlers)) as *mut Option<_>,
            produce_glyphs: Some(gui_produce_glyphs),
            write_glyphs: Some(gui_write_glyphs),
            insert_glyphs: None,
            clear_end_of_line: Some(gui_clear_end_of_line),
            clear_under_internal_border: None,
            scroll_run_hook: None,
            after_update_window_line_hook: Some(after_update_window_line),
            update_window_begin_hook: Some(update_window_begin),
            update_window_end_hook: Some(update_window_end),
            flush_display: None,
            clear_window_mouse_face: Some(gui_clear_window_mouse_face),
            get_glyph_overhangs: Some(gui_get_glyph_overhangs),
            fix_overlapping_area: Some(gui_fix_overlapping_area),
            draw_fringe_bitmap: None,
            define_fringe_bitmap: None,
            destroy_fringe_bitmap: None,
            compute_glyph_string_overhangs: None,
            draw_glyph_string: Some(draw_glyph_string),
            define_frame_cursor: None,
            default_font_parameter: None,
            clear_frame_area: Some(clear_frame_area),
            draw_window_cursor: Some(draw_window_cursor),
            draw_vertical_window_border: None,
            draw_window_divider: None,
            shift_glyphs_for_insert: None,
            show_hourglass: None,
            hide_hourglass: None,
        };

        RedisplayInterface(interface)
    };
}

#[allow(unused_variables)]
extern "C" fn update_window_begin(w: *mut Lisp_Window) {}

#[allow(unused_variables)]
extern "C" fn update_window_end(
    w: *mut Lisp_Window,
    cursor_no_p: bool,
    mouse_face_overwritten_p: bool,
) {
}

#[allow(unused_variables)]
extern "C" fn after_update_window_line(w: *mut Lisp_Window, desired_row: *mut glyph_row) {}

#[allow(unused_variables)]
extern "C" fn draw_glyph_string(s: *mut glyph_string) {}

#[allow(unused_variables)]
extern "C" fn clear_frame_area(s: *mut Lisp_Frame, x: i32, y: i32, width: i32, height: i32) {}

extern "C" fn draw_window_cursor(
    _window: *mut Lisp_Window,
    _row: *mut glyph_row,
    _x: i32,
    _y: i32,
    _cursor_type: text_cursor_kinds::Type,
    _cursor_width: i32,
    _on_p: bool,
    _active_p: bool,
) {
}

extern "C" fn get_string_resource(
    _rdb: *mut libc::c_void,
    _name: *const libc::c_char,
    _class: *const libc::c_char,
) -> *const libc::c_char {
    ptr::null()
}

fn wr_create_terminal(mut dpyinfo: DisplayInfoRef) -> TerminalRef {
    let terminal_ptr = unsafe {
        create_terminal(
            output_method::output_wr,
            &REDISPLAY_INTERFACE.0 as *const _ as *mut _,
        )
    };
    let mut terminal = TerminalRef::new(terminal_ptr);

    // Link terminal and dpyinfo together
    terminal.display_info.wr = dpyinfo.get_raw().as_mut();
    dpyinfo.get_inner().terminal = terminal;
    dpyinfo.get_raw().terminal = terminal.as_mut();

    //TODO: add terminal hook
    // Other hooks are NULL by default.
    terminal.get_string_resource_hook = Some(get_string_resource);

    terminal
}

pub fn wr_term_init(display_name: LispObject) -> DisplayInfoRef {
    let dpyinfo = Box::new(DisplayInfo::new());
    let mut dpyinfo_ref = DisplayInfoRef::new(Box::into_raw(dpyinfo));

    let mut terminal = wr_create_terminal(dpyinfo_ref);

    let mut kboard = allocate_keyboard(Qwr);

    terminal.kboard = kboard.as_mut();

    // Don't let the initial kboard remain current longer than necessary.
    // That would cause problems if a file loaded on startup tries to
    // prompt in the mini-buffer.
    unsafe {
        if current_kboard == initial_kboard {
            current_kboard = terminal.kboard;
        }
    }

    kboard.add_ref();

    {
        let mut dpyinfo_ref = dpyinfo_ref.get_raw();
        dpyinfo_ref.name_list_element = unsafe { Fcons(display_name, Qnil) };

        // https://lists.gnu.org/archive/html/emacs-devel/2015-11/msg00194.html
        dpyinfo_ref.smallest_font_height = 1;
        dpyinfo_ref.smallest_char_width = 1;
    }

    // Set the name of the terminal.
    terminal.name = unsafe { xlispstrdup(display_name) };

    dpyinfo_ref
}
