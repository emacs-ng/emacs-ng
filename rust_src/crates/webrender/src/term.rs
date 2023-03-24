use crate::frame::LispFrameWindowSystemExt;
use crate::image::ImageExt;
use crate::image::ImageRef;
use crate::output::OutputRef;
use emacs::multibyte::LispStringRef;
use lisp_macros::lisp_fn;
use std::ptr;
use std::{cmp::max, ffi::CString};

use webrender::api::units::LayoutPoint;
use webrender::api::units::LayoutRect;

use super::font::{WRFont, WRFontRef};
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
        gui_clear_cursor, gui_draw_right_divider, gui_draw_vertical_border, image, run,
        unblock_input,
    },
    bindings::{
        draw_fringe_bitmap_params, fontset_from_font, glyph_row, glyph_string, terminal,
        text_cursor_kinds, Emacs_Color, Emacs_Pixmap, Fprovide,
    },
    font::LispFontRef,
    frame::{LispFrameRef, Lisp_Frame},
    globals::{Qnil, Qwr},
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
    let scale = frame.canvas().scale();

    let row_rect: LayoutRect = unsafe {
        let (window_x, window_y, window_width, _) = window.area_box(glyph_row_area::ANY_AREA);

        let x = window_x;

        let row_y = window.frame_pixel_y(max(0, (*row).y));
        let y = max(row_y, window_y);

        let width = window_width;
        let height = (*row).visible_height;

        (x, y).by(width, height, scale)
    };

    let which = unsafe { (*p).which };

    let pos_x = unsafe { (*p).x };
    let pos_y = unsafe { (*p).y };

    let pos = LayoutPoint::new(pos_x as f32, pos_y as f32);

    let image_clip_rect: LayoutRect = {
        let width = unsafe { (*p).wd };
        let height = unsafe { (*p).h };

        if which > 0 {
            (pos_x, pos_y).by(width, height, scale)
        } else {
            LayoutRect::zero()
        }
    };

    let clear_rect = if unsafe { (*p).bx >= 0 && !(*p).overlay_p() } {
        unsafe { ((*p).bx, (*p).by).by((*p).nx, (*p).ny, scale) }
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
    let wr_font = WRFontRef::new(font as *mut WRFont);

    frame.line_height = wr_font.font.height;
    frame.column_width = wr_font.font.average_width;

    let pixel_width = frame.text_cols * frame.column_width;
    let pixel_height = frame.text_lines * frame.line_height;

    /* Now make the frame display the given font.  */
    frame.adjust_size(pixel_width, pixel_height, 3, false, emacs::globals::Qfont);

    #[cfg(window_system_winit)]
    {
        use crate::window_system::api::dpi::PhysicalSize;
        use crate::window_system::frame::LispFrameWinitExt;

        let size = PhysicalSize::new(
            (pixel_width as f64 * frame.scale_factor()) as u32,
            (pixel_height as f64 * frame.scale_factor()) as u32,
        );
        frame.set_inner_size(size);
    }

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

    let size = frame.canvas().device_size();

    wr_clear_frame_area(f, 0, 0, size.width, size.height);
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
    let frame: LispFrameRef = f.into();
    frame.canvas().delete_image_by_pixmap(pixmap);

    // take back ownership and RAII will drop resource.
    let _ = unsafe { Box::from_raw(pixmap as *mut WrPixmap) };
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_baseline_offset(output: OutputRef) -> i32 {
    0
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_pixel(ximg: *mut image, x: i32, y: i32) -> i32 {
    unimplemented!();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_put_pixel(ximg: *mut image, x: i32, y: i32, pixel: u64) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wr_can_use_native_image_api(image_type: LispObject) -> bool {
    crate::image::can_use_native_image_api(image_type)
}

#[no_mangle]
pub extern "C" fn wr_load_image(
    frame: LispFrameRef,
    img: *mut image,
    _spec_file: LispObject,
    _spec_data: LispObject,
) -> bool {
    let image: ImageRef = img.into();
    image.load(frame)
}

#[no_mangle]
pub extern "C" fn wr_transform_image(
    frame: LispFrameRef,
    img: *mut image,
    width: i32,
    height: i32,
    rotation: f64,
) {
    let image: ImageRef = img.into();
    image.transform(frame, width, height, rotation);
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
pub extern "C" fn image_sync_to_pixmaps(_frame: LispFrameRef, _img: *mut image) {
    unimplemented!();
}

/// Capture the contents of the current WebRender frame and
/// save them to a folder relative to the current working directory.
///
/// If START-SEQUENCE is not nil, start capturing each WebRender frame to disk.
/// If there is already a sequence capture in progress, stop it and start a new
/// one, with the new path and flags.
#[allow(unused_variables)]
#[lisp_fn(min = "2")]
pub fn wr_api_capture(path: LispStringRef, bits_raw: LispObject, start_sequence: LispObject) {
    #[cfg(not(feature = "capture"))]
    error!("Webrender capture not avaiable");
    #[cfg(feature = "capture")]
    {
        use emacs::frame::window_frame_live_or_selected;
        use std::fs::{create_dir_all, File};
        use std::io::Write;

        let path = std::path::PathBuf::from(path.to_utf8());
        match create_dir_all(&path) {
            Ok(_) => {}
            Err(err) => {
                error!("Unable to create path '{:?}' for capture: {:?}", &path, err);
            }
        };
        let bits_raw = unsafe {
            emacs::bindings::check_integer_range(
                bits_raw,
                webrender::CaptureBits::SCENE.bits() as i64,
                webrender::CaptureBits::all().bits() as i64,
            )
        };

        let frame = emacs::frame::window_frame_live_or_selected(Qnil);
        let canvas = frame.canvas();
        let bits = webrender::CaptureBits::from_bits(bits_raw as _).unwrap();
        let revision_file_path = path.join("wr.txt");
        message!("Trying to save webrender capture under {:?}", &path);

        // api call here can possibly make Emacs panic. For example there isn't
        // enough disk space left. `panic::catch_unwind` isn't support here.
        if start_sequence.is_nil() {
            canvas.render_api.save_capture(path, bits);
        } else {
            canvas.render_api.start_capture_sequence(path, bits);
        }

        match File::create(revision_file_path) {
            Ok(mut file) => {
                if let Err(err) = write!(&mut file, "{}", "") {
                    error!("Unable to write webrender revision: {:?}", err)
                }
            }
            Err(err) => error!(
                "Capture triggered, creating webrender revision info skipped: {:?}",
                err
            ),
        }
    }
}

/// Stop a capture begun with `wr--capture'.
#[lisp_fn(min = "0")]
pub fn wr_api_stop_capture_sequence() {
    #[cfg(not(feature = "capture"))]
    error!("Webrender capture not avaiable");
    #[cfg(feature = "capture")]
    {
        use emacs::frame::window_frame_live_or_selected;

        message!("Stop capturing WR state");
        let frame = emacs::frame::window_frame_live_or_selected(Qnil);
        let canvas = frame.canvas();
        canvas.render_api.stop_capture_sequence();
    }
}

#[no_mangle]
pub extern "C" fn wr_handle_scale_factor_change(_f: *mut Lisp_Frame, _scale_factor: f64) {
    #[cfg(window_system_pgtk)]
    {
        let mut frame: LispFrameRef = _f.into();
        frame.handle_scale_factor_change(_scale_factor);
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

include!(concat!(env!("OUT_DIR"), "/term_exports.rs"));
