use std::ffi::CString;
use std::ptr;

use lazy_static::lazy_static;

use webrender::api::*;

use super::{
    color::{color_to_pixel, color_to_xcolor, lookup_color_by_name_or_hex},
    display_info::{DisplayInfo, DisplayInfoRef},
    output::OutputRef,
};

use lisp::{
    font::LispFontRef,
    frame::LispFrameRef,
    glyph::GlyphStringRef,
    keyboard::allocate_keyboard,
    lisp::{ExternalPtr, LispObject},
    remacs_sys::{
        block_input, face_id, gui_clear_end_of_line, gui_clear_window_mouse_face,
        gui_draw_right_divider, gui_draw_vertical_border, gui_fix_overlapping_area,
        gui_get_glyph_overhangs, gui_produce_glyphs, gui_set_bottom_divider_width, gui_set_font,
        gui_set_font_backend, gui_set_left_fringe, gui_set_right_divider_width,
        gui_set_right_fringe, gui_write_glyphs, unblock_input, update_face_from_frame_parameter,
    },
    remacs_sys::{
        create_terminal, current_kboard, draw_fringe_bitmap_params, fontset_from_font,
        frame_parm_handler, glyph_row, glyph_string, initial_kboard, output_method,
        redisplay_interface, terminal, text_cursor_kinds, xlispstrdup, Emacs_Color, Fcons,
        Fredraw_frame, Lisp_Frame, Lisp_Window, Qbackground_color, Qnil, Qwr,
    },
    window::LispWindowRef,
};

pub type TerminalRef = ExternalPtr<terminal>;

fn get_frame_parm_handlers() -> [frame_parm_handler; 47] {
    // Keep this list in the same order as frame_parms in frame.c.
    // Use None for unsupported frame parameters.
    let handlers: [frame_parm_handler; 47] = [
        None,
        None,
        Some(set_background_color),
        None,
        None,
        None,
        None,
        Some(gui_set_font),
        None,
        None,
        None,
        None,
        Some(gui_set_right_divider_width),
        Some(gui_set_bottom_divider_width),
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
        Some(gui_set_left_fringe),
        Some(gui_set_right_fringe),
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
            flush_display: Some(flush_display),
            clear_window_mouse_face: Some(gui_clear_window_mouse_face),
            get_glyph_overhangs: Some(gui_get_glyph_overhangs),
            fix_overlapping_area: Some(gui_fix_overlapping_area),
            draw_fringe_bitmap: Some(draw_fringe_bitmap),
            define_fringe_bitmap: None,
            destroy_fringe_bitmap: None,
            compute_glyph_string_overhangs: None,
            draw_glyph_string: Some(draw_glyph_string),
            define_frame_cursor: None,
            default_font_parameter: None,
            clear_frame_area: Some(clear_frame_area),
            draw_window_cursor: Some(draw_window_cursor),
            draw_vertical_window_border: Some(draw_vertical_window_border),
            draw_window_divider: Some(draw_window_divider),
            shift_glyphs_for_insert: None,
            show_hourglass: None,
            hide_hourglass: None,
        };

        RedisplayInterface(interface)
    };
}

#[allow(unused_variables)]
extern "C" fn update_window_begin(w: *mut Lisp_Window) {}

extern "C" fn update_window_end(
    window: *mut Lisp_Window,
    _cursor_no_p: bool,
    _mouse_face_overwritten_p: bool,
) {
    let mut window: LispWindowRef = window.into();

    if window.pseudo_window_p() {
        return;
    }

    unsafe { block_input() };
    if window.right_divider_width() > 0 {
        unsafe { gui_draw_right_divider(window.as_mut()) }
    } else {
        unsafe { gui_draw_vertical_border(window.as_mut()) }
    }
    unsafe { unblock_input() };
}

extern "C" fn flush_display(f: *mut Lisp_Frame) {
    let frame: LispFrameRef = f.into();
    let mut output: OutputRef = unsafe { frame.output_data.wr.into() };

    output.flush();
}

#[allow(unused_variables)]
extern "C" fn after_update_window_line(w: *mut Lisp_Window, desired_row: *mut glyph_row) {}

#[allow(unused_variables)]
extern "C" fn draw_glyph_string(s: *mut glyph_string) {
    let s: GlyphStringRef = s.into();

    let output: OutputRef = {
        let frame: LispFrameRef = s.f.into();
        unsafe { frame.output_data.wr.into() }
    };

    output.canvas().draw_glyph_string(s);
}

extern "C" fn draw_fringe_bitmap(
    window: *mut Lisp_Window,
    row: *mut glyph_row,
    p: *mut draw_fringe_bitmap_params,
) {
    let window: LispWindowRef = window.into();
    let frame: LispFrameRef = window.get_frame();

    let output: OutputRef = unsafe { frame.output_data.wr.into() };

    output.canvas().draw_fringe_bitmap(row, p);
}

extern "C" fn draw_window_divider(window: *mut Lisp_Window, x0: i32, x1: i32, y0: i32, y1: i32) {
    let window: LispWindowRef = window.into();
    let frame: LispFrameRef = window.get_frame();

    let output: OutputRef = unsafe { frame.output_data.wr.into() };

    let face = frame.face_from_id(face_id::WINDOW_DIVIDER_FACE_ID);
    let face_first = frame.face_from_id(face_id::WINDOW_DIVIDER_FIRST_PIXEL_FACE_ID);
    let face_last = frame.face_from_id(face_id::WINDOW_DIVIDER_LAST_PIXEL_FACE_ID);

    let color = match face {
        Some(f) => unsafe { (*f).foreground },
        None => frame.foreground_pixel,
    };

    let color_first = match face_first {
        Some(f) => unsafe { (*f).foreground },
        None => frame.foreground_pixel,
    };

    let color_last = match face_last {
        Some(f) => unsafe { (*f).foreground },
        None => frame.foreground_pixel,
    };

    output
        .canvas()
        .draw_window_divider(color, color_first, color_last, x0, x1, y0, y1);
}

extern "C" fn draw_vertical_window_border(window: *mut Lisp_Window, x: i32, y0: i32, y1: i32) {
    let window: LispWindowRef = window.into();
    let frame: LispFrameRef = window.get_frame();

    let output: OutputRef = unsafe { frame.output_data.wr.into() };

    let face = frame.face_from_id(face_id::VERTICAL_BORDER_FACE_ID);

    output.canvas().draw_vertical_window_border(face, x, y0, y1);
}

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

extern "C" fn new_font(
    frame: *mut Lisp_Frame,
    font_object: LispObject,
    fontset: i32,
) -> LispObject {
    let mut frame: LispFrameRef = frame.into();

    let font = LispFontRef::from_vectorlike(font_object.as_vectorlike().unwrap()).as_font_mut();
    let mut output: OutputRef = unsafe { frame.output_data.wr.into() };

    let fontset = if fontset < 0 {
        unsafe { fontset_from_font(font_object) }
    } else {
        fontset
    };

    output.fontset = fontset;

    if output.font == font.into() {
        return font_object;
    }

    output.font = font.into();

    frame.line_height = unsafe { (*font).height };
    frame.column_width = unsafe { (*font).average_width };

    font_object
}

extern "C" fn defined_color(
    _frame: *mut Lisp_Frame,
    color_name: *const libc::c_char,
    color_def: *mut Emacs_Color,
    _alloc_p: bool,
    _make_indext: bool,
) -> bool {
    let c_color = unsafe { CString::from_raw(color_name as *mut _) };

    let color = c_color
        .to_str()
        .ok()
        .and_then(|color| lookup_color_by_name_or_hex(color));

    // throw back the c pointer
    c_color.into_raw();

    match color {
        Some(c) => {
            color_to_xcolor(c, color_def);
            true
        }
        _ => false,
    }
}

extern "C" fn frame_visible_invisible(frame: *mut Lisp_Frame, is_visible: bool) {
    let mut f: LispFrameRef = frame.into();

    f.set_visible(is_visible as u32);

    let output: OutputRef = unsafe { f.output_data.wr.into() };

    if is_visible {
        output.show_window();
    } else {
        output.hide_window();
    }
}

extern "C" fn set_background_color(f: *mut Lisp_Frame, arg: LispObject, _old_val: LispObject) {
    let mut frame: LispFrameRef = f.into();
    let mut output: OutputRef = unsafe { frame.output_data.wr.into() };

    let color = lookup_color_by_name_or_hex(&format!("{}", arg.as_string().unwrap()))
        .unwrap_or_else(|| ColorF::WHITE);

    let pixel = color_to_pixel(color);

    frame.background_pixel = pixel;
    output.background_color = color;

    unsafe { update_face_from_frame_parameter(frame.as_mut(), Qbackground_color, arg) };

    if frame.is_visible() {
        unsafe { Fredraw_frame(frame.into()) };
    }
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

    // Other hooks are NULL by default.
    terminal.get_string_resource_hook = Some(get_string_resource);
    terminal.set_new_font_hook = Some(new_font);
    terminal.defined_color_hook = Some(defined_color);
    terminal.frame_visible_invisible_hook = Some(frame_visible_invisible);

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

        dpyinfo_ref.resx = 1.0;
        dpyinfo_ref.resy = 1.0;
    }

    // Set the name of the terminal.
    terminal.name = unsafe { xlispstrdup(display_name) };

    dpyinfo_ref
}
