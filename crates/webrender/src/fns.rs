use crate::cursor::draw_bar_cursor;
use crate::cursor::draw_filled_cursor;
use crate::cursor::draw_hollow_box_cursor;
use crate::display_info::DisplayInfoExtWr;
use crate::frame::FrameExtWrCommon;
use crate::fringe::get_or_create_fringe_bitmap;
use crate::image::ImageExt;
use crate::image::ImageRef;
use crate::image::WrPixmap;
use crate::output::OutputRef;
use crate::util::HandyDandyRectBuilder;
use emacs_sys::bindings::block_input;
use emacs_sys::bindings::display_and_set_cursor;
use emacs_sys::bindings::draw_fringe_bitmap_params;
use emacs_sys::bindings::draw_window_fringes;
use emacs_sys::bindings::face_id;
use emacs_sys::bindings::fontset_from_font;
use emacs_sys::bindings::globals;
use emacs_sys::bindings::glyph_row;
use emacs_sys::bindings::glyph_string;
use emacs_sys::bindings::gui_clear_cursor;
use emacs_sys::bindings::gui_draw_right_divider;
use emacs_sys::bindings::gui_draw_vertical_border;
use emacs_sys::bindings::image;
use emacs_sys::bindings::lookup_basic_face;
use emacs_sys::bindings::run;
use emacs_sys::bindings::terminal;
use emacs_sys::bindings::text_cursor_kinds;
use emacs_sys::bindings::unblock_input;
use emacs_sys::bindings::Emacs_Color;
use emacs_sys::bindings::Emacs_Pixmap;
use emacs_sys::bindings::Fprovide;
use emacs_sys::bindings::FACE_FROM_ID_OR_NULL;
use emacs_sys::color::color_to_xcolor;
use emacs_sys::color::lookup_color_by_name_or_hex;
use emacs_sys::color::pixel_to_color;
use emacs_sys::display_traits::FaceId;
use emacs_sys::display_traits::GlyphRowArea;
use emacs_sys::display_traits::GlyphRowRef;
use emacs_sys::display_traits::GlyphStringRef;
use emacs_sys::font::LispFontRef;
use emacs_sys::frame::Frame;
use emacs_sys::frame::FrameRef;
use emacs_sys::globals::Qnil;
use emacs_sys::globals::Qwr;
use emacs_sys::lisp::LispObject;
use emacs_sys::multibyte::LispStringRef;
use emacs_sys::terminal::TerminalRef;
use emacs_sys::window::Window;
use emacs_sys::window::WindowRef;
use font::FontInfo;
use font::FontInfoRef;

use lisp_macros::lisp_fn;
use std::cmp::max;
use std::ffi::CString;
use std::ptr;
use webrender::api::units::LayoutPoint;
use webrender::api::units::LayoutRect;
use webrender::api::ColorF;

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_update_window_begin(w: *mut Window) {}

#[no_mangle]
pub extern "C" fn wr_update_window_end(
    window: *mut Window,
    cursor_no_p: bool,
    _mouse_face_overwritten_p: bool,
) {
    let mut window: WindowRef = window.into();

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

    let frame: FrameRef = window.get_frame();
    frame.gl_renderer().flush();
}

#[no_mangle]
pub extern "C" fn wr_flush_display(f: *mut Frame) {
    let frame: FrameRef = f.into();

    frame.gl_renderer().flush();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_after_update_window_line(w: *mut Window, desired_row: *mut glyph_row) {
    let window: WindowRef = w.into();

    if !unsafe { (*desired_row).mode_line_p() } && !window.pseudo_window_p() {
        unsafe { (*desired_row).set_redraw_fringe_bitmaps_p(true) };
    }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_draw_glyph_string(s: *mut glyph_string) {
    let s: GlyphStringRef = s.into();

    let mut frame: FrameRef = s.f.into();

    frame.draw_glyph_string(s);
}

#[no_mangle]
pub extern "C" fn wr_draw_fringe_bitmap(
    window: *mut Window,
    row: *mut glyph_row,
    p: *mut draw_fringe_bitmap_params,
) {
    let window: WindowRef = window.into();
    let mut frame: FrameRef = window.get_frame();
    let scale = frame.gl_renderer().scale();

    let row_rect: LayoutRect = unsafe {
        let (window_x, window_y, window_width, _) = window.area_box(GlyphRowArea::Any);

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
pub extern "C" fn wr_draw_window_divider(window: *mut Window, x0: i32, x1: i32, y0: i32, y1: i32) {
    let window: WindowRef = window.into();
    let mut frame: FrameRef = window.get_frame();

    let face_fg_color = |id: FaceId| -> ColorF {
        frame
            .face_from_id(id)
            .map(|f| f.fg_color())
            .unwrap_or(frame.fg_color())
    };

    let color = face_fg_color(FaceId::WindowDivider);
    let color_first = face_fg_color(FaceId::WindowDividerFirstPixel);
    let color_last = face_fg_color(FaceId::WindowDividerLastPixel);

    frame.draw_window_divider(color, color_first, color_last, x0, x1, y0, y1);
}

#[no_mangle]
pub extern "C" fn wr_draw_vertical_window_border(window: *mut Window, x: i32, y0: i32, y1: i32) {
    let window: WindowRef = window.into();
    let mut frame: FrameRef = window.get_frame();

    let face = frame.face_from_id(face_id::VERTICAL_BORDER_FACE_ID);

    frame.draw_vertical_window_border(face, x, y0, y1);
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_clear_frame_area(f: *mut Frame, x: i32, y: i32, width: i32, height: i32) {
    wr_clear_area(f, x, y, width, height);
}

fn wr_clear_area(f: *mut Frame, x: i32, y: i32, width: i32, height: i32) {
    let mut frame: FrameRef = f.into();

    let color = pixel_to_color(frame.background_pixel);

    frame.clear_area(color, x, y, width, height);
}

#[no_mangle]
pub extern "C" fn wr_draw_window_cursor(
    window: *mut Window,
    row: *mut glyph_row,
    _x: i32,
    _y: i32,
    cursor_type: text_cursor_kinds::Type,
    cursor_width: i32,
    on_p: bool,
    _active_p: bool,
) {
    let mut window: WindowRef = window.into();
    let row: GlyphRowRef = row.into();

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
    frame: *mut Frame,
    font_object: LispObject,
    fontset: i32,
) -> LispObject {
    let mut frame: FrameRef = frame.into();

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
    let wr_font = FontInfoRef::new(font as *mut FontInfo);

    frame.line_height = wr_font.font.height;
    frame.column_width = wr_font.font.average_width;

    let pixel_width = frame.text_cols * frame.column_width;
    let pixel_height = frame.text_lines * frame.line_height;

    /* Now make the frame display the given font.  */
    frame.adjust_size(
        pixel_width,
        pixel_height,
        3,
        false,
        emacs_sys::globals::Qfont,
    );

    font_object
}

#[no_mangle]
pub extern "C" fn wr_defined_color(
    _frame: *mut Frame,
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

#[no_mangle]
pub extern "C" fn wr_clear_frame(f: *mut Frame) {
    let frame: FrameRef = f.into();
    let mut output = frame.gl_renderer();

    output.clear_display_list_builder();

    let size = frame.gl_renderer().device_size();

    wr_clear_frame_area(f, 0, 0, size.width, size.height);
}

#[no_mangle]
pub extern "C" fn wr_scroll_run(w: *mut Window, run: *mut run) {
    let window: WindowRef = w.into();
    let mut frame = window.get_frame();

    let (x, y, width, height) = window.area_box(GlyphRowArea::Any);

    let from_y = unsafe { (*run).current_y + window.top_edge_y() };
    let to_y = unsafe { (*run).desired_y + window.top_edge_y() };

    let scroll_height = unsafe { (*run).height };

    // Cursor off.  Will be switched on again in gui_update_window_end.
    unsafe { gui_clear_cursor(w) };

    frame.scroll(x, y, width, height, from_y, to_y, scroll_height);
}

#[no_mangle]
pub extern "C" fn wr_update_end(f: *mut Frame) {
    let mut dpyinfo = {
        let frame: FrameRef = f.into();
        frame.display_info()
    };

    // Mouse highlight may be displayed again.
    dpyinfo.mouse_highlight.set_mouse_face_defer(false);
}

#[no_mangle]
pub extern "C" fn wr_free_pixmap(f: *mut Frame, pixmap: Emacs_Pixmap) {
    let frame: FrameRef = f.into();
    frame.gl_renderer().delete_image_by_pixmap(pixmap);

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
    frame: FrameRef,
    img: *mut image,
    _spec_file: LispObject,
    _spec_data: LispObject,
) -> bool {
    let image: ImageRef = img.into();
    image.load(frame)
}

#[no_mangle]
pub extern "C" fn wr_transform_image(
    frame: FrameRef,
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
    _frame: FrameRef,
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
pub extern "C" fn image_sync_to_pixmaps(_frame: FrameRef, _img: *mut image) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn gl_clear_under_internal_border(f: *mut Frame) {
    let mut f = FrameRef::new(f);
    let border = f.internal_border_width();
    let width = f.pixel_width;
    let height = f.pixel_height;
    let margin = f.top_margin_height();
    let bottom_margin = f.bottom_margin_height();
    let face_id_fallback = |id: face_id| {
        if unsafe { globals.Vface_remapping_alist.is_not_nil() } {
            unsafe { lookup_basic_face(ptr::null_mut(), f.clone().as_mut(), id as i32) }
        } else {
            id as i32
        }
    };
    let face_id = match f.parent_frame() {
        Some(_) => face_id_fallback(face_id::CHILD_FRAME_BORDER_FACE_ID),
        None => face_id_fallback(face_id::INTERNAL_BORDER_FACE_ID),
    };
    let face = unsafe { FACE_FROM_ID_OR_NULL(f.as_mut(), face_id) };

    unsafe { block_input() };

    if face.is_null() {
        wr_clear_area(f.as_mut(), 0, 0, border, height);
        wr_clear_area(f.as_mut(), 0, margin, width, border);
        wr_clear_area(f.as_mut(), 0, width - border, border, height);
        wr_clear_area(
            f.as_mut(),
            0,
            height - bottom_margin - border,
            width,
            border,
        );
    } else {
        log::error!("unimplemented: clean under internal border with face");
    }

    unsafe { unblock_input() };
}

#[no_mangle]
pub extern "C" fn gl_renderer_parse_color(
    _f: *mut Frame,
    color_name: *const ::libc::c_char,
    xcolor: *mut Emacs_Color,
) -> ::libc::c_int {
    use std::ffi::CStr;
    let color_name: &CStr = unsafe { CStr::from_ptr(color_name) };
    let color_name: &str = color_name.to_str().unwrap();
    if let Some(color) = lookup_color_by_name_or_hex(&format!("{}", color_name.to_owned())) {
        color_to_xcolor(color, xcolor);
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn gl_renderer_free_frame_resources(f: *mut Frame) {
    let mut frame: FrameRef = f.into();
    frame.free_gl_renderer_resources();
}

#[no_mangle]
pub extern "C" fn gl_renderer_free_terminal_resources(terminal: *mut terminal) {
    let terminal: TerminalRef = terminal.into();
    let mut display_info = terminal.display_info();
    display_info.free_gl_renderer_data();
}

/// Fit GL context to frame, reflecting frame/scale factor changes
#[no_mangle]
pub extern "C" fn gl_renderer_fit_context(f: *mut Frame) {
    let frame: FrameRef = f.into();
    if frame.output().is_null() || frame.output().gl_renderer.is_null() {
        return;
    }
    frame.gl_renderer().update();
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
        use emacs_sys::frame::window_frame_live_or_selected;
        use std::fs::create_dir_all;
        use std::fs::File;
        use std::io::Write;

        let path = std::path::PathBuf::from(path.to_utf8());
        match create_dir_all(&path) {
            Ok(_) => {}
            Err(err) => {
                error!("Unable to create path '{:?}' for capture: {:?}", &path, err);
            }
        };
        let bits_raw = unsafe {
            emacs_sys::bindings::check_integer_range(
                bits_raw,
                webrender::CaptureBits::SCENE.bits() as i64,
                webrender::CaptureBits::all().bits() as i64,
            )
        };

        let frame = emacs_sys::frame::window_frame_live_or_selected(Qnil);
        let canvas = frame.gl_renderer();
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
        use emacs_sys::frame::window_frame_live_or_selected;

        message!("Stop capturing WR state");
        let frame = emacs_sys::frame::window_frame_live_or_selected(Qnil);
        let canvas = frame.gl_renderer();
        canvas.render_api.stop_capture_sequence();
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
                emacs_sys::bindings::intern_c_string(wr_capture_sym.as_ptr()),
                Qnil,
            );
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/fns_exports.rs"));
