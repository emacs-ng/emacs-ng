use std::ptr;

use lazy_static::lazy_static;

use super::display_info::{DisplayInfo, DisplayInfoRef};

use crate::{
    lisp::{ExternalPtr, LispObject},
    remacs_sys::{
        allocate_kboard, create_terminal, current_kboard, frame_parm_handler, gui_set_font,
        gui_set_font_backend, initial_kboard, output_method, redisplay_interface, terminal,
        xlispstrdup, Fcons, Qnil, Qwr, KBOARD,
    },
};

pub type TerminalRef = ExternalPtr<terminal>;

impl Default for TerminalRef {
    fn default() -> Self {
        Self::new(ptr::null_mut())
    }
}

pub type KboardRef = ExternalPtr<KBOARD>;

impl KboardRef {
    pub fn add_ref(&mut self) {
        (*self).reference_count = (*self).reference_count + 1;
    }
}

type RedisplayInterfaceRef = ExternalPtr<redisplay_interface>;
unsafe impl Sync for RedisplayInterfaceRef {}

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

lazy_static! {
    static ref REDISPLAY_INTERFACE: RedisplayInterfaceRef = {
        let frame_parm_handlers = Box::new(get_frame_parm_handlers());

        let interface = Box::new(redisplay_interface {
            frame_parm_handlers: (Box::into_raw(frame_parm_handlers)) as *mut Option<_>,
            produce_glyphs: None,
            write_glyphs: None,
            insert_glyphs: None,
            clear_end_of_line: None,
            clear_under_internal_border: None,
            scroll_run_hook: None,
            after_update_window_line_hook: None,
            update_window_begin_hook: None,
            update_window_end_hook: None,
            flush_display: None,
            clear_window_mouse_face: None,
            get_glyph_overhangs: None,
            fix_overlapping_area: None,
            draw_fringe_bitmap: None,
            define_fringe_bitmap: None,
            destroy_fringe_bitmap: None,
            compute_glyph_string_overhangs: None,
            draw_glyph_string: None,
            define_frame_cursor: None,
            default_font_parameter: None,
            clear_frame_area: None,
            draw_window_cursor: None,
            draw_vertical_window_border: None,
            draw_window_divider: None,
            shift_glyphs_for_insert: None,
            show_hourglass: None,
            hide_hourglass: None,
        });

        RedisplayInterfaceRef::new(Box::into_raw(interface))
    };
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
            REDISPLAY_INTERFACE.clone().as_mut(),
        )
    };
    let mut terminal = TerminalRef::new(terminal_ptr);

    // Link terminal and dpyinfo together
    terminal.display_info.wr = dpyinfo.as_mut();
    dpyinfo.get_inner().terminal = terminal;
    dpyinfo.terminal = terminal.as_mut();

    //TODO: add terminal hook
    // Other hooks are NULL by default.
    terminal.get_string_resource_hook = Some(get_string_resource);

    terminal
}

pub fn wr_term_init(display_name: LispObject) -> DisplayInfoRef {
    let dpyinfo = Box::new(DisplayInfo::new());
    let mut dpyinfo_ref = DisplayInfoRef::new(Box::into_raw(dpyinfo));

    let mut terminal = wr_create_terminal(dpyinfo_ref);

    let mut kboard = KboardRef::new(unsafe { allocate_kboard(Qwr) });
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

    dpyinfo_ref.name_list_element = unsafe { Fcons(display_name, Qnil) };

    // https://lists.gnu.org/archive/html/emacs-devel/2015-11/msg00194.html
    dpyinfo_ref.smallest_font_height = 1;
    dpyinfo_ref.smallest_char_width = 1;

    // Set the name of the terminal.
    terminal.name = unsafe { xlispstrdup(display_name) };

    dpyinfo_ref
}
