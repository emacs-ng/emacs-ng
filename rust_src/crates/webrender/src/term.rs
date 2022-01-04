use std::ptr;
use std::{cmp::max, ffi::CString};

use emacs::multibyte::LispStringRef;
use glutin::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyboardInput, WindowEvent},
};
use lazy_static::lazy_static;

use webrender::api::units::LayoutPoint;
use webrender::api::{units::LayoutRect, *};

use crate::event_loop::EVENT_BUFFER;
use crate::frame::LispFrameExt;
use crate::fringe::get_or_create_fringe_bitmap;
use crate::{
    color::{color_to_pixel, color_to_xcolor, lookup_color_by_name_or_hex, pixel_to_color},
    cursor::{draw_bar_cursor, draw_filled_cursor, draw_hollow_box_cursor},
    display_info::{DisplayInfo, DisplayInfoRef},
    event::create_emacs_event,
    image::WrPixmap,
    output::OutputRef,
    util::HandyDandyRectBuilder,
};

use emacs::{
    bindings::{
        block_input, display_and_set_cursor, do_pending_window_change, draw_window_fringes,
        face_id, glyph_row_area, gui_clear_cursor, gui_clear_end_of_line,
        gui_clear_window_mouse_face, gui_draw_right_divider, gui_draw_vertical_border,
        gui_fix_overlapping_area, gui_get_glyph_overhangs, gui_produce_glyphs, gui_set_alpha,
        gui_set_autolower, gui_set_autoraise, gui_set_border_width, gui_set_bottom_divider_width,
        gui_set_font, gui_set_font_backend, gui_set_fullscreen, gui_set_horizontal_scroll_bars,
        gui_set_left_fringe, gui_set_line_spacing, gui_set_no_special_glyphs,
        gui_set_right_divider_width, gui_set_right_fringe, gui_set_screen_gamma,
        gui_set_scroll_bar_height, gui_set_scroll_bar_width, gui_set_unsplittable,
        gui_set_vertical_scroll_bars, gui_set_visibility, gui_update_cursor, gui_write_glyphs,
        input_event, kbd_buffer_store_event_hold, run, unblock_input, Time,
    },
    bindings::{
        create_terminal, current_kboard, draw_fringe_bitmap_params, fontset_from_font,
        frame_parm_handler, fullscreen_type, glyph_row, glyph_string, initial_kboard,
        note_mouse_highlight, output_method, redisplay_interface, scroll_bar_part, terminal,
        text_cursor_kinds, xlispstrdup, Emacs_Color, Emacs_Cursor, Emacs_Pixmap, Fcons,
        Fredraw_frame,
    },
    font::LispFontRef,
    frame::{all_frames, LispFrameRef, Lisp_Frame},
    globals::{Qbackground_color, Qfullscreen, Qmaximized, Qnil, Qx},
    glyph::GlyphStringRef,
    keyboard::allocate_keyboard,
    lisp::{ExternalPtr, LispObject},
    window::{LispWindowRef, Lisp_Window},
};

pub type TerminalRef = ExternalPtr<terminal>;

fn get_frame_parm_handlers() -> [frame_parm_handler; 48] {
    // Keep this list in the same order as frame_parms in frame.c.
    // Use None for unsupported frame parameters.
    let handlers: [frame_parm_handler; 48] = [
        Some(gui_set_autoraise),
        Some(gui_set_autolower),
        Some(set_background_color),
        None,
        Some(gui_set_border_width),
        Some(set_cursor_color),
        None,
        Some(gui_set_font),
        None,
        None,
        None,
        None,
        None,
        Some(gui_set_right_divider_width),
        Some(gui_set_bottom_divider_width),
        None,
        None,
        None,
        Some(gui_set_scroll_bar_width),
        Some(gui_set_scroll_bar_height),
        None,
        Some(gui_set_unsplittable),
        Some(gui_set_vertical_scroll_bars),
        Some(gui_set_horizontal_scroll_bars),
        Some(gui_set_visibility),
        None,
        None,
        None,
        None,
        Some(gui_set_screen_gamma),
        Some(gui_set_line_spacing),
        Some(gui_set_left_fringe),
        Some(gui_set_right_fringe),
        None,
        Some(gui_set_fullscreen),
        Some(gui_set_font_backend),
        Some(gui_set_alpha),
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
        Some(gui_set_no_special_glyphs),
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
            scroll_run_hook: Some(scroll_run),
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
            define_frame_cursor: Some(define_frame_cursor),
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
    cursor_no_p: bool,
    _mouse_face_overwritten_p: bool,
) {
    let mut window: LispWindowRef = window.into();

    if window.pseudo_window_p() {
        return;
    }

    unsafe { block_input() };
    if cursor_no_p {
        unsafe {
            display_and_set_cursor(
                window.as_mut(),
                true,
                window.output_cursor.hpos,
                window.output_cursor.vpos,
                window.output_cursor.x,
                window.output_cursor.y,
            )
        };
    }

    if unsafe { draw_window_fringes(window.as_mut(), true) } {
        if window.right_divider_width() > 0 {
            unsafe { gui_draw_right_divider(window.as_mut()) }
        } else {
            unsafe { gui_draw_vertical_border(window.as_mut()) }
        }
    }

    unsafe { unblock_input() };

    let frame: LispFrameRef = window.get_frame();
    frame.wr_output().flush();
}

extern "C" fn flush_display(f: *mut Lisp_Frame) {
    let frame: LispFrameRef = f.into();

    frame.wr_output().flush();
}

#[allow(unused_variables)]
extern "C" fn after_update_window_line(w: *mut Lisp_Window, desired_row: *mut glyph_row) {
    let window: LispWindowRef = w.into();

    if !unsafe { (*desired_row).mode_line_p() } && !window.pseudo_window_p() {
        unsafe { (*desired_row).set_redraw_fringe_bitmaps_p(true) };
    }
}

#[allow(unused_variables)]
extern "C" fn draw_glyph_string(s: *mut glyph_string) {
    let s: GlyphStringRef = s.into();

    let output: OutputRef = {
        let frame: LispFrameRef = s.f.into();
        frame.wr_output()
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

    let output = frame.wr_output();

    let row_rect: LayoutRect = unsafe {
        let (window_x, window_y, window_width, _) = window.area_box(glyph_row_area::ANY_AREA);

        let x = window_x;

        let row_y = window.frame_pixel_y(max(0, (*row).y));
        let y = max(row_y, window_y);

        let width = window_width;
        let height = (*row).visible_height;

        (x, y).by(width, height)
    };

    let which = unsafe { (*p).which };

    let pos_x = unsafe { (*p).x };
    let pos_y = unsafe { (*p).y };

    let pos = LayoutPoint::new(pos_x as f32, pos_y as f32);

    let image_clip_rect: LayoutRect = {
        let width = unsafe { (*p).wd };
        let height = unsafe { (*p).h };

        if which > 0 {
            (pos_x, pos_y).by(width, height)
        } else {
            LayoutRect::zero()
        }
    };

    let clear_rect = if unsafe { (*p).bx >= 0 && !(*p).overlay_p() } {
        unsafe { ((*p).bx, (*p).by).by((*p).nx, (*p).ny) }
    } else {
        LayoutRect::zero()
    };

    let image = get_or_create_fringe_bitmap(output, which, p);

    let face = unsafe { (*p).face };

    let background_color = pixel_to_color(unsafe { (*face).background });

    let bitmap_color = if unsafe { (*p).cursor_p() } {
        output.cursor_color
    } else if unsafe { (*p).overlay_p() } {
        background_color
    } else {
        pixel_to_color(unsafe { (*face).foreground })
    };

    output.canvas().draw_fringe_bitmap(
        pos,
        image,
        bitmap_color,
        background_color,
        image_clip_rect,
        clear_rect,
        row_rect,
    );
}

extern "C" fn set_cursor_color(f: *mut Lisp_Frame, arg: LispObject, _old_val: LispObject) {
    let frame: LispFrameRef = f.into();

    let color_str: LispStringRef = arg.as_symbol_or_string().into();
    let color_str = format!("{}", color_str.to_string());
    let color = lookup_color_by_name_or_hex(&color_str);

    if let Some(color) = color {
        frame.wr_output().cursor_color = color;
    }

    if frame.is_visible() {
        unsafe { gui_update_cursor(f, false) };
        unsafe { gui_update_cursor(f, true) };
    }
}

extern "C" fn draw_window_divider(window: *mut Lisp_Window, x0: i32, x1: i32, y0: i32, y1: i32) {
    let window: LispWindowRef = window.into();
    let frame: LispFrameRef = window.get_frame();

    let output = frame.wr_output();

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

    let output = frame.wr_output();

    let face = frame.face_from_id(face_id::VERTICAL_BORDER_FACE_ID);

    output.canvas().draw_vertical_window_border(face, x, y0, y1);
}

#[allow(unused_variables)]
extern "C" fn clear_frame_area(f: *mut Lisp_Frame, x: i32, y: i32, width: i32, height: i32) {
    let frame: LispFrameRef = f.into();
    let output = frame.wr_output();

    let color = pixel_to_color(frame.background_pixel);

    output.canvas().clear_area(color, x, y, width, height);
}

extern "C" fn draw_window_cursor(
    window: *mut Lisp_Window,
    row: *mut glyph_row,
    _x: i32,
    _y: i32,
    cursor_type: text_cursor_kinds::Type,
    cursor_width: i32,
    on_p: bool,
    _active_p: bool,
) {
    let mut window: LispWindowRef = window.into();

    if !on_p {
        return;
    }

    window.phys_cursor_type = cursor_type;
    window.set_phys_cursor_on_p(true);

    match cursor_type {
        text_cursor_kinds::FILLED_BOX_CURSOR => {
            draw_filled_cursor(window, row);
        }

        text_cursor_kinds::HOLLOW_BOX_CURSOR => {
            draw_hollow_box_cursor(window, row);
        }

        text_cursor_kinds::BAR_CURSOR => {
            draw_bar_cursor(window, row, cursor_width, false);
        }
        text_cursor_kinds::HBAR_CURSOR => {
            draw_bar_cursor(window, row, cursor_width, true);
        }

        text_cursor_kinds::NO_CURSOR => {
            window.phys_cursor_width = 0;
        }
        _ => panic!("invalid cursor type"),
    }
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
    let mut output = frame.wr_output();

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
    let _ = c_color.into_raw();

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

    let output = f.wr_output();

    if is_visible {
        output.show_window();
    } else {
        output.hide_window();
    }
}

extern "C" fn set_background_color(f: *mut Lisp_Frame, arg: LispObject, _old_val: LispObject) {
    let mut frame: LispFrameRef = f.into();
    let mut output = frame.wr_output();

    let color = lookup_color_by_name_or_hex(&format!("{}", arg.as_string().unwrap()))
        .unwrap_or_else(|| ColorF::WHITE);

    let pixel = color_to_pixel(color);

    frame.background_pixel = pixel;
    output.background_color = color;

    frame.update_face_from_frame_param(Qbackground_color, arg);

    if frame.is_visible() {
        unsafe { Fredraw_frame(frame.into()) };
    }
}

extern "C" fn clear_frame(f: *mut Lisp_Frame) {
    let frame: LispFrameRef = f.into();
    let mut output = frame.wr_output();

    output.clear_display_list_builder();

    let width = frame.pixel_width;
    let height = frame.pixel_height;

    clear_frame_area(f, 0, 0, width, height);
}

extern "C" fn scroll_run(w: *mut Lisp_Window, run: *mut run) {
    let window: LispWindowRef = w.into();
    let frame = window.get_frame();
    let output = frame.wr_output();

    let (x, y, width, height) = window.area_box(glyph_row_area::ANY_AREA);

    let from_y = unsafe { (*run).current_y + window.top_edge_y() };
    let to_y = unsafe { (*run).desired_y + window.top_edge_y() };

    let scroll_height = unsafe { (*run).height };

    // Cursor off.  Will be switched on again in gui_update_window_end.
    unsafe { gui_clear_cursor(w) };

    output
        .canvas()
        .scroll(x, y, width, height, from_y, to_y, scroll_height);
}

extern "C" fn define_frame_cursor(f: *mut Lisp_Frame, cursor: Emacs_Cursor) {
    let frame: LispFrameRef = f.into();
    let output = frame.wr_output();

    output.set_mouse_cursor(cursor);
}

extern "C" fn read_input_event(terminal: *mut terminal, hold_quit: *mut input_event) -> i32 {
    let terminal: TerminalRef = terminal.into();
    let dpyinfo = DisplayInfoRef::new(unsafe { terminal.display_info.wr } as *mut _);

    let mut dpyinfo = dpyinfo.get_inner();

    let mut count = 0;

    let mut events = EVENT_BUFFER.lock().unwrap();

    for e in events.iter() {
        let e = e.clone();

        match e {
            Event::WindowEvent { window_id, event } => {
                let output = dpyinfo.outputs.get_mut(&window_id);

                if output.is_none() {
                    continue;
                }

                let output = output.unwrap();

                let frame: LispObject = output.get_frame().into();

                match event {
                    WindowEvent::ReceivedCharacter(key_code) => {
                        if let Some(mut iev) = dpyinfo.input_processor.receive_char(key_code, frame)
                        {
                            unsafe { kbd_buffer_store_event_hold(&mut iev, hold_quit) };
                            count += 1;
                        }
                    }

                    WindowEvent::ModifiersChanged(state) => {
                        dpyinfo.input_processor.change_modifiers(state);
                    }

                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(key_code),
                                ..
                            },
                        ..
                    } => match state {
                        ElementState::Pressed => {
                            if let Some(mut iev) =
                                dpyinfo.input_processor.key_pressed(key_code, frame)
                            {
                                unsafe { kbd_buffer_store_event_hold(&mut iev, hold_quit) };
                                count += 1;
                            }
                        }
                        ElementState::Released => dpyinfo.input_processor.key_released(),
                    },

                    WindowEvent::MouseInput { state, button, .. } => {
                        if let Some(mut iev) =
                            dpyinfo.input_processor.mouse_pressed(button, state, frame)
                        {
                            unsafe { kbd_buffer_store_event_hold(&mut iev, hold_quit) };
                            count += 1;
                        }
                    }

                    WindowEvent::MouseWheel { delta, phase, .. } => {
                        if let Some(mut iev) = dpyinfo
                            .input_processor
                            .mouse_wheel_scrolled(delta, phase, frame)
                        {
                            unsafe { kbd_buffer_store_event_hold(&mut iev, hold_quit) };
                            count += 1;
                        }

                        let mut frame: LispFrameRef = frame.into();
                        frame.set_mouse_moved(false);
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        let mut frame: LispFrameRef = frame.into();

                        unsafe {
                            note_mouse_highlight(
                                frame.as_mut(),
                                position.x as i32,
                                position.y as i32,
                            )
                        };

                        dpyinfo.input_processor.cursor_move(position);

                        frame.set_mouse_moved(true);
                    }

                    WindowEvent::Focused(is_focused) => {
                        let mut dpyinfo =
                            DisplayInfoRef::new(unsafe { terminal.display_info.wr } as *mut _);

                        let mut top_frame = frame.as_frame().unwrap();

                        let focus_frame = if !top_frame.focus_frame.eq(Qnil) {
                            top_frame.focus_frame.as_frame().unwrap().as_mut()
                        } else {
                            top_frame.as_mut()
                        };

                        dpyinfo.get_raw().highlight_frame = if is_focused {
                            focus_frame
                        } else {
                            ptr::null_mut()
                        };

                        let event_type = if is_focused {
                            emacs::bindings::event_kind::FOCUS_IN_EVENT
                        } else {
                            emacs::bindings::event_kind::FOCUS_OUT_EVENT
                        };

                        let mut event = create_emacs_event(event_type, top_frame.into());

                        unsafe { kbd_buffer_store_event_hold(&mut event, hold_quit) };
                        count += 1;
                    }

                    WindowEvent::Resized(size) => {
                        output.resize(&size);

                        let frame: LispFrameRef = frame.into();
                        frame.change_size(
                            size.width as i32,
                            size.height as i32 - frame.menu_bar_height,
                            false,
                            true,
                            false,
                        );

                        unsafe { do_pending_window_change(false) };
                    }

                    WindowEvent::CloseRequested => {
                        let mut event = create_emacs_event(
                            emacs::bindings::event_kind::DELETE_WINDOW_EVENT,
                            frame,
                        );

                        unsafe { kbd_buffer_store_event_hold(&mut event, hold_quit) };
                        count += 1;
                    }

                    _ => {}
                }
            }
            _ => {}
        };
    }

    events.clear();

    count
}

extern "C" fn fullscreen(f: *mut Lisp_Frame) {
    let frame: LispFrameRef = f.into();

    if !frame.is_visible() {
        return;
    }

    let output = frame.wr_output();

    if frame.want_fullscreen() == fullscreen_type::FULLSCREEN_MAXIMIZED {
        output.maximize();

        frame.store_param(Qfullscreen, Qmaximized);
    }
}

// This function should be called by Emacs redisplay code to set the
// name; names set this way will never override names set by the user's
// lisp code.
extern "C" fn implicitly_set_name(frame: *mut Lisp_Frame, arg: LispObject, _oldval: LispObject) {
    let mut frame: LispFrameRef = frame.into();

    if frame.name.eq(arg) {
        return;
    }

    frame.name = arg;

    let title = format!("{}", arg.force_string());

    frame.wr_output().set_title(&title);
}

extern "C" fn get_focus_frame(frame: *mut Lisp_Frame) -> LispObject {
    let frame: LispFrameRef = frame.into();
    let dpyinfo = frame.wr_output().display_info();

    let focus_frame = dpyinfo.get_inner().focus_frame;

    match focus_frame.is_null() {
        true => Qnil,
        false => focus_frame.into(),
    }
}

// This tries to wait until the frame is really visible, depending on
// the value of Vx_wait_for_event_timeout.
// However, if the window manager asks the user where to position
// the frame, this will return before the user finishes doing that.
// The frame will not actually be visible at that time,
// but it will become visible later when the window manager
// finishes with it.
extern "C" fn make_frame_visible_invisible(f: *mut Lisp_Frame, visible: bool) {
    let mut frame: LispFrameRef = f.into();

    frame.set_visible(visible as u32);

    let output = frame.wr_output();

    if visible {
        output.show_window();
    } else {
        output.hide_window();
    }
}

extern "C" fn iconify_frame(f: *mut Lisp_Frame) {
    let mut frame: LispFrameRef = f.into();

    frame.set_iconified(true);

    frame.wr_output().hide_window()
}

extern "C" fn mouse_position(
    fp: *mut *mut Lisp_Frame,
    _insist: i32,
    bar_window: *mut LispObject,
    part: *mut scroll_bar_part::Type,
    x: *mut LispObject,
    y: *mut LispObject,
    _timestamp: *mut Time,
) {
    let dpyinfo = {
        let frame: LispFrameRef = unsafe { (*fp).into() };
        frame.wr_display_info()
    };

    // Clear the mouse-moved flag for every frame on this display.
    for mut frame in all_frames() {
        if frame.wr_display_info() == dpyinfo {
            frame.set_mouse_moved(false);
        }
    }

    unsafe { *bar_window = Qnil };
    unsafe { *part = 0 };

    let dpyinfo = dpyinfo.get_inner();
    let cursor_pos: PhysicalPosition<i32> =
        dpyinfo.input_processor.current_cursor_position().cast();

    unsafe { *x = cursor_pos.x.into() };
    unsafe { *y = cursor_pos.y.into() };
}

extern "C" fn update_end(f: *mut Lisp_Frame) {
    let mut dpyinfo = {
        let frame: LispFrameRef = f.into();
        frame.wr_display_info()
    };

    let mut dpyinfo = dpyinfo.get_raw();

    // Mouse highlight may be displayed again.
    dpyinfo.mouse_highlight.set_mouse_face_defer(false);
}

extern "C" fn free_pixmap(f: *mut Lisp_Frame, pixmap: Emacs_Pixmap) {
    // take back ownership and RAII will drop resource.
    let pixmap = unsafe { Box::from_raw(pixmap as *mut WrPixmap) };

    let image_key = pixmap.image_key;

    let frame: LispFrameRef = f.into();
    frame.wr_output().delete_image(image_key);
}

extern "C" fn delete_frame(f: *mut Lisp_Frame) {
    let frame: LispFrameRef = f.into();
    let mut output = frame.wr_output();

    let display_info = output.display_info();
    let window_id = output.get_window().id();

    display_info.get_inner().outputs.remove(&window_id);

    // Take back output ownership and destroy it
    let _ = unsafe { Box::from_raw(output.as_rust_ptr()) };
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

    // Terminal hooks
    // Other hooks are NULL by default.
    terminal.get_string_resource_hook = Some(get_string_resource);
    terminal.set_new_font_hook = Some(new_font);
    terminal.defined_color_hook = Some(defined_color);
    terminal.frame_visible_invisible_hook = Some(frame_visible_invisible);
    terminal.clear_frame_hook = Some(clear_frame);
    terminal.read_socket_hook = Some(read_input_event);
    terminal.fullscreen_hook = Some(fullscreen);
    terminal.implicit_set_name_hook = Some(implicitly_set_name);
    terminal.get_focus_frame = Some(get_focus_frame);
    terminal.frame_visible_invisible_hook = Some(make_frame_visible_invisible);
    terminal.iconify_frame_hook = Some(iconify_frame);
    terminal.mouse_position_hook = Some(mouse_position);
    terminal.update_end_hook = Some(update_end);
    terminal.free_pixmap = Some(free_pixmap);
    terminal.delete_frame_hook = Some(delete_frame);

    terminal
}

pub fn wr_term_init(display_name: LispObject) -> DisplayInfoRef {
    let dpyinfo = Box::new(DisplayInfo::new());
    let mut dpyinfo_ref = DisplayInfoRef::new(Box::into_raw(dpyinfo));

    let mut terminal = wr_create_terminal(dpyinfo_ref);

    // Pretend that we are X while actually wr
    let mut kboard = allocate_keyboard(Qx);

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
