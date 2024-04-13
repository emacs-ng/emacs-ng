//! wrterm.rs

use crate::cursor::build_mouse_cursors;
use crate::frame::FrameExtWinit;
use crate::input::keysym_to_emacs_key_name;
use crate::term::winit_term_init;
use emacs_sys::bindings::gui_update_cursor;
use emacs_sys::bindings::selected_frame;
use emacs_sys::bindings::Fredraw_frame;
use emacs_sys::color::color_to_pixel;
use emacs_sys::display_traits::FrameParam;
use emacs_sys::frame::Frame;
use emacs_sys::output::Output;
use emacs_sys::terminal::TerminalRef;
use font::register_swash_font_driver;
use raw_window_handle::RawWindowHandle;
use std::ffi::CString;
use std::ptr;
use std::sync::Mutex;
use webrender_api::*;
use winit::dpi::LogicalSize;

use emacs_sys::bindings::output_method;
use winit::monitor::MonitorHandle;

use lisp_macros::lisp_fn;

use emacs_sys::color::lookup_color_by_name_or_hex;
use emacs_sys::output::OutputRef;
use raw_window_handle::RawDisplayHandle;

use emacs_sys::{
    bindings::globals,
    bindings::hash_table_weakness_t::Weak_None,
    bindings::{
        block_input, build_string, gui_figure_window_size, hashtest_eql, list3i, make_fixnum,
        make_hash_table, make_monitor_attribute_list, unblock_input, Display, Emacs_Rectangle,
        Fcons, Fcopy_alist, Fmake_vector, Fprovide, MonitorInfo, Vframe_list, Vwindow_list, Window,
        CHECK_STRING,
    },
    definitions::EmacsInt,
    frame::{all_frames, window_frame_live_or_selected, FrameRef},
    globals::{
        QAndroidNdk,
        QAppKit,
        QDrm,
        QGbm,
        QHaiku,
        QOrbital,
        QUiKit,
        QWayland,
        QWeb, //QWebCanvas, QWebOffscreenCanvas,
        QWin32,
        QWinRt,
        QXcb,
        QXlib,
        Qbackground_color,
        Qbox,
        Qicon,
        Qnil,
        Qright,
        Qt,
        Qunbound,
        Qwinit,
        // Qx_create_frame_1,
        // Qx_create_frame_2,
    },
    lisp::{ExternalPtr, LispObject},
};

pub use emacs_sys::display_info::DisplayInfoRef;

pub type DisplayRef = ExternalPtr<Display>;

#[no_mangle]
pub static tip_frame: LispObject = Qnil;

#[no_mangle]
pub static WINIT_DISPLAY_LIST: Mutex<DisplayInfoRef> =
    Mutex::new(DisplayInfoRef::new(ptr::null_mut()));

fn winit_display_list() -> DisplayInfoRef {
    let v = WINIT_DISPLAY_LIST.lock().map_or_else(
        |e| {
            log::error!("failed to read winit_display_list: {e:?}");
            DisplayInfoRef::new(ptr::null_mut())
        },
        |v| v.clone(),
    );
    v
}

fn set_winit_display_list(dpyinfo: DisplayInfoRef) {
    match WINIT_DISPLAY_LIST.lock().and_then(|mut v| Ok(*v = dpyinfo)) {
        Err(e) => log::error!("failed to update winit_display_list: {e:?}"),
        _ => {}
    }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn winit_display_available() -> bool {
    let display = winit_display_list();
    !display.is_null()
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn winit_get_window_desc(_: OutputRef) -> Window {
    0
}

#[no_mangle]
pub extern "C" fn winit_get_display_info(output: OutputRef) -> DisplayInfoRef {
    output.display_info()
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn winit_get_display(display_info: DisplayInfoRef) -> DisplayRef {
    DisplayRef::new(ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn winit_set_background_color(f: *mut Frame, arg: LispObject, _old_val: LispObject) {
    let mut frame: FrameRef = f.into();

    let color = lookup_color_by_name_or_hex(&format!("{}", arg.as_string().unwrap()))
        .unwrap_or_else(|| ColorF::WHITE);

    let pixel = color_to_pixel(color);

    frame.background_pixel = pixel;
    frame.set_background_color(color);

    frame.update_face_from_frame_param(Qbackground_color, arg);

    if frame.is_visible() {
        unsafe { Fredraw_frame(frame.into()) };
    }
}

#[no_mangle]
pub extern "C" fn winit_set_cursor_color(f: *mut Frame, arg: LispObject, _old_val: LispObject) {
    let frame: FrameRef = f.into();
    let color_str: String = arg.into();

    let color_str = format!("{}", color_str);
    let color = lookup_color_by_name_or_hex(&color_str);

    if let Some(color) = color {
        frame.set_cursor_color(color);
    }

    if frame.is_visible() {
        unsafe { gui_update_cursor(f, false) };
        unsafe { gui_update_cursor(f, true) };
    }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn get_keysym_name(keysym: i32) -> *mut libc::c_char {
    let name = keysym_to_emacs_key_name(keysym);

    name as *mut libc::c_char
}

#[no_mangle]
pub extern "C" fn check_x_display_info(obj: LispObject) -> DisplayInfoRef {
    if obj.is_nil() {
        let frame = window_frame_live_or_selected(obj);

        if (frame.output_method() == output_method::output_winit) && frame.is_live() {
            return frame.display_info();
        }

        let winit_display_list = winit_display_list();
        if !winit_display_list.is_null() {
            return winit_display_list;
        }

        error!("Winit frames are not in use or not initialized");
    }

    if let Some(terminal) = obj.as_terminal() {
        if terminal.type_ != output_method::output_winit {
            error!("Terminal {} is not a webrender display", terminal.id);
        }

        let dpyinfo = DisplayInfoRef::new(unsafe { terminal.display_info.winit as *mut _ });

        return dpyinfo;
    }

    if let Some(display_name) = obj.as_string() {
        let display_name = display_name.to_string();
        let mut dpyinfo = winit_display_list();

        while !dpyinfo.is_null() {
            if dpyinfo
                .name_list_element
                .force_cons()
                .car()
                .force_string()
                .to_string()
                == display_name
            {
                return dpyinfo;
            }

            dpyinfo = DisplayInfoRef::new(dpyinfo.next as *mut _);
        }

        x_open_connection(obj, Qnil, Qnil);

        let winit_display_list = winit_display_list();
        if !winit_display_list.is_null() {
            return winit_display_list;
        }

        error!("Display on {} not responding.", display_name);
    }

    let frame = window_frame_live_or_selected(obj);
    return frame.display_info();
}

// Move the mouse to position pixel PIX_X, PIX_Y relative to frame F.
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn frame_set_mouse_pixel_position(f: FrameRef, pix_x: i32, pix_y: i32) {
    unsafe { block_input() };
    // set mouse
    unsafe { unblock_input() };
}

// STRING in a "tooltip" window on frame FRAME.
// A tooltip window is a small X window displaying a string.

// This is an internal function; Lisp code should call `tooltip-show'.

// FRAME nil or omitted means use the selected frame.

// PARMS is an optional list of frame parameters which can be used to
// change the tooltip's appearance.

// Automatically hide the tooltip after TIMEOUT seconds.  TIMEOUT nil
// means use the default timeout from the `x-show-tooltip-timeout'
// variable.

// If the list of frame parameters PARMS contains a `left' parameter,
// display the tooltip at that x-position.  If the list of frame parameters
// PARMS contains no `left' but a `right' parameter, display the tooltip
// right-adjusted at that x-position. Otherwise display it at the
// x-position of the mouse, with offset DX added (default is 5 if DX isn't
// specified).

// Likewise for the y-position: If a `top' frame parameter is specified, it
// determines the position of the upper edge of the tooltip window.  If a
// `bottom' parameter but no `top' frame parameter is specified, it
// determines the position of the lower edge of the tooltip window.
// Otherwise display the tooltip window at the y-position of the mouse,
// with offset DY added (default is -10).

// A tooltip's maximum size is specified by `x-max-tooltip-size'.
// Text larger than the specified size is clipped.
#[lisp_fn(min = "1")]
pub fn x_show_tip(
    _string: LispObject,
    _frame: LispObject,
    _parms: LispObject,
    _timeout: LispObject,
    _dx: LispObject,
    _dy: LispObject,
) -> LispObject {
    //TODO
    Qnil
}

/// Hide the current tooltip window, if there is any.
/// Value is t if tooltip was open, nil otherwise.
#[lisp_fn]
pub fn x_hide_tip() -> LispObject {
    //TODO
    Qnil
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
pub fn winit_create_frame(params: LispObject) -> FrameRef {
    // x_get_arg modifies params.
    let params = unsafe { Fcopy_alist(params) };

    log::debug!("Creating a new frame with params: {:?}", params);

    // Use this general default value to start with
    // until we know if this frame has a specified name.
    unsafe {
        globals.Vx_resource_name = globals.Vinvocation_name;
    }

    let mut dpyinfo = DisplayInfoRef::null();
    let terminal_or_display = dpyinfo.terminal_or_display_arg(params);
    let mut dpyinfo = check_x_display_info(terminal_or_display);

    let mut f = FrameRef::build(dpyinfo, params);
    let frame = LispObject::from(f);

    f.set_output_method(output_method::output_winit);
    let mut output = Box::new(Output::default());
    build_mouse_cursors(&mut output.as_mut());
    // Remeber to destory the Output object when frame destoried.
    let output = Box::into_raw(output);
    f.output_data.winit = output as *mut Output;
    f.init_winit_data();
    f.set_fontset(-1);
    f.set_display_info(dpyinfo);

    let parent_id = dpyinfo.gui_arg(params, FrameParam::ParentId);
    if parent_id.is_not_nil() {
        f.output().parent_desc =
            unsafe { emacs_sys::bindings::XFIXNAT(parent_id) as emacs_sys::bindings::Window };
        f.output().set_explicit_parent(true);
    } else {
        f.output().parent_desc = f.display_info().root_window;
        f.output().set_explicit_parent(false);
    }

    //TODO do something specific to each display
    match f.raw_display_handle().unwrap() {
        RawDisplayHandle::UiKit(_) => {
            todo!()
        }
        RawDisplayHandle::AppKit(_) => {
            todo!()
        }
        RawDisplayHandle::Orbital(_) => {
            todo!()
        }
        RawDisplayHandle::Xlib(_) | RawDisplayHandle::Xcb(_) => {
            // TODO apply x resources
        }
        RawDisplayHandle::Wayland(_) => {
            // TODO maybe apply gsettings
        }
        RawDisplayHandle::Drm(_) => {}
        RawDisplayHandle::Gbm(_) => {}
        RawDisplayHandle::Windows(_) => {
            // TODO apply settings from registry
        }
        RawDisplayHandle::Web(_) => {}
        RawDisplayHandle::Android(_) => {}
        RawDisplayHandle::Haiku(_) => {}
        _ => {}
    }

    register_swash_font_driver(f.as_mut());

    // //TODO Make it a global state for assert when both GLYPH_DEBUG ENABLE_CHECKING are enabled
    // let image_cache_refcount = if !f.image_cache().is_null() { f.image_cache().refcount} else {0};
    // //TODO Make it a global state for assert when both GLYPH_DEBUG is enabled
    // let dpyinfo_refcount = dpyinfo.reference_count;

    f.gui_default_parameter(params, FrameParam::FontBackend, Qnil);
    // We rely on Rust font-index crate to choose a generic Monospace font
    f.gui_default_parameter(params, FrameParam::Font, "Monospace");

    if f.font().is_null() {
        unsafe { emacs_sys::bindings::delete_frame(frame, emacs_sys::globals::Qnoelisp) };
        error!("Invalid frame font");
    }

    f.gui_default_parameter(params, FrameParam::BorderWidth, 0);
    f.gui_default_parameter(params, FrameParam::InternalBorderWidth, 0);
    f.gui_default_parameter(params, FrameParam::ChildFrameBorderWidth, 0);
    f.gui_default_parameter(params, FrameParam::RightDividerWidth, 0);
    f.gui_default_parameter(params, FrameParam::BottomDividerWidth, 0);
    f.gui_default_parameter(params, FrameParam::VerticalScrollBars, Qright);
    f.gui_default_parameter(params, FrameParam::HorizontalScrollBars, Qnil);
    f.gui_default_parameter(params, FrameParam::ForegroundColor, "black");
    f.gui_default_parameter(params, FrameParam::BackgroundColor, "white");
    f.gui_default_parameter(params, FrameParam::MouseColor, "black");
    f.gui_default_parameter(params, FrameParam::BorderColor, "black");
    f.gui_default_parameter(params, FrameParam::ScreenGamma, Qnil);
    f.gui_default_parameter(params, FrameParam::LineSpacing, Qnil);
    f.gui_default_parameter(params, FrameParam::LeftFringe, Qnil);
    f.gui_default_parameter(params, FrameParam::RightFringe, Qnil);
    f.gui_default_parameter(params, FrameParam::NoSpecialGlyphs, Qnil);
    f.gui_default_parameter(params, FrameParam::ScrollBarForeground, Qnil);
    f.gui_default_parameter(params, FrameParam::ScrollBarBackground, Qnil);

    f.init_faces();

    /* Set the menu-bar-lines and tool-bar-lines parameters.  We don't
    look up the X resources controlling the menu-bar and tool-bar
    here; they are processed specially at startup, and reflected in
    the values of the mode variables.  */
    let menu_bar_lines = if unsafe { globals.Vmenu_bar_mode.is_nil() } {
        0
    } else {
        1
    };
    f.gui_default_parameter_no_x_resource(params, FrameParam::MenuBarLines, menu_bar_lines);
    let tab_bar_lines = if unsafe { globals.Vtab_bar_mode.is_nil() } {
        0
    } else {
        1
    };
    f.gui_default_parameter_no_x_resource(params, FrameParam::TabBarLines, tab_bar_lines);
    let tool_bar_lines = if unsafe { globals.Vtool_bar_mode.is_nil() } {
        0
    } else {
        1
    };
    f.gui_default_parameter_no_x_resource(params, FrameParam::ToolBarLines, tool_bar_lines);

    f.gui_default_parameter(params, FrameParam::BufferPredicate, Qnil);
    f.gui_default_parameter(params, FrameParam::Title, Qnil);
    f.gui_default_parameter(params, FrameParam::WaitForWm, Qt);
    f.gui_default_parameter(params, FrameParam::ToolBarPosition, f.tool_bar_position);
    f.gui_default_parameter(params, FrameParam::InhibitDoubleBuffering, Qnil);

    /* We need to do this after creating the X window, so that the
    icon-creation functions can say whose icon they're describing.  */
    f.gui_default_parameter(params, FrameParam::IconType, true);
    f.gui_default_parameter(params, FrameParam::AutoRaise, false);
    f.gui_default_parameter(params, FrameParam::AutoLower, false);
    f.gui_default_parameter(params, FrameParam::CursorType, Qbox);
    f.gui_default_parameter(params, FrameParam::ScrollBarWidth, 0);
    f.gui_default_parameter(params, FrameParam::ScrollBarHeight, 0);
    f.gui_default_parameter(params, FrameParam::Alpha, 0);
    f.gui_default_parameter(params, FrameParam::AlphaBackground, 0);
    f.gui_default_parameter(params, FrameParam::NoFocusOnMap, false);
    f.gui_default_parameter(params, FrameParam::NoAcceptFocus, false);
    f.gui_default_parameter(params, FrameParam::Fullscreen, Qnil);

    /* Compute the size of the winit window.  */
    // FIXME what to do with the window_prompting here
    let _window_prompting = unsafe { gui_figure_window_size(f.as_mut(), params, true, true) };
    // TODO Create the menu bar.

    f.set_can_set_window_size(true);

    // f.text_width = f.pixel_to_text_width(f.pixel_width as i32);
    // f.text_height = f.pixel_to_text_height(f.pixel_height as i32);

    // f.adjust_size(f.text_width, f.text_height, 5, true, Qx_create_frame_1);

    // f.adjust_size(f.text_width, f.text_height, 0, true, Qx_create_frame_2);

    /* Make the window appear on the frame and enable display, unless
    the caller says not to.  However, with explicit parent, Emacs
    cannot control visibility, so don't try.  */
    if f.output().explicit_parent() {
        let visibility = dpyinfo.gui_arg(params, FrameParam::Visibility);
        let visibility = if visibility.base_eq(Qunbound) {
            Qt
        } else {
            visibility
        };
        let height = dpyinfo.gui_arg(params, FrameParam::Height);
        let width = dpyinfo.gui_arg(params, FrameParam::Width);
        if visibility.eq(Qicon) {
            f.set_was_invisible(true);
            f.iconify();
        } else {
            if visibility.is_not_nil() {
                f.set_visible_(true);
            } else {
                f.set_was_invisible(true);
            }
        }
        /* Leave f->was_invisible true only if height or width were
        specified too.  This takes effect only when we are not called
        from `x-create-frame-with-faces' (see above comment).  */
        let was_invisible =
            f.was_invisible() && (!height.base_eq(Qunbound) || !width.base_eq(Qunbound));
        f.set_was_invisible(was_invisible);
        f.store_param(FrameParam::Visibility, visibility);
    }

    /* Works if frame has been already mapped.  */
    f.gui_default_parameter(params, FrameParam::SkipTaskbar, false);
    /* The `z-group' parameter works only for visible frames.  */
    f.gui_default_parameter(params, FrameParam::ZGroup, false);

    //     /* Initialize `default-minibuffer-frame' in case this is the first
    //    frame on this terminal.  */
    // if (FRAME_HAS_MINIBUF_P (f)
    //     && (!FRAMEP (KVAR (kb, Vdefault_minibuffer_frame))
    //         || !FRAME_LIVE_P (XFRAME (KVAR (kb, Vdefault_minibuffer_frame)))))
    //   kset_default_minibuffer_frame (kb, frame);

    // /* All remaining specified parameters, which have not been "used"
    //    by gui_display_get_arg and friends, now go in the misc. alist of the frame.  */
    // for (tem = parms; CONSP (tem); tem = XCDR (tem))
    //   if (CONSP (XCAR (tem)) && !NILP (XCAR (XCAR (tem))))
    //     fset_param_alist (f, Fcons (XCAR (tem), f->param_alist));

    /* Make sure windows on this frame appear in calls to next-window
    and similar functions.  */
    unsafe { Vwindow_list = Qnil };

    // let mut dpyinfo = frame.display_info();

    dpyinfo.highlight_frame = f.as_mut();

    f.setup_winit();

    /* Now consider the frame official.  */
    f.terminal().reference_count += 1;
    f.display_info().reference_count += 1;
    unsafe { Vframe_list = Fcons(frame, Vframe_list) };

    f
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
    let display = if display.is_nil() {
        unsafe { build_string("".as_ptr() as *const ::libc::c_char) }
    } else {
        display
    };

    unsafe { CHECK_STRING(display) };

    let mut display_info = winit_term_init(display);

    // Put this display on the chain.
    display_info.next = winit_display_list().as_mut();
    set_winit_display_list(display_info);
    Qnil
}

/// Internal function called by `display-color-p', which see.
#[lisp_fn(min = "0")]
pub fn xw_display_color_p(_terminal: LispObject) -> LispObject {
    // webrender support color display
    Qt
}

/// Return t if the X display supports shades of gray.
/// Note that color displays do support shades of gray.
/// The optional argument TERMINAL specifies which display to ask about.
/// TERMINAL should be a terminal object, a frame or a display name (a string).
/// If omitted or nil, that stands for the selected frame's display.
#[lisp_fn(min = "0")]
pub fn x_display_grayscale_p(_terminal: LispObject) -> LispObject {
    // webrender support shades of gray
    Qt
}

/// Internal function called by `color-defined-p', which see.
#[lisp_fn(min = "1")]
pub fn xw_color_defined_p(color: LispObject, _frame: LispObject) -> LispObject {
    let color_str = format!("{}", color.force_string());
    match lookup_color_by_name_or_hex(&color_str) {
        Some(_) => Qt,
        None => Qnil,
    }
}

/// Internal function called by `color-values', which see.
#[lisp_fn(min = "1")]
pub fn xw_color_values(color: LispObject, _frame: Option<FrameRef>) -> LispObject {
    let color_str = format!("{}", color.force_string());
    match lookup_color_by_name_or_hex(&color_str) {
        Some(c) => unsafe {
            list3i(
                (c.r * u16::MAX as f32) as i64,
                (c.g * u16::MAX as f32) as i64,
                (c.b * u16::MAX as f32) as i64,
            )
        },
        None => Qnil,
    }
}

/// Request that dnd events are made for ClientMessages with ATOM.
/// ATOM can be a symbol or a string.  The ATOM is interned on the display that
/// FRAME is on.  If FRAME is nil, the selected frame is used.
#[lisp_fn(min = "1")]
pub fn x_register_dnd_atom(_atom: LispObject, _frame: LispObject) -> LispObject {
    Qnil
}

/// Change window property PROP to VALUE on the X window of FRAME.
/// PROP must be a string.  VALUE may be a string or a list of conses,
/// numbers and/or strings.  If an element in the list is a string, it is
/// converted to an atom and the value of the atom is used.  If an element
/// is a cons, it is converted to a 32 bit number where the car is the 16
/// top bits and the cdr is the lower 16 bits.
///
/// FRAME nil or omitted means use the selected frame.
/// If TYPE is given and non-nil, it is the name of the type of VALUE.
/// If TYPE is not given or nil, the type is STRING.
/// FORMAT gives the size in bits of each element if VALUE is a list.
/// It must be one of 8, 16 or 32.
/// If VALUE is a string or FORMAT is nil or not given, FORMAT defaults to 8.
/// If OUTER-P is non-nil, the property is changed for the outer X window of
/// FRAME.  Default is to change on the edit X window.
#[lisp_fn(min = "2")]
pub fn x_change_window_property(
    _prop: LispObject,
    value: LispObject,
    _frame: LispObject,
    _type: LispObject,
    _format: LispObject,
    _outer_p: LispObject,
    _window_id: LispObject,
) -> LispObject {
    value
}

/// Return the number of color cells of the X display TERMINAL.
/// The optional argument TERMINAL specifies which display to ask about.
/// TERMINAL should be a terminal object, a frame or a display name (a string).
/// If omitted or nil, that stands for the selected frame's display.
#[lisp_fn(min = "0")]
pub fn x_display_color_cells(obj: LispObject) -> LispObject {
    // FIXME: terminal object or display name (a string) is not implemented
    let frame = window_frame_live_or_selected(obj);
    let terminal = frame.terminal();

    let mut color_bits = terminal.get_color_bits();

    // Truncate color_bits to 24 to avoid integer overflow.
    // Some displays says 32, but only 24 bits are actually significant.
    // There are only very few and rare video cards that have more than
    // 24 significant bits.  Also 24 bits is more than 16 million colors,
    // it "should be enough for everyone".
    if color_bits > 24 {
        color_bits = 24;
    }

    let cells = (2 as EmacsInt).pow(color_bits as u32);

    unsafe { make_fixnum(cells) }
}

/// Return the number of bitplanes of the X display TERMINAL.
/// The optional argument TERMINAL specifies which display to ask about.
/// TERMINAL should be a terminal object, a frame or a display name (a string).
/// If omitted or nil, that stands for the selected frame's display.
/// \(On MS Windows, this function does not accept terminal objects.)
#[lisp_fn(min = "0")]
pub fn x_display_planes(obj: LispObject) -> LispObject {
    // FIXME: terminal object or display name (a string) is not implemented
    let frame = window_frame_live_or_selected(obj);
    let terminal = frame.terminal();
    let color_bits = terminal.get_color_bits();

    // color_bits as EmacsInt
    unsafe { make_fixnum(color_bits.into()) }
}

/// Send the size hints for frame FRAME to the window manager.
/// If FRAME is omitted or nil, use the selected frame.
/// Signal error if FRAME is not an X frame.
#[lisp_fn(min = "0")]
pub fn x_wm_set_size_hint(_frame: LispObject) {}

/// Return the visual class of the X display TERMINAL.
/// The value is one of the symbols `static-gray', `gray-scale',
/// `static-color', `pseudo-color', `true-color', or `direct-color'.
/// \(On MS Windows, the second and last result above are not possible.)
///
/// The optional argument TERMINAL specifies which display to ask about.
/// TERMINAL should a terminal object, a frame or a display name (a string).
/// If omitted or nil, that stands for the selected frame's display.
/// \(On MS Windows, this function does not accept terminal objects.)
#[lisp_fn(min = "0")]
pub fn x_display_visual_class(_terminal: LispObject) -> LispObject {
    new_unibyte_string!("true-color")
}

pub fn winit_monitor_to_emacs_monitor(m: MonitorHandle) -> (MonitorInfo, Option<CString>) {
    let dpi_factor = m.scale_factor();

    let physical_pos = m.position();
    let physical_size = m.size();

    let logical_pos = physical_pos.to_logical::<i32>(dpi_factor);
    let logical_size = physical_size.to_logical::<u32>(dpi_factor);

    let geom = Emacs_Rectangle {
        x: logical_pos.x,
        y: logical_pos.y,
        width: logical_size.width,
        height: logical_size.height,
    };

    let physical_size: (u32, u32) = physical_size.into();

    let name = m.name().and_then(|s| CString::new(s).ok());

    let name_c_ptr = name
        .as_ref()
        .map(|s| s.as_ptr())
        .unwrap_or_else(|| ptr::null_mut());

    let monitor_info = MonitorInfo {
        geom,
        work: geom,
        mm_width: physical_size.0 as i32,
        mm_height: physical_size.1 as i32,
        name: name_c_ptr as *mut i8,
    };

    (monitor_info, name)
}

/// Return a list of physical monitor attributes on the X display TERMINAL.
///
/// The optional argument TERMINAL specifies which display to ask about.
/// TERMINAL should be a terminal object, a frame or a display name (a string).
/// If omitted or nil, that stands for the selected frame's display.
///
/// In addition to the standard attribute keys listed in
/// `display-monitor-attributes-list', the following keys are contained in
/// the attributes:
///
/// source -- String describing the source from which multi-monitor
/// information is obtained, one of \"Gdk\", \"XRandr\",
/// \"Xinerama\", or \"fallback\"
///
/// Internal use only, use `display-monitor-attributes-list' instead.
#[lisp_fn(min = "0")]
pub fn winit_display_monitor_attributes_list(terminal: LispObject) -> LispObject {
    let terminal: TerminalRef = terminal.into();

    let monitors: Vec<_> = terminal
        .available_monitors()
        .and_then(|ms| Some(ms.collect()))
        .unwrap_or(Vec::new());
    let primary_monitor = terminal.primary_monitor();

    let primary_monitor_index = monitors
        .iter()
        .position(|m| {
            primary_monitor
                .as_ref()
                .and_then(|pm| Some(pm.name() == m.name()))
                .unwrap_or(false)
        })
        .unwrap_or(0);

    let emacs_monitor_infos: Vec<_> = monitors
        .iter()
        .map(|m| winit_monitor_to_emacs_monitor(m.clone()))
        .collect();

    let mut emacs_monitors: Vec<_> = emacs_monitor_infos.iter().map(|(m, _)| m.clone()).collect();

    let n_monitors = monitors.len();
    let mut monitor_frames = unsafe { Fmake_vector(n_monitors.into(), Qnil).as_vector_unchecked() };

    for frame in all_frames() {
        let current_monitor = frame.current_monitor();

        if current_monitor.is_none() {
            continue;
        }

        let current_monitor = current_monitor.unwrap();

        if let Some(index) = monitors
            .iter()
            .position(|m| m.name() == current_monitor.name())
        {
            monitor_frames.set(index, unsafe {
                Fcons(frame.into(), monitor_frames.get(index))
            });
        }
    }

    let source = CString::new("fallback").unwrap();

    unsafe {
        make_monitor_attribute_list(
            emacs_monitors.as_mut_ptr(),
            n_monitors as i32,
            primary_monitor_index as i32,
            monitor_frames.into(),
            source.as_ptr(),
        )
    }
}

fn winit_screen_size(terminal: LispObject) -> LogicalSize<i32> {
    let dpyinfo = check_x_display_info(terminal);
    let terminal = dpyinfo.terminal();
    terminal
        .primary_monitor()
        .or_else(|| terminal.available_monitors()?.next())
        .and_then(|m| Some(m.size().to_logical::<i32>(m.scale_factor())))
        .unwrap_or(LogicalSize::new(0, 0))
}

/// Return the width in pixels of the X display TERMINAL.
/// The optional argument TERMINAL specifies which display to ask about.
/// TERMINAL should be a terminal object, a frame or a display name (a string).
/// If omitted or nil, that stands for the selected frame's display.
/// \(On MS Windows, this function does not accept terminal objects.)
///
/// On \"multi-monitor\" setups this refers to the pixel width for all
/// physical monitors associated with TERMINAL.  To get information for
/// each physical monitor, use `display-monitor-attributes-list'.
#[lisp_fn(min = "0")]
pub fn x_display_pixel_width(terminal: LispObject) -> LispObject {
    unsafe { make_fixnum(winit_screen_size(terminal).width as i64) }
}

/// Return the height in pixels of the X display TERMINAL.
/// The optional argument TERMINAL specifies which display to ask about.
/// TERMINAL should be a terminal object, a frame or a display name (a string).
/// If omitted or nil, that stands for the selected frame's display.
/// \(On MS Windows, this function does not accept terminal objects.)
///
/// On \"multi-monitor\" setups this refers to the pixel height for all
/// physical monitors associated with TERMINAL.  To get information for
/// each physical monitor, use `display-monitor-attributes-list'.
#[lisp_fn(min = "0")]
pub fn x_display_pixel_height(terminal: LispObject) -> LispObject {
    unsafe { make_fixnum(winit_screen_size(terminal).height as i64) }
}

/// Assert an X selection of type SELECTION and value VALUE.
/// SELECTION is a symbol, typically `PRIMARY', `SECONDARY', or `CLIPBOARD'.
/// \(Those are literal upper-case symbol names, since that's what X expects.)
/// VALUE is typically a string, or a cons of two markers, but may be
/// anything that the functions on `selection-converter-alist' know about.
///
/// FRAME should be a frame that should own the selection.  If omitted or
/// nil, it defaults to the selected frame.
///
/// On Nextstep, FRAME is unused.
#[lisp_fn(min = "2")]
pub fn x_own_selection_internal(
    _selection: LispObject,
    value: LispObject,
    frame: LispObject,
) -> LispObject {
    let frame = if frame.is_nil() {
        unsafe { selected_frame }
    } else {
        frame
    };
    let frame: FrameRef = frame.into();
    let content = value.force_string().to_utf8();
    frame
        .terminal()
        .winit_data()
        .and_then(|mut d| d.clipboard.set_text(content).ok())
        .map(|_| value)
        .unwrap_or(Qnil)
}

/// Return text selected from some X window.
/// SELECTION-SYMBOL is typically `PRIMARY', `SECONDARY', or `CLIPBOARD'.
/// \(Those are literal upper-case symbol names, since that's what X expects.)
/// TARGET-TYPE is the type of data desired, typically `STRING'.
///
/// TIME-STAMP is the time to use in the XConvertSelection call for foreign
/// selections.  If omitted, defaults to the time for the last event.
///
/// TERMINAL should be a terminal object or a frame specifying the X
/// server to query.  If omitted or nil, that stands for the selected
/// frame's display, or the first available X display.
///
/// On Nextstep, TIME-STAMP and TERMINAL are unused.
#[lisp_fn(min = "2")]
pub fn x_get_selection_internal(
    _selection_symbol: LispObject,
    _target_type: LispObject,
    _time_stamp: LispObject,
    terminal: LispObject,
) -> LispObject {
    let terminal: TerminalRef = terminal.into();
    terminal
        .winit_data()
        .and_then(|mut d| d.clipboard.get_text().ok())
        .map(|contents| LispObject::from(contents))
        .unwrap_or(Qnil)
}

/// Whether the current Emacs process owns the given X Selection.
/// The arg should be the name of the selection in question, typically one of
/// the symbols `PRIMARY', `SECONDARY', or `CLIPBOARD'.
/// \(Those are literal upper-case symbol names, since that's what X expects.)
/// For convenience, the symbol nil is the same as `PRIMARY',
/// and t is the same as `SECONDARY'.
///
/// TERMINAL should be a terminal object or a frame specifying the X
/// server to query.  If omitted or nil, that stands for the selected
/// frame's display, or the first available X display.
///
/// On Nextstep, TERMINAL is unused.
#[lisp_fn(min = "0")]
pub fn x_selection_owner_p(_selection: LispObject, _terminal: LispObject) -> LispObject {
    Qnil
}

/// Whether there is an owner for the given X selection.
/// SELECTION should be the name of the selection in question, typically
/// one of the symbols `PRIMARY', `SECONDARY', `CLIPBOARD', or
/// `CLIPBOARD_MANAGER' (X expects these literal upper-case names.)  The
/// symbol nil is the same as `PRIMARY', and t is the same as `SECONDARY'.
///
/// TERMINAL should be a terminal object or a frame specifying the X
/// server to query.  If omitted or nil, that stands for the selected
/// frame's display, or the first available X display.
///
/// On Nextstep, TERMINAL is unused.
#[lisp_fn(min = "0")]
pub fn x_selection_exists_p(_selection: LispObject, _terminal: LispObject) -> LispObject {
    Qnil
}

/// Return edge coordinates of FRAME.
/// FRAME must be a live frame and defaults to the selected one.  The return
/// value is a list of the form (LEFT, TOP, RIGHT, BOTTOM).  All values are
/// in pixels relative to the origin - the position (0, 0) - of FRAME's
/// display.
///
/// If optional argument TYPE is the symbol `outer-edges', return the outer
/// edges of FRAME.  The outer edges comprise the decorations of the window
/// manager (like the title bar or external borders) as well as any external
/// menu or tool bar of FRAME.  If optional argument TYPE is the symbol
/// `native-edges' or nil, return the native edges of FRAME.  The native
/// edges exclude the decorations of the window manager and any external
/// menu or tool bar of FRAME.  If TYPE is the symbol `inner-edges', return
/// the inner edges of FRAME.  These edges exclude title bar, any borders,
/// menu bar or tool bar of FRAME.
#[lisp_fn(min = "0")]
pub fn winit_frame_edges(frame: LispObject, type_: LispObject) -> LispObject {
    let frame = window_frame_live_or_selected(frame);
    frame.edges(type_)
}

/// Return the name of the RawWindowHandle of FRAME.
/// The optional argument FRAME specifies which frame to ask about.
/// FRAME should be a frame object.
/// If omitted or nil, that stands for the selected frame.
#[lisp_fn(min = "0")]
pub fn winit_raw_window_handle_name(frame: LispObject) -> LispObject {
    let frame: FrameRef = window_frame_live_or_selected(frame);

    match frame.raw_window_handle().unwrap() {
        RawWindowHandle::UiKit(_) => QUiKit,
        RawWindowHandle::AppKit(_) => QAppKit,
        RawWindowHandle::Orbital(_) => QOrbital,
        RawWindowHandle::Xlib(_) => QXlib,
        RawWindowHandle::Xcb(_) => QXcb,
        RawWindowHandle::Wayland(_) => QWayland,
        RawWindowHandle::Drm(_) => QDrm,
        RawWindowHandle::Gbm(_) => QGbm,
        RawWindowHandle::Win32(_) => QWin32,
        RawWindowHandle::WinRt(_) => QWinRt,
        RawWindowHandle::Web(_) => QWeb,
        // RawWindowHandle::WebCanvas(_) => QWebCanvas,
        // RawWindowHandle::WebOffscreenCanvas(_) => QWebOffscreenCanvas,
        RawWindowHandle::AndroidNdk(_) => QAndroidNdk,
        RawWindowHandle::Haiku(_) => QHaiku,
        _ => Qnil,
    }
}

#[no_mangle]
#[allow(unused_doc_comments)]
pub extern "C" fn syms_of_winit_term() {
    def_lisp_sym!(Qwinit, "winit");
    def_lisp_sym!(QUiKit, "UiKit");
    def_lisp_sym!(QAppKit, "AppKit");
    def_lisp_sym!(QOrbital, "Orbital");
    def_lisp_sym!(QXlib, "Xlib");
    def_lisp_sym!(QXcb, "Xcb");
    def_lisp_sym!(QWayland, "Wayland");
    def_lisp_sym!(QDrm, "Drm");
    def_lisp_sym!(QGbm, "Gbm");
    def_lisp_sym!(QWin32, "Win32");
    def_lisp_sym!(QWinRt, "WinRt");
    def_lisp_sym!(QWeb, "Web");
    // def_lisp_sym!(QWebCanvas, "WebCanvas");
    // def_lisp_sym!(QWebOffscreenCanvas, "WebOffscreenCanvas");
    def_lisp_sym!(QAndroidNdk, "AndroidNdk");
    def_lisp_sym!(QHaiku, "Haiku");

    unsafe {
        Fprovide(Qwinit, Qnil);
        Fprovide(QUiKit, Qnil);
        Fprovide(QAppKit, Qnil);
        Fprovide(QOrbital, Qnil);
        Fprovide(QXlib, Qnil);
        Fprovide(QXcb, Qnil);
        Fprovide(QWayland, Qnil);
        Fprovide(QDrm, Qnil);
        Fprovide(QGbm, Qnil);
        Fprovide(QWin32, Qnil);
        Fprovide(QWinRt, Qnil);
        Fprovide(QWeb, Qnil);
        // Fprovide(QWebCanvas, Qnil);
        // Fprovide(QWebOffscreenCanvas, Qnil);
        Fprovide(QAndroidNdk, Qnil);
        Fprovide(QHaiku, Qnil);
    }

    let winit_keysym_table = unsafe { make_hash_table(&hashtest_eql, 900, Weak_None, false) };

    // Hash table of character codes indexed by X keysym codes.
    #[rustfmt::skip]
    defvar_lisp!(Vwinit_keysym_table, "winit-keysym-table", winit_keysym_table);

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

    // Whether to enable X clipboard manager support.
    // If non-nil, then whenever Emacs is killed or an Emacs frame is deleted
    // while owning the X clipboard, the clipboard contents are saved to the
    // clipboard manager if one is present.
    #[rustfmt::skip]
    defvar_lisp!(Vx_select_enable_clipboard_manager, "x-select-enable-clipboard-manager", Qt);
}

include!(concat!(env!("OUT_DIR"), "/fns_exports.rs"));
