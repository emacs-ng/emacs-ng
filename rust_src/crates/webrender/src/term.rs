use crate::frame::LispFrameWindowSystemExt;
use crate::output::OutputRef;
use std::ptr;
use std::{cmp::max, ffi::CString};

use webrender::api::units::LayoutPoint;
use webrender::api::units::LayoutRect;

use crate::frame::LispFrameExt;
use crate::fringe::get_or_create_fringe_bitmap;
use crate::renderer::Renderer;
use crate::{
    color::{color_to_xcolor, lookup_color_by_name_or_hex, pixel_to_color},
    cursor::{draw_bar_cursor, draw_filled_cursor, draw_hollow_box_cursor},
    image::WrPixmap,
    util::HandyDandyRectBuilder,
};

use emacs::{
    bindings::{
        block_input, display_and_set_cursor, draw_window_fringes, face_id, glyph_row_area,
        gui_clear_cursor, gui_draw_right_divider, gui_draw_vertical_border, image as Emacs_Image,
        run, unblock_input,
    },
    bindings::{
        draw_fringe_bitmap_params, fontset_from_font, glyph_row, glyph_string, terminal,
        text_cursor_kinds, Emacs_Color, Emacs_Pixmap, Fprovide, Fredisplay, Fredraw_display,
        Fredraw_frame,
    },
    font::LispFontRef,
    frame::{LispFrameRef, Lisp_Frame},
    globals::{Qnil, Qt, Qwr},
    glyph::GlyphStringRef,
    lisp::{ExternalPtr, LispObject},
    window::{LispWindowRef, Lisp_Window},
};

pub type TerminalRef = ExternalPtr<terminal>;

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_update_window_begin(w: *mut Lisp_Window) {}

#[no_mangle]
pub extern "C" fn wr_update_window_end(
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
    frame.canvas().flush();
}

#[no_mangle]
pub extern "C" fn wr_flush_display(f: *mut Lisp_Frame) {
    let frame: LispFrameRef = f.into();

    frame.canvas().flush();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_after_update_window_line(w: *mut Lisp_Window, desired_row: *mut glyph_row) {
    let window: LispWindowRef = w.into();

    if !unsafe { (*desired_row).mode_line_p() } && !window.pseudo_window_p() {
        unsafe { (*desired_row).set_redraw_fringe_bitmaps_p(true) };
    }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_draw_glyph_string(s: *mut glyph_string) {
    let s: GlyphStringRef = s.into();

    let mut frame: LispFrameRef = s.f.into();

    frame.draw_glyph_string(s);
}

#[no_mangle]
pub extern "C" fn wr_draw_fringe_bitmap(
    window: *mut Lisp_Window,
    row: *mut glyph_row,
    p: *mut draw_fringe_bitmap_params,
) {
    let window: LispWindowRef = window.into();
    let mut frame: LispFrameRef = window.get_frame();

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

    let image = get_or_create_fringe_bitmap(frame, which, p);

    let face = unsafe { (*p).face };

    let background_color = pixel_to_color(unsafe { (*face).background });

    let bitmap_color = if unsafe { (*p).cursor_p() } {
        frame.cursor_color()
    } else if unsafe { (*p).overlay_p() } {
        background_color
    } else {
        pixel_to_color(unsafe { (*face).foreground })
    };

    frame.draw_fringe_bitmap(
        pos,
        image,
        bitmap_color,
        background_color,
        image_clip_rect,
        clear_rect,
        row_rect,
    );
}

#[no_mangle]
pub extern "C" fn wr_draw_window_divider(
    window: *mut Lisp_Window,
    x0: i32,
    x1: i32,
    y0: i32,
    y1: i32,
) {
    let window: LispWindowRef = window.into();
    let mut frame: LispFrameRef = window.get_frame();

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

    frame.draw_window_divider(color, color_first, color_last, x0, x1, y0, y1);
}

#[no_mangle]
pub extern "C" fn wr_draw_vertical_window_border(
    window: *mut Lisp_Window,
    x: i32,
    y0: i32,
    y1: i32,
) {
    let window: LispWindowRef = window.into();
    let mut frame: LispFrameRef = window.get_frame();

    let face = frame.face_from_id(face_id::VERTICAL_BORDER_FACE_ID);

    frame.draw_vertical_window_border(face, x, y0, y1);
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_clear_frame_area(f: *mut Lisp_Frame, x: i32, y: i32, width: i32, height: i32) {
    let mut frame: LispFrameRef = f.into();

    let color = pixel_to_color(frame.background_pixel);

    frame.clear_area(color, x, y, width, height);
}

#[no_mangle]
pub extern "C" fn wr_draw_window_cursor(
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

#[no_mangle]
pub extern "C" fn wr_get_string_resource(
    _rdb: *mut libc::c_void,
    _name: *const libc::c_char,
    _class: *const libc::c_char,
) -> *const libc::c_char {
    ptr::null()
}

#[no_mangle]
pub extern "C" fn wr_new_font(
    frame: *mut Lisp_Frame,
    font_object: LispObject,
    fontset: i32,
) -> LispObject {
    let mut frame: LispFrameRef = frame.into();

    let font = LispFontRef::from_vectorlike(font_object.as_vectorlike().unwrap()).as_font_mut();

    let fontset = if fontset < 0 {
        unsafe { fontset_from_font(font_object) }
    } else {
        fontset
    };

    frame.set_fontset(fontset);

    if frame.font() == font.into() {
        return font_object;
    }

    frame.set_font(font.into());

    frame.line_height = unsafe { (*font).height };
    frame.column_width = unsafe { (*font).average_width };

    font_object
}

pub extern "C" fn wr_defined_color(
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

pub extern "C" fn wr_clear_frame(f: *mut Lisp_Frame) {
    let frame: LispFrameRef = f.into();
    let mut output = frame.canvas();

    output.clear_display_list_builder();

    let width = frame.pixel_width;
    let height = frame.pixel_height;

    wr_clear_frame_area(f, 0, 0, width, height);
}

#[no_mangle]
pub extern "C" fn wr_scroll_run(w: *mut Lisp_Window, run: *mut run) {
    let window: LispWindowRef = w.into();
    let mut frame = window.get_frame();

    let (x, y, width, height) = window.area_box(glyph_row_area::ANY_AREA);

    let from_y = unsafe { (*run).current_y + window.top_edge_y() };
    let to_y = unsafe { (*run).desired_y + window.top_edge_y() };

    let scroll_height = unsafe { (*run).height };

    // Cursor off.  Will be switched on again in gui_update_window_end.
    unsafe { gui_clear_cursor(w) };

    frame.scroll(x, y, width, height, from_y, to_y, scroll_height);
}

#[no_mangle]
pub extern "C" fn wr_update_end(f: *mut Lisp_Frame) {
    let mut dpyinfo = {
        let frame: LispFrameRef = f.into();
        frame.display_info()
    };

    let mut dpyinfo = dpyinfo.get_raw();

    // Mouse highlight may be displayed again.
    dpyinfo.mouse_highlight.set_mouse_face_defer(false);
}

#[no_mangle]
pub extern "C" fn wr_free_pixmap(f: *mut Lisp_Frame, pixmap: Emacs_Pixmap) {
    // take back ownership and RAII will drop resource.
    let pixmap = unsafe { Box::from_raw(pixmap as *mut WrPixmap) };

    let image_key = pixmap.image_key;

    let frame: LispFrameRef = f.into();
    frame.canvas().delete_image(image_key);
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_baseline_offset(output: OutputRef) -> i32 {
    0
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_pixel(ximg: *mut Emacs_Image, x: i32, y: i32) -> i32 {
    unimplemented!();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_put_pixel(ximg: *mut Emacs_Image, x: i32, y: i32, pixel: u64) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wr_can_use_native_image_api(image_type: LispObject) -> bool {
    crate::image::can_use_native_image_api(image_type)
}

#[no_mangle]
pub extern "C" fn wr_load_image(
    frame: LispFrameRef,
    img: *mut Emacs_Image,
    spec_file: LispObject,
    spec_data: LispObject,
) -> bool {
    crate::image::load_image(frame, img, spec_file, spec_data)
}

#[no_mangle]
pub extern "C" fn wr_transform_image(
    frame: LispFrameRef,
    img: *mut Emacs_Image,
    width: i32,
    height: i32,
    rotation: f64,
) {
    crate::image::transform_image(frame, img, width, height, rotation);
}

#[no_mangle]
pub extern "C" fn image_pixmap_draw_cross(
    _frame: LispFrameRef,
    _pixmap: Emacs_Pixmap,
    _x: i32,
    _y: i32,
    _width: i32,
    _height: u32,
    _color: u64,
) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn image_sync_to_pixmaps(_frame: LispFrameRef, _img: *mut Emacs_Image) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wr_adjust_canvas_size(
    f: *mut Lisp_Frame,
    _width: ::libc::c_int,
    _height: ::libc::c_int,
) {
    let frame: LispFrameRef = f.into();
    if frame.is_visible() && !frame.output().is_null() && frame.resized_p() {
        let size = frame.canvas().device_size();
        frame.canvas().resize(&size);
        spin_sleep::sleep(std::time::Duration::from_millis(16));
        unsafe { Fredisplay(Qt) };
        unsafe { Fredraw_display() };
        unsafe { Fredraw_frame(frame.into()) };
    }
}

#[no_mangle]
#[allow(unused_doc_comments)]
pub extern "C" fn syms_of_webrender() {
    def_lisp_sym!(Qwr, "wr");
    unsafe {
        Fprovide(Qwr, Qnil);
    }

    #[cfg(feature = "capture")]
    {
        let wr_capture_sym =
            CString::new("wr-capture").expect("Failed to create string for intern function call");
        def_lisp_sym!(Qwr_capture, "wr-capture");
        unsafe {
            Fprovide(
                emacs::bindings::intern_c_string(wr_capture_sym.as_ptr()),
                Qnil,
            );
        }
    }
}
