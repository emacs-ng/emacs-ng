//! wrterm.rs

use std::ffi::CString;
use std::ptr;

use lisp_macros::lisp_fn;

use crate::{
    frame::LispFrameRef,
    webrender_backend::{
        font::{FontRef, FONT_DRIVER},
        frame::create_frame,
        output::OutputRef,
        term::wr_term_init,
    },
};

use lisp::{
    lisp::{ExternalPtr, LispObject},
    remacs_sys::globals,
    remacs_sys::resource_types::{RES_TYPE_NUMBER, RES_TYPE_STRING, RES_TYPE_SYMBOL},
    remacs_sys::{
        block_input, gui_default_parameter, gui_display_get_arg, hashtest_eql, image,
        make_hash_table, register_font_driver, unblock_input, Display, Emacs_Pixmap, Fcopy_alist,
        Fprovide, Pixmap, Qfont, Qfont_backend, Qminibuffer, Qname, Qnil, Qparent_id, Qterminal,
        Qunbound, Qwr, Qx, WRImage, Window, XrmDatabase, DEFAULT_REHASH_SIZE,
        DEFAULT_REHASH_THRESHOLD,
    },
};

pub use crate::webrender_backend::display_info::{DisplayInfo, DisplayInfoRef};

pub type DisplayRef = ExternalPtr<Display>;
pub type ImageRef = ExternalPtr<WRImage>;

#[no_mangle]
pub static tip_frame: LispObject = Qnil;

#[no_mangle]
pub static mut wr_display_list: DisplayInfoRef = DisplayInfoRef::new(ptr::null_mut());

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_fontset(output: OutputRef) -> i32 {
    unimplemented!();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_font(output: OutputRef) -> FontRef {
    output.get_inner().font
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_window_desc(output: OutputRef) -> Window {
    unimplemented!();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_display_info(output: OutputRef) -> DisplayInfoRef {
    output.get_inner().display_info
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_display(display_info: DisplayInfoRef) -> DisplayRef {
    unimplemented!();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_baseline_offset(output: OutputRef) -> i32 {
    unimplemented!();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_pixel(ximg: ImageRef, x: i32, y: i32) -> i32 {
    unimplemented!();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_free_pixmap(display: DisplayRef, pixmap: Pixmap) -> i32 {
    unimplemented!();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn get_keysym_name(keysym: i32) -> *mut libc::c_char {
    unimplemented!();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_clear_under_internal_border(frame: LispFrameRef) {}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_implicitly_set_name(frame: LispFrameRef, arg: LispObject, oldval: LispObject) {
    unimplemented!();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_set_scroll_bar_default_width(frame: LispFrameRef) {
    // Currently, the web render based GUI does't support scroll bar.
    // So Do nothing.
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_set_scroll_bar_default_height(frame: LispFrameRef) {
    // Currently, the web render based GUI does't support scroll bar.
    // So Do nothing.
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_get_string_resource(
    _rdb: XrmDatabase,
    _name: *const libc::c_char,
    _class: *const libc::c_char,
) -> *mut libc::c_char {
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn check_x_display_info(obj: LispObject) -> DisplayInfoRef {
    if obj.is_nil() {
        unsafe { wr_display_list }
    } else {
        unimplemented!();
    }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_bitmap_icon(frame: LispFrameRef, icon: LispObject) -> bool {
    unimplemented!();
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_focus_frame(frame: LispFrameRef, noactivate: bool) {
    unimplemented!();
}

// FRAME is used only to get a handle on the X display.  We don't pass the
// display info directly because we're called from frame.c, which doesn't
// know about that structure.
#[no_mangle]
pub extern "C" fn x_get_focus_frame(frame: LispFrameRef) -> LispObject {
    let output: OutputRef = unsafe { frame.output_data.wr.into() };
    let dpyinfo = output.get_inner().display_info;

    let focus_frame = dpyinfo.get_inner().focus_frame;

    match focus_frame.is_null() {
        true => Qnil,
        false => focus_frame.into(),
    }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_set_offset(frame: LispFrameRef, xoff: i32, yoff: i32, change_gravity: i32) {
    unimplemented!();
}

// This tries to wait until the frame is really visible, depending on
// the value of Vx_wait_for_event_timeout.
// However, if the window manager asks the user where to position
// the frame, this will return before the user finishes doing that.
// The frame will not actually be visible at that time,
// but it will become visible later when the window manager
// finishes with it.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_make_frame_visible(f: LispFrameRef) {}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_make_frame_invisible(f: LispFrameRef) {}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_iconify_frame(f: LispFrameRef) {}

// Set the pixel height of the tool bar of frame F to HEIGHT.
#[no_mangle]
pub extern "C" fn x_change_tool_bar_height(_f: LispFrameRef, _hight: i32) {
    // Currently, the webrender based GUI does't support tool bar.
    // So Do nothing.
}

// Call this to change the size of frame F's x-window.
// If CHANGE_GRAVITY, change to top-left-corner window gravity
// for this size change and subsequent size changes.
// Otherwise we leave the window gravity unchanged.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn x_set_window_size(
    f: LispFrameRef,
    change_gravity: bool,
    width: i32,
    height: i32,
    pixelwise: bool,
) {
}

// Move the mouse to position pixel PIX_X, PIX_Y relative to frame F.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn frame_set_mouse_pixel_position(f: LispFrameRef, pix_x: i32, pix_y: i32) {
    unsafe { block_input() };
    // set mouse
    unsafe { unblock_input() };
}

#[no_mangle]
pub extern "C" fn image_sync_to_pixmaps(_frame: LispFrameRef, _img: *mut image) {
    unimplemented!();
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

/// Hide the current tooltip window, if there is any.
/// Value is t if tooltip was open, nil otherwise.
#[lisp_fn]
pub fn x_hide_tip() -> bool {
    unimplemented!();
}

/// Make a new X window, which is called a "frame" in Emacs terms.
/// Return an Emacs frame object.  PARMS is an alist of frame parameters.
/// If the parameters specify that the frame should not have a minibuffer,
/// and do not specify a specific minibuffer window to use, then
/// `default-minibuffer-frame' must be a frame whose minibuffer can be
/// shared by the new frame.
///
/// This function is an internal primitive--use `make-frame' instead.
#[lisp_fn]
pub fn x_create_frame(parms: LispObject) -> LispFrameRef {
    // x_get_arg modifies parms.
    let parms = unsafe { Fcopy_alist(parms) };

    // Use this general default value to start with
    // until we know if this frame has a specified name.
    unsafe {
        globals.Vx_resource_name = globals.Vinvocation_name;
    }

    let mut display = unsafe {
        gui_display_get_arg(
            ptr::null_mut(),
            parms,
            Qterminal,
            ptr::null(),
            ptr::null(),
            RES_TYPE_STRING,
        )
    };

    if display.eq(Qunbound) {
        display = Qnil;
    }

    let mut dpyinfo = check_x_display_info(display);

    if dpyinfo.get_inner().terminal.name == ptr::null_mut() {
        error!("Terminal is not live, can't create new frames on it");
    }

    let kb = dpyinfo.get_inner().terminal.kboard;
    let name = unsafe {
        gui_display_get_arg(
            dpyinfo.as_mut(),
            parms,
            Qname,
            ptr::null(),
            ptr::null(),
            RES_TYPE_STRING,
        )
    };

    if !name.is_string() && !name.eq(Qunbound) && !name.is_nil() {
        error!("Invalid frame name--not a string or nil");
    }

    if name.is_string() {
        unsafe {
            globals.Vx_resource_name = name;
        }
    }

    let mut parent = unsafe {
        gui_display_get_arg(
            dpyinfo.as_mut(),
            parms,
            Qparent_id,
            ptr::null(),
            ptr::null(),
            RES_TYPE_NUMBER,
        )
    };

    if parent.eq(Qunbound) {
        parent = Qnil;
    }

    if parent.is_not_nil() {}
    let tem = unsafe {
        gui_display_get_arg(
            dpyinfo.as_mut(),
            parms,
            Qminibuffer,
            CString::new("minibuffer").unwrap().as_ptr(),
            CString::new("Minibuffer").unwrap().as_ptr(),
            RES_TYPE_SYMBOL,
        )
    };

    let mut frame = create_frame(display, dpyinfo, tem, kb.into());

    unsafe {
        register_font_driver(FONT_DRIVER.clone().as_mut(), frame.as_mut());
    };

    unsafe {
        gui_default_parameter(
            frame.as_mut(),
            parms,
            Qfont_backend,
            Qnil,
            CString::new("fontBackend").unwrap().as_ptr(),
            CString::new("FontBackend").unwrap().as_ptr(),
            RES_TYPE_STRING,
        );
    };

    unsafe {
        gui_default_parameter(
            frame.as_mut(),
            parms,
            Qfont,
            "Mono".into(),
            CString::new("font").unwrap().as_ptr(),
            CString::new("Font").unwrap().as_ptr(),
            RES_TYPE_STRING,
        );
    };

    frame
}

/// Open a connection to a display server.
/// DISPLAY is the name of the display to connect to.
/// Optional second arg XRM-STRING is a string of resources in xrdb format.
/// If the optional third arg MUST-SUCCEED is non-nil,
/// terminate Emacs if we can't open the connection.
/// \(In the Nextstep version, the last two arguments are currently ignored.)
#[lisp_fn(min = "1")]
pub fn x_open_connection(
    display: LispObject,
    _xrm_string: LispObject,
    _must_succeed: LispObject,
) -> LispObject {
    let mut display_info = wr_term_init(display);

    // Put this display on the chain.
    unsafe {
        display_info.next = wr_display_list.as_mut();
        wr_display_list = display_info;
    }
    Qnil
}

/// Internal function called by `display-color-p', which see.
#[lisp_fn(min = "0")]
pub fn xw_display_color_p(_terminal: LispObject) -> bool {
    unimplemented!();
}

/// Return t if the X display supports shades of gray.
/// Note that color displays do support shades of gray.
/// The optional argument TERMINAL specifies which display to ask about.
/// TERMINAL should be a terminal object, a frame or a display name (a string).
/// If omitted or nil, that stands for the selected frame's display.
#[lisp_fn(min = "0")]
pub fn x_display_grayscale_p(_terminal: LispObject) -> bool {
    unimplemented!();
}

#[no_mangle]
#[allow(unused_doc_comments)]
pub extern "C" fn syms_of_wrterm() {
    // pretend webrender as a X gui backend, so we can reuse the x-win.el logic
    def_lisp_sym!(Qx, "x");
    def_lisp_sym!(Qwr, "wr");
    unsafe {
        Fprovide(Qx, Qnil);
        Fprovide(Qwr, Qnil);
    }

    let x_keysym_table = unsafe {
        make_hash_table(
            hashtest_eql.clone(),
            900,
            DEFAULT_REHASH_SIZE,
            DEFAULT_REHASH_THRESHOLD,
            Qnil,
            false,
        )
    };

    // Hash table of character codes indexed by X keysym codes.
    #[rustfmt::skip]
    defvar_lisp!(Vx_keysym_table, "x-keysym-table", x_keysym_table);

    // Which toolkit scroll bars Emacs uses, if any.
    // A value of nil means Emacs doesn't use toolkit scroll bars.
    // With the X Window system, the value is a symbol describing the
    // X toolkit.  Possible values are: gtk, motif, xaw, or xaw3d.
    // With MS Windows or Nextstep, the value is t.
    #[rustfmt::skip]
    defvar_lisp!(Vx_toolkit_scroll_bars, "x-toolkit-scroll-bars", Qnil);

    // Non-nil means make use of UNDERLINE_POSITION font properties.
    // A value of nil means ignore them.  If you encounter fonts with bogus
    // UNDERLINE_POSITION font properties, set this to nil.  You can also use
    // `underline-minimum-offset' to override the font's UNDERLINE_POSITION for
    // small font display sizes.
    #[rustfmt::skip]
    defvar_bool!(Vx_use_underline_position_properties, "x-use-underline-position-properties", true);

    // Non-nil means to draw the underline at the same place as the descent line.
    // (If `line-spacing' is in effect, that moves the underline lower by
    // that many pixels.)
    // A value of nil means to draw the underline according to the value of the
    // variable `x-use-underline-position-properties', which is usually at the
    // baseline level.  The default value is nil.
    #[rustfmt::skip]
    defvar_bool!(Vx_underline_at_descent_line, "x-underline-at-descent-line", false);
}

include!(concat!(env!("OUT_DIR"), "/wrterm_exports.rs"));
