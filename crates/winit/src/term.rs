use super::frame::FrameExtWinit;
use crate::input::InputProcessor;
use crate::winit_set_background_color;
use crate::winit_set_cursor_color;
use emacs_sys::bindings::block_input;
use emacs_sys::bindings::check_int_nonnegative;
use emacs_sys::bindings::gl_clear_under_internal_border;
use emacs_sys::bindings::gl_renderer_free_frame_resources;
use emacs_sys::bindings::gl_renderer_free_terminal_resources;
use emacs_sys::bindings::init_sigio;
use emacs_sys::bindings::interrupt_input;
use emacs_sys::bindings::set_frame_cursor_types;
use emacs_sys::bindings::unblock_input;
use emacs_sys::bindings::wr_after_update_window_line;
use emacs_sys::bindings::wr_clear_frame;
use emacs_sys::bindings::wr_clear_frame_area;
use emacs_sys::bindings::wr_defined_color;
use emacs_sys::bindings::wr_draw_fringe_bitmap;
use emacs_sys::bindings::wr_draw_glyph_string;
use emacs_sys::bindings::wr_draw_vertical_window_border;
use emacs_sys::bindings::wr_draw_window_cursor;
use emacs_sys::bindings::wr_draw_window_divider;
use emacs_sys::bindings::wr_flush_display;
use emacs_sys::bindings::wr_free_pixmap;
use emacs_sys::bindings::wr_new_font;
use emacs_sys::bindings::wr_scroll_run;
use emacs_sys::bindings::wr_update_end;
use emacs_sys::bindings::wr_update_window_begin;
use emacs_sys::bindings::wr_update_window_end;
use emacs_sys::current_winit_data;
use emacs_sys::globals::Qinternal_border_width;
use emacs_sys::terminal::TerminalRef;
use libc::fd_set;
use libc::sigset_t;
use libc::timespec;
use std::ptr;
use std::sync::LazyLock;
use std::time::Duration;
use winit::event::ElementState;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::keyboard::Key;
use winit::keyboard::NamedKey;
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::platform::pump_events::PumpStatus;

use webrender_api::units::*;
use webrender_api::{self};

use crate::event::create_emacs_event;
use emacs_sys::display_info::DisplayInfo;
use emacs_sys::display_info::DisplayInfoRef;

use emacs_sys::bindings::create_terminal;
use emacs_sys::bindings::current_kboard;
use emacs_sys::bindings::frame_parm_handler;
use emacs_sys::bindings::gui_clear_end_of_line;
use emacs_sys::bindings::gui_clear_window_mouse_face;
use emacs_sys::bindings::gui_fix_overlapping_area;
use emacs_sys::bindings::gui_get_glyph_overhangs;
use emacs_sys::bindings::gui_insert_glyphs;
use emacs_sys::bindings::gui_produce_glyphs;
use emacs_sys::bindings::gui_set_alpha;
use emacs_sys::bindings::gui_set_autolower;
use emacs_sys::bindings::gui_set_autoraise;
use emacs_sys::bindings::gui_set_border_width;
use emacs_sys::bindings::gui_set_bottom_divider_width;
use emacs_sys::bindings::gui_set_font;
use emacs_sys::bindings::gui_set_font_backend;
use emacs_sys::bindings::gui_set_fullscreen;
use emacs_sys::bindings::gui_set_horizontal_scroll_bars;
use emacs_sys::bindings::gui_set_left_fringe;
use emacs_sys::bindings::gui_set_line_spacing;
use emacs_sys::bindings::gui_set_no_special_glyphs;
use emacs_sys::bindings::gui_set_right_divider_width;
use emacs_sys::bindings::gui_set_right_fringe;
use emacs_sys::bindings::gui_set_screen_gamma;
use emacs_sys::bindings::gui_set_scroll_bar_height;
use emacs_sys::bindings::gui_set_scroll_bar_width;
use emacs_sys::bindings::gui_set_unsplittable;
use emacs_sys::bindings::gui_set_vertical_scroll_bars;
use emacs_sys::bindings::gui_set_visibility;
use emacs_sys::bindings::gui_write_glyphs;
use emacs_sys::bindings::initial_kboard;
use emacs_sys::bindings::input_event;
use emacs_sys::bindings::kbd_buffer_store_event_hold;
use emacs_sys::bindings::note_mouse_highlight;
use emacs_sys::bindings::output_method;
use emacs_sys::bindings::redisplay_interface;
use emacs_sys::bindings::scroll_bar_part;
use emacs_sys::bindings::terminal;
use emacs_sys::bindings::xlispstrdup;
use emacs_sys::bindings::Emacs_Cursor;
use emacs_sys::bindings::Fcons;
use emacs_sys::bindings::Time;
use emacs_sys::bindings::PT_PER_INCH;
use emacs_sys::frame::all_frames;
use emacs_sys::frame::Frame;
use emacs_sys::frame::FrameRef;
use emacs_sys::globals::Qnil;
use emacs_sys::globals::Qparent_frame;
use emacs_sys::globals::Qwinit;
use emacs_sys::keyboard::allocate_keyboard;
use emacs_sys::lisp::LispObject;

fn get_frame_parm_handlers() -> [frame_parm_handler; 51] {
    // Keep this list in the same order as frame_parms in frame.c.
    // Use None for unsupported frame parameters.
    let handlers: [frame_parm_handler; 51] = [
        Some(gui_set_autoraise),
        Some(gui_set_autolower),
        Some(winit_set_background_color),
        Some(winit_set_border_color),
        Some(winit_set_border_width),
        Some(winit_set_cursor_color),
        Some(winit_set_cursor_type),
        Some(gui_set_font),
        Some(winit_set_foreground_color),
        Some(winit_set_icon_name),
        Some(winit_set_icon_type),
        Some(winit_set_child_frame_border_width),
        Some(winit_set_internal_border_width),
        Some(gui_set_right_divider_width),
        Some(gui_set_bottom_divider_width),
        Some(winit_set_menu_bar_lines),
        Some(winit_set_mouse_color),
        Some(winit_explicitly_set_name),
        Some(gui_set_scroll_bar_width),
        Some(gui_set_scroll_bar_height),
        Some(winit_set_title),
        Some(gui_set_unsplittable),
        Some(gui_set_vertical_scroll_bars),
        Some(gui_set_horizontal_scroll_bars),
        Some(gui_set_visibility),
        Some(winit_set_tab_bar_lines),
        Some(winit_set_tool_bar_lines),
        Some(winit_set_scroll_bar_foreground),
        Some(winit_set_scroll_bar_background),
        Some(gui_set_screen_gamma),
        Some(gui_set_line_spacing),
        Some(gui_set_left_fringe),
        Some(gui_set_right_fringe),
        Some(winit_set_wait_for_wm),
        Some(gui_set_fullscreen),
        Some(gui_set_font_backend),
        Some(gui_set_alpha),
        Some(winit_set_sticky),
        Some(winit_set_tool_bar_position),
        Some(winit_set_inhibit_double_buffering),
        Some(winit_set_undecorated),
        Some(winit_set_parent_frame),
        Some(winit_set_skip_taskbar),
        Some(winit_set_no_focus_on_map),
        Some(winit_set_no_accept_focus),
        Some(winit_set_z_group),
        Some(winit_set_override_redirect),
        Some(gui_set_no_special_glyphs),
        Some(winit_set_alpha_background),
        Some(winit_set_use_frame_synchronization),
        Some(winit_set_shaded),
    ];

    handlers
}

struct RedisplayInterface(pub redisplay_interface);
unsafe impl Sync for RedisplayInterface {}
unsafe impl Send for RedisplayInterface {}

static REDISPLAY_INTERFACE: LazyLock<RedisplayInterface> = LazyLock::new(|| {
    log::trace!("REDISPLAY_INTERFACE is being created...");
    let frame_parm_handlers = Box::new(get_frame_parm_handlers());

    let interface = redisplay_interface {
        frame_parm_handlers: (Box::into_raw(frame_parm_handlers)) as *mut Option<_>,
        produce_glyphs: Some(gui_produce_glyphs),
        write_glyphs: Some(gui_write_glyphs),
        insert_glyphs: Some(gui_insert_glyphs),
        clear_end_of_line: Some(gui_clear_end_of_line),
        scroll_run_hook: Some(wr_scroll_run),
        after_update_window_line_hook: Some(wr_after_update_window_line),
        update_window_begin_hook: Some(wr_update_window_begin),
        update_window_end_hook: Some(wr_update_window_end),
        flush_display: Some(wr_flush_display),
        clear_window_mouse_face: Some(gui_clear_window_mouse_face),
        get_glyph_overhangs: Some(gui_get_glyph_overhangs),
        fix_overlapping_area: Some(gui_fix_overlapping_area),
        draw_fringe_bitmap: Some(wr_draw_fringe_bitmap),
        define_fringe_bitmap: None,
        destroy_fringe_bitmap: None,
        compute_glyph_string_overhangs: None,
        draw_glyph_string: Some(wr_draw_glyph_string),
        define_frame_cursor: Some(winit_define_frame_cursor),
        clear_frame_area: Some(wr_clear_frame_area),
        clear_under_internal_border: None,
        draw_window_cursor: Some(wr_draw_window_cursor),
        draw_vertical_window_border: Some(wr_draw_vertical_window_border),
        draw_window_divider: Some(wr_draw_window_divider),
        shift_glyphs_for_insert: None, /* Never called; see comment in xterm.c.  */
        show_hourglass: None,
        hide_hourglass: None,
        default_font_parameter: None,
    };

    RedisplayInterface(interface)
});
impl RedisplayInterface {
    fn global() -> &'static RedisplayInterface {
        &REDISPLAY_INTERFACE
    }
}

extern "C" fn get_string_resource(
    _rdb: *mut libc::c_void,
    _name: *const libc::c_char,
    _class: *const libc::c_char,
) -> *const libc::c_char {
    ptr::null()
}

extern "C" fn winit_frame_visible_invisible(frame: *mut Frame, is_visible: bool) {
    let mut f: FrameRef = frame.into();

    f.set_visible_(is_visible);
}

extern "C" fn winit_define_frame_cursor(f: *mut Frame, cursor: Emacs_Cursor) {
    let frame: FrameRef = f.into();
    frame.set_cursor_icon(cursor);
}

extern "C" fn winit_read_input_event(terminal: *mut terminal, hold_quit: *mut input_event) -> i32 {
    let terminal: TerminalRef = terminal.into();
    let mut display_info = terminal.display_info();

    let mut count = 0;
    let mut handle_event = |e: &Event<i32>| {
        match e {
            Event::WindowEvent {
                window_id, event, ..
            } => {
                let frame = all_frames().find(|f| {
                    return f
                        .winit_data()
                        .and_then(|d| d.window.as_ref().and_then(|w| Some(w.id() == *window_id)))
                        .unwrap_or(false);
                });

                if frame.is_none() {
                    return;
                }

                let mut frame: FrameRef = frame.unwrap();
                //lisp frame
                let lframe: LispObject = frame.into();

                match event {
                    WindowEvent::RedrawRequested => {}
                    &WindowEvent::ModifiersChanged(modifiers) => {
                        let _ = InputProcessor::handle_modifiers_changed(modifiers.state());
                    }

                    WindowEvent::KeyboardInput { event, .. } => match event.state {
                        ElementState::Pressed => match event.logical_key {
                            Key::Character(_) | Key::Named(NamedKey::Space) => {
                                for (_i, key_code) in
                                    event.logical_key.to_text().unwrap().chars().enumerate()
                                {
                                    if let Some(mut iev) =
                                        InputProcessor::handle_receive_char(key_code, lframe)
                                    {
                                        unsafe { kbd_buffer_store_event_hold(&mut iev, hold_quit) };
                                        count += 1;
                                    }
                                }
                            }
                            _ => {
                                if let Some(mut iev) =
                                    InputProcessor::handle_key_pressed(event.physical_key, lframe)
                                {
                                    unsafe { kbd_buffer_store_event_hold(&mut iev, hold_quit) };
                                    count += 1;
                                }
                            }
                        },
                        ElementState::Released => {
                            InputProcessor::handle_key_released();
                        }
                    },

                    &WindowEvent::MouseInput { state, button, .. } => {
                        if let Some(mut iev) =
                            InputProcessor::handle_mouse_pressed(button, state, lframe)
                        {
                            unsafe { kbd_buffer_store_event_hold(&mut iev, hold_quit) };
                            count += 1;
                        }
                    }

                    &WindowEvent::MouseWheel { delta, phase, .. } => {
                        if let Some(mut iev) =
                            InputProcessor::handle_mouse_wheel_scrolled(delta, phase, lframe)
                        {
                            unsafe { kbd_buffer_store_event_hold(&mut iev, hold_quit) };
                            count += 1;
                        }

                        frame.set_mouse_moved(false);
                    }

                    &WindowEvent::CursorMoved { position, .. } => {
                        unsafe {
                            note_mouse_highlight(
                                frame.as_mut(),
                                position.x as i32,
                                position.y as i32,
                            )
                        };

                        frame.set_cursor_position(position);

                        frame.set_mouse_moved(true);
                    }

                    &WindowEvent::Focused(is_focused) => {
                        let mut top_frame = lframe.as_frame().unwrap();

                        let focus_frame = if !top_frame.focus_frame.eq(Qnil) {
                            top_frame.focus_frame.as_frame().unwrap().as_mut()
                        } else {
                            top_frame.as_mut()
                        };
                        display_info.highlight_frame = if is_focused {
                            focus_frame
                        } else {
                            ptr::null_mut()
                        };

                        let event_type = if is_focused {
                            emacs_sys::bindings::event_kind::FOCUS_IN_EVENT
                        } else {
                            emacs_sys::bindings::event_kind::FOCUS_OUT_EVENT
                        };

                        let mut event = create_emacs_event(event_type, top_frame.into());

                        unsafe { kbd_buffer_store_event_hold(&mut event, hold_quit) };
                        count += 1;
                    }

                    &WindowEvent::Resized(size) => {
                        let scale_factor = frame.scale_factor();
                        let size = DeviceIntSize::new(
                            (size.width as f64 / scale_factor).round() as i32,
                            (size.height as f64 / scale_factor).round() as i32,
                        );
                        frame.handle_size_change(size, scale_factor);
                    }

                    &WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        inner_size_writer: _,
                    } => {
                        frame.handle_scale_factor_change(scale_factor);
                    }

                    WindowEvent::CloseRequested => {
                        let mut event = create_emacs_event(
                            emacs_sys::bindings::event_kind::DELETE_WINDOW_EVENT,
                            lframe,
                        );

                        unsafe { kbd_buffer_store_event_hold(&mut event, hold_quit) };
                        count += 1;
                    }

                    _ => {}
                }
            }
            _ => {}
        }
    };

    let _ = current_winit_data().and_then(|mut d| {
        for e in &d.pending_events {
            handle_event(e);
        }
        d.pending_events.clear();
        Some(())
    });

    count
}

fn winit_clear_under_internal_border(f: *mut Frame) {
    if FrameRef::new(f)
        .winit_data()
        .and_then(|d| d.window.as_ref().map(|_| true))
        .is_some()
    {
        unsafe { gl_clear_under_internal_border(f) };
    }
}

extern "C" fn winit_set_internal_border_width(f: *mut Frame, arg: LispObject, _oldval: LispObject) {
    let border = unsafe { check_int_nonnegative(arg) };
    let mut f = FrameRef::new(f);
    if border != f.internal_border_width() {
        f.set_internal_border_width(border);
    }

    if f.winit_data()
        .and_then(|d| d.window.as_ref().map(|_| true))
        .is_some()
    {
        f.adjust_size(-1, -1, 3, false, Qinternal_border_width);
        winit_clear_under_internal_border(f.as_mut());
    }
}

// Set the border-color of frame F to value described by ARG.
// ARG can be a string naming a color.
// The border-color is used for the border that is drawn by the display server
// This should be working when winit is using X11, however winit haven't expose
// a set method for change window bordor color as the time when writing
// check x_set_border_color if we want to support this
// For wayland/ns/windows, unknown
extern "C" fn winit_set_border_color(_f: *mut Frame, _arg: LispObject, _oldval: LispObject) {}

// See comments from winit_set_border_color
extern "C" fn winit_set_border_width(f: *mut Frame, arg: LispObject, oldval: LispObject) {
    unsafe { gui_set_border_width(f, arg, oldval) };
}

extern "C" fn winit_set_cursor_type(f: *mut Frame, arg: LispObject, _oldval: LispObject) {
    unsafe { set_frame_cursor_types(f, arg) };
}

extern "C" fn winit_set_foreground_color(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_foreground_color {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_icon_name(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_icon_name {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_icon_type(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_icon_type {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_child_frame_border_width(
    _f: *mut Frame,
    arg: LispObject,
    oldval: LispObject,
) {
    log::debug!(
        "TODO: winit_set_child_frame_border_width {:?} {:?}",
        arg,
        oldval
    );
}

extern "C" fn winit_set_mouse_color(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_mouse_color {:?} {:?}", arg, oldval);
}
extern "C" fn winit_explicitly_set_name(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_explicitly_set_name {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_title(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_title {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_tab_bar_lines(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_tab_bar_lines {:?} {:?}", arg, oldval);
}
extern "C" fn winit_set_tool_bar_lines(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!(
        "TODO: winit_set_internal_border_width {:?} {:?}",
        arg,
        oldval
    );
}
extern "C" fn winit_set_scroll_bar_foreground(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!(
        "TODO: winit_set_scroll_bar_foreground {:?} {:?}",
        arg,
        oldval
    );
}
extern "C" fn winit_set_scroll_bar_background(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!(
        "TODO: winit_set_scroll_bar_background {:?} {:?}",
        arg,
        oldval
    );
}

extern "C" fn winit_set_use_frame_synchronization(
    _f: *mut Frame,
    arg: LispObject,
    oldval: LispObject,
) {
    log::debug!(
        "TODO: winit_set_use_frame_synchronization {:?} {:?}",
        arg,
        oldval
    );
}

extern "C" fn winit_set_alpha_background(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_alpha_background {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_wait_for_wm(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_wait_for_wm {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_shaded(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_shaded {:?} {:?}", arg, oldval);
}
extern "C" fn winit_set_skip_taskbar(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_skip_taskbar {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_no_focus_on_map(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_no_focus_on_map {:?} {:?}", arg, oldval);
}
extern "C" fn winit_set_no_accept_focus(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_no_accept_focus {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_z_group(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_z_group {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_override_redirect(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_override_redirect {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_sticky(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_sticky {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_tool_bar_position(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_tool_bar_position {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_inhibit_double_buffering(
    _f: *mut Frame,
    arg: LispObject,
    oldval: LispObject,
) {
    log::debug!(
        "TODO: winit_set_inhibit_double_buffering {:?} {:?}",
        arg,
        oldval
    );
}

extern "C" fn winit_set_undecorated(_f: *mut Frame, arg: LispObject, oldval: LispObject) {
    log::debug!("TODO: winit_set_undecorated {:?} {:?}", arg, oldval);
}

extern "C" fn winit_set_fullscreen(f: *mut Frame) {
    let frame: FrameRef = f.into();
    frame.set_fullscreen();
}

extern "C" fn winit_menu_show(
    _f: *mut Frame,
    _x: ::libc::c_int,
    _y: ::libc::c_int,
    _menuflags: ::libc::c_int,
    _title: LispObject,
    _error_name: *mut *const ::libc::c_char,
) -> LispObject {
    message!("Menu functionalities currently is not available for Winit");
    Qnil
}

// This function should be called by Emacs redisplay code to set the
// name; names set this way will never override names set by the user's
// lisp code.
extern "C" fn winit_implicitly_set_name(frame: *mut Frame, arg: LispObject, old_val: LispObject) {
    let mut frame: FrameRef = frame.into();

    frame.implicitly_set_name(arg, old_val);
}

extern "C" fn winit_get_focus_frame(frame: *mut Frame) -> LispObject {
    match FrameRef::from(frame)
        .terminal()
        .winit_data()
        .and_then(|d| Some(d.focus_frame))
    {
        Some(frame) if !frame.is_null() => frame.into(),
        _ => Qnil,
    }
}

// This tries to wait until the frame is really visible, depending on
// the value of Vx_wait_for_event_timeout.
// However, if the window manager asks the user where to position
// the frame, this will return before the user finishes doing that.
// The frame will not actually be visible at that time,
// but it will become visible later when the window manager
// finishes with it.
extern "C" fn winit_make_frame_visible_invisible(f: *mut Frame, visible: bool) {
    let mut frame: FrameRef = f.into();

    frame.set_visible_(visible);
}

extern "C" fn winit_iconify_frame(f: *mut Frame) {
    let mut frame: FrameRef = f.into();
    frame.iconify();
}

extern "C" fn winit_mouse_position(
    fp: *mut *mut Frame,
    _insist: i32,
    bar_window: *mut LispObject,
    part: *mut scroll_bar_part::Type,
    x: *mut LispObject,
    y: *mut LispObject,
    _timestamp: *mut Time,
) {
    let (dpyinfo, cursor_pos) = {
        let frame: FrameRef = unsafe { (*fp).into() };

        (frame.display_info(), frame.cursor_position())
    };

    // Clear the mouse-moved flag for every frame on this display.
    for mut frame in all_frames() {
        if frame.display_info() == dpyinfo {
            frame.set_mouse_moved(false);
        }
    }

    unsafe { *bar_window = Qnil };
    unsafe { *part = 0 };

    unsafe { *x = cursor_pos.x.into() };
    unsafe { *y = cursor_pos.y.into() };
}

// cleanup frame resource after frame is deleted
extern "C" fn winit_destroy_frame(f: *mut Frame) {
    unsafe { gl_renderer_free_frame_resources(f) };
    let f: FrameRef = f.into();
    f.free_winit_data();
}

extern "C" fn winit_set_menu_bar_lines(f: *mut Frame, _value: LispObject, _old_value: LispObject) {
    let frame = FrameRef::from(f);
    /* Right now, menu bars don't work properly in minibuf-only frames;
    most of the commands try to apply themselves to the minibuffer
    frame itself, and get an error because you can't switch buffers
    in or split the minibuffer window.  */
    if frame.is_minibuf_only() || frame.parent_frame().is_some() {
        return;
    }

    //TODO unimplemented set_menu_bar_lines
    return;
}

// Set frame F's `parent-frame' parameter.  If non-nil, make F a child
// frame of the frame specified by that parameter.  Technically, this
// makes F's window-system window a child window of the parent frame's
// window-system window.  If nil, make F's window-system window a
// top-level window--a child of its display's root window.
extern "C" fn winit_set_parent_frame(f: *mut Frame, value: LispObject, old_value: LispObject) {
    if value.is_not_nil()
        && (!value.is_frame()
            || !FrameRef::from(value).is_live()
            || !FrameRef::from(value).is_current_window_system())
    {
        FrameRef::from(f).store_param(Qparent_frame, old_value);
        error!("Invalid specification of `parent-frame'");
    }

    let p = FrameRef::from(value);
    let f = FrameRef::from(f);

    if p != f {
        unsafe { block_input() };
        if !p.is_null() {
            if f.display_info() != p.display_info() {
                error!("Cross display reparent.");
            }
        }

        if p.is_null() {
            //
        } else {
        }

        unsafe { unblock_input() };
        f.set_parent(value);
    }
}

#[no_mangle]
pub extern "C" fn set_frame_menubar(_f: *mut Frame, _deep_p: bool) {
    todo!()
}

fn winit_create_terminal(mut dpyinfo: DisplayInfoRef) -> TerminalRef {
    let redisplay_interface = RedisplayInterface::global();
    let terminal_ptr = unsafe {
        create_terminal(
            output_method::output_winit,
            &redisplay_interface.0 as *const _ as *mut _,
        )
    };

    let mut terminal = TerminalRef::new(terminal_ptr);

    // Link terminal and dpyinfo together
    terminal.display_info.winit = dpyinfo.as_mut();
    dpyinfo.terminal = terminal.as_mut();

    // Terminal hooks
    // Other hooks are NULL by default.
    terminal.get_string_resource_hook = Some(get_string_resource);
    terminal.set_new_font_hook = Some(wr_new_font);
    terminal.defined_color_hook = Some(wr_defined_color);
    terminal.frame_visible_invisible_hook = Some(winit_frame_visible_invisible);
    terminal.clear_frame_hook = Some(wr_clear_frame);
    terminal.read_socket_hook = Some(winit_read_input_event);
    terminal.fullscreen_hook = Some(winit_set_fullscreen);
    terminal.menu_show_hook = Some(winit_menu_show);
    terminal.implicit_set_name_hook = Some(winit_implicitly_set_name);
    terminal.get_focus_frame = Some(winit_get_focus_frame);
    terminal.frame_visible_invisible_hook = Some(winit_make_frame_visible_invisible);
    terminal.iconify_frame_hook = Some(winit_iconify_frame);
    terminal.mouse_position_hook = Some(winit_mouse_position);
    terminal.update_end_hook = Some(wr_update_end);
    terminal.free_pixmap = Some(wr_free_pixmap);
    terminal.delete_frame_hook = Some(winit_destroy_frame);
    terminal.delete_terminal_hook = Some(winit_delete_terminal);

    // Init term data for winit
    terminal.init_winit_data();

    terminal
}

extern "C" fn winit_delete_terminal(terminal: *mut terminal) {
    unsafe { gl_renderer_free_terminal_resources(terminal) };
    let mut terminal: TerminalRef = terminal.into();
    terminal.free_winit_data();
}

fn winit_pump_events(timeout: Option<Duration>) -> Vec<Event<i32>> {
    current_winit_data().map_or(Vec::new(), |mut d| {
        let mut pending_events: Vec<Event<i32>> = Vec::new();
        let mut add_event = |e: Event<i32>| {
            pending_events.push(e);
        };
        let status = d.event_loop.pump_events(timeout, |e, _elwt| {
            if let Event::WindowEvent { event, .. } = &e {
                // Print only Window events to reduce noise
                log::trace!("{event:?}");
            }

            match e {
                Event::AboutToWait => {
                    all_frames().for_each(|f| {
                        let _ = f
                            .winit_data()
                            .map(|d| d.window.as_ref().map(|w| w.request_redraw()));
                    });
                    spin_sleep::sleep(Duration::from_millis(8));
                }
                Event::WindowEvent {
                    event, window_id, ..
                } => match event {
                    WindowEvent::Resized(_)
                    | WindowEvent::KeyboardInput { .. }
                    | WindowEvent::ModifiersChanged(_)
                    | WindowEvent::MouseInput { .. }
                    | WindowEvent::CursorMoved { .. }
                    | WindowEvent::ThemeChanged(_)
                    | WindowEvent::Focused(_)
                    | WindowEvent::MouseWheel { .. }
                    | WindowEvent::RedrawRequested
                    | WindowEvent::CloseRequested => {
                        add_event(Event::WindowEvent { window_id, event });
                    }
                    _ => {}
                },
                Event::UserEvent(_nfds) => {}
                _ => {}
            }
        });
        if let Some(PumpStatus::Exit(_exit_code)) = Some(status) {
            // break 'main ExitCode::from(exit_code as u8);
        }
        pending_events
    })
}

#[no_mangle]
pub extern "C" fn winit_select(
    nfds: i32,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    _exceptfds: *mut fd_set,
    timeout: *mut timespec,
    _sigmask: *mut sigset_t,
) -> i32 {
    use nix::sys::signal::Signal;
    use nix::sys::signal::{self};
    use nix::sys::time::TimeSpec;
    use std::time::Instant;

    let duration = unsafe { Duration::new((*timeout).tv_sec as u64, (*timeout).tv_nsec as u32) };
    // Code from C sometimes set durations to 10000s. No idea how to reason about that.
    // Manually reduce it here.
    let duration = std::cmp::min(duration, Duration::from_millis(25));
    let deadline = Instant::now() + duration;

    let mut pending_events = winit_pump_events(Some(duration));
    if pending_events.len() > 0 {
        let _ = current_winit_data().and_then(|mut d| {
            d.pending_events.append(&mut pending_events);
            Some(())
        });

        // notify emacs's code that a keyboard event arrived.
        match signal::raise(Signal::SIGIO) {
            Ok(_) => {}
            Err(err) => log::error!("sigio err: {err:?}"),
        };
    }

    let now = Instant::now();
    let mut timespec = TimeSpec::from_duration(deadline - now);

    return unsafe {
        emacs_sys::bindings::thread_select(
            Some(libc::pselect),
            nfds,
            readfds,
            writefds,
            _exceptfds,
            timespec.as_mut(),
            _sigmask,
        )
    };
}

pub fn winit_term_init(display_name: LispObject) -> DisplayInfoRef {
    log::info!("Winit term init");

    let dpyinfo = Box::new(DisplayInfo::default());
    let mut dpyinfo_ref = DisplayInfoRef::new(Box::into_raw(dpyinfo));
    let mut terminal = winit_create_terminal(dpyinfo_ref);

    // baud_rate is the value computed from fileno (tty->input)
    // Hardcode the value for now
    unsafe { emacs_sys::bindings::init_baud_rate(38400) };
    // Fset_input_interrupt_mode (Qnil);

    let fd = emacs_sys::display_descriptor(terminal.raw_display_handle().unwrap());

    unsafe {
        if interrupt_input {
            init_sigio(fd);
        }
    };

    let mut kboard = allocate_keyboard(Qwinit);

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
        dpyinfo_ref.name_list_element = unsafe { Fcons(display_name, Qnil) };

        // https://lists.gnu.org/archive/html/emacs-devel/2015-11/msg00194.html
        dpyinfo_ref.smallest_font_height = 1;
        dpyinfo_ref.smallest_char_width = 1;

        // we have https://docs.rs/winit/0.23.0/winit/dpi/index.html
        // set to base DPI PT_PER_INCH to equal out POINT_TO_PIXEL/PIXEL_TO_POINT
        dpyinfo_ref.resx = PT_PER_INCH;
        dpyinfo_ref.resy = PT_PER_INCH;
    }

    // Set the name of the terminal.
    terminal.name = unsafe { xlispstrdup(display_name) };

    dpyinfo_ref
}
