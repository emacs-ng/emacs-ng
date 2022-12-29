//! wrterm.rs

use emacs::multibyte::LispStringRef;
use std::ffi::CString;
use std::ptr;

use emacs::bindings::output_method;
use winit::{event::VirtualKeyCode, monitor::MonitorHandle};

use lisp_macros::lisp_fn;

use crate::event_loop::EVENT_LOOP;
use crate::frame::frame_edges;
use crate::frame::LispFrameExt;
use crate::{
    color::lookup_color_by_name_or_hex,
    font::{FontRef, FONT_DRIVER},
    frame::create_frame,
    input::winit_keycode_emacs_key_name,
    output::OutputRef,
    term::wr_term_init,
};

use emacs::{
    bindings::globals,
    bindings::resource_types::{RES_TYPE_NUMBER, RES_TYPE_STRING, RES_TYPE_SYMBOL},
    bindings::{
        block_input, build_string, gui_display_get_arg, hashtest_eql, image as Emacs_Image, list3i,
        make_fixnum, make_hash_table, make_monitor_attribute_list, register_font_driver,
        unblock_input, Display, Emacs_Pixmap, Emacs_Rectangle, Fcons, Fcopy_alist, Fmake_vector,
        Fprovide, MonitorInfo, Vframe_list, Window, CHECK_STRING, DEFAULT_REHASH_SIZE,
        DEFAULT_REHASH_THRESHOLD,
    },
    definitions::EmacsInt,
    frame::{all_frames, window_frame_live_or_selected, LispFrameRef},
    globals::{
        Qbackground_color, Qfont, Qfont_backend, Qforeground_color, Qleft_fringe, Qminibuffer,
        Qname, Qnil, Qparent_id, Qright_fringe, Qt, Qterminal, Qunbound, Qwr, Qx_create_frame_1,
        Qx_create_frame_2,
    },
    lisp::{ExternalPtr, LispObject},
};

pub use crate::display_info::{DisplayInfo, DisplayInfoRef};

pub type DisplayRef = ExternalPtr<Display>;

#[no_mangle]
pub static tip_frame: LispObject = Qnil;

#[no_mangle]
pub static mut wr_display_list: DisplayInfoRef = DisplayInfoRef::new(ptr::null_mut());

#[no_mangle]
pub extern "C" fn wr_get_fontset(output: OutputRef) -> i32 {
    output.fontset
}

#[no_mangle]
pub extern "C" fn wr_get_font(output: OutputRef) -> FontRef {
    output.font
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_window_desc(output: OutputRef) -> Window {
    0
}

#[no_mangle]
pub extern "C" fn wr_get_display_info(output: OutputRef) -> DisplayInfoRef {
    output.display_info()
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn wr_get_display(display_info: DisplayInfoRef) -> DisplayRef {
    DisplayRef::new(ptr::null_mut())
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
pub extern "C" fn get_keysym_name(keysym: i32) -> *mut libc::c_char {
    let name =
        winit_keycode_emacs_key_name(unsafe { std::mem::transmute::<i32, VirtualKeyCode>(keysym) });

    name as *mut libc::c_char
}

#[no_mangle]
pub extern "C" fn check_x_display_info(obj: LispObject) -> DisplayInfoRef {
    if obj.is_nil() {
        let frame = window_frame_live_or_selected(obj);

        if (frame.output_method() == output_method::output_wr) && frame.is_live() {
            return frame.wr_display_info();
        }

        if !unsafe { wr_display_list.is_null() } {
            return unsafe { wr_display_list };
        }

        error!("Webrender windows are not in use or not initialized");
    }

    if let Some(terminal) = obj.as_terminal() {
        if terminal.type_ != output_method::output_wr {
            error!("Terminal {} is not a webrender display", terminal.id);
        }

        let dpyinfo = DisplayInfoRef::new(unsafe { terminal.display_info.wr as *mut _ });

        return dpyinfo;
    }

    if let Some(display_name) = obj.as_string() {
        let display_name = display_name.to_string();
        let mut dpyinfo = unsafe { wr_display_list };

        while !dpyinfo.is_null() {
            if dpyinfo
                .get_raw()
                .name_list_element
                .force_cons()
                .car()
                .force_string()
                .to_string()
                == display_name
            {
                return dpyinfo;
            }

            dpyinfo = DisplayInfoRef::new(dpyinfo.get_raw().next as *mut _);
        }

        x_open_connection(obj, Qnil, Qnil);

        if !unsafe { wr_display_list.is_null() } {
            return unsafe { wr_display_list };
        }

        error!("Display on {} not responding.", display_name);
    }

    let frame = window_frame_live_or_selected(obj);
    return frame.wr_display_info();
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
pub extern "C" fn image_sync_to_pixmaps(_frame: LispFrameRef, _img: *mut Emacs_Image) {
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
pub fn x_hide_tip() -> LispObject {
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
pub fn wr_create_frame(parms: LispObject) -> LispFrameRef {
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
            dpyinfo.get_raw().as_mut(),
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
            dpyinfo.get_raw().as_mut(),
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
        let lcmb = CString::new("minibuffer").unwrap();
        let ucmb = CString::new("Minibuffer").unwrap();
        gui_display_get_arg(
            dpyinfo.get_raw().as_mut(),
            parms,
            Qminibuffer,
            lcmb.as_ptr(),
            ucmb.as_ptr(),
            RES_TYPE_SYMBOL,
        )
    };

    let mut frame = create_frame(display, dpyinfo, tem, kb.into());

    unsafe {
        register_font_driver(&FONT_DRIVER.0 as *const _, frame.as_mut());
    };

    frame.gui_default_parameter(
        parms,
        Qfont_backend,
        Qnil,
        "fontBackend",
        "FontBackend",
        RES_TYPE_STRING,
    );

    frame.gui_default_parameter(
        parms,
        Qfont,
        "Monospace".into(),
        "font",
        "Font",
        RES_TYPE_STRING,
    );

    frame.gui_default_parameter(
        parms,
        Qforeground_color,
        "black".into(),
        "foreground",
        "Foreground",
        RES_TYPE_STRING,
    );
    frame.gui_default_parameter(
        parms,
        Qbackground_color,
        "white".into(),
        "background",
        "Background",
        RES_TYPE_STRING,
    );

    frame.gui_default_parameter(
        parms,
        Qleft_fringe,
        Qnil,
        "leftFringe",
        "LeftFringe",
        RES_TYPE_NUMBER,
    );

    frame.gui_default_parameter(
        parms,
        Qright_fringe,
        Qnil,
        "rightFringe",
        "RightFringe",
        RES_TYPE_NUMBER,
    );

    let output = frame.wr_output();

    let output_size = output.get_inner_size();

    frame.pixel_width = output_size.width as i32;
    frame.pixel_height = output_size.height as i32;

    frame.text_width = frame.pixel_to_text_width(output_size.width as i32);
    frame.text_height = frame.pixel_to_text_height(output_size.height as i32);

    frame.set_can_set_window_size(true);

    frame.adjust_size(
        frame.text_width,
        frame.text_height,
        5,
        true,
        Qx_create_frame_1,
    );

    frame.adjust_size(
        frame.text_width,
        frame.text_height,
        0,
        true,
        Qx_create_frame_2,
    );

    frame.init_faces();

    /* Now consider the frame official.  */
    unsafe { Vframe_list = Fcons(frame.into(), Vframe_list) };

    let mut dpyinfo = output.display_info();

    dpyinfo.get_raw().highlight_frame = frame.as_mut();

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
    let display = if display.is_nil() {
        unsafe { build_string("".as_ptr() as *const ::libc::c_char) }
    } else {
        display
    };

    unsafe { CHECK_STRING(display) };

    let mut event_loop = EVENT_LOOP.lock().unwrap();
    let _native_display = event_loop.open_native_display();

    let mut display_info = wr_term_init(display);

    // Put this display on the chain.
    unsafe {
        display_info.get_raw().next = wr_display_list.get_raw().as_mut();
        wr_display_list = display_info;
    }
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

/// Internal function called by `color-values', which see.
#[lisp_fn(min = "1")]
pub fn xw_color_values(color: LispObject, _frame: Option<LispFrameRef>) -> LispObject {
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

    let output = frame.wr_output();

    let mut color_bits = output.get_color_bits();

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

    let output = frame.wr_output();

    let color_bits = output.get_color_bits();

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

pub fn webrender_monitor_to_emacs_monitor(m: MonitorHandle) -> (MonitorInfo, Option<CString>) {
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
pub fn x_display_monitor_attributes_list(_terminal: LispObject) -> LispObject {
    let event_loop = EVENT_LOOP.lock().unwrap();

    let monitors: Vec<_> = event_loop.get_available_monitors().collect();
    let primary_monitor = event_loop.get_primary_monitor();

    let mut primary_monitor_index = 0;

    for (i, m) in monitors.iter().enumerate() {
        if m.name() == primary_monitor.name() {
            primary_monitor_index = i;
            break;
        }
    }

    let emacs_monitor_infos: Vec<_> = monitors
        .iter()
        .map(|m| webrender_monitor_to_emacs_monitor(m.clone()))
        .collect();

    let mut emacs_monitors: Vec<_> = emacs_monitor_infos.iter().map(|(m, _)| m.clone()).collect();

    let n_monitors = monitors.len();
    let mut monitor_frames = unsafe { Fmake_vector(n_monitors.into(), Qnil).as_vector_unchecked() };

    for frame in all_frames() {
        let output = frame.wr_output();

        let current_monitor = output.get_window().current_monitor();

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
pub fn x_display_pixel_width(_terminal: LispObject) -> LispObject {
    let event_loop = EVENT_LOOP.lock().unwrap();

    let primary_monitor = event_loop.get_primary_monitor();

    let dpi_factor = primary_monitor.scale_factor();

    let physical_size = primary_monitor.size();
    let logical_size = physical_size.to_logical::<i32>(dpi_factor);

    unsafe { make_fixnum(logical_size.width as i64) }
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
pub fn x_display_pixel_height(_terminal: LispObject) -> LispObject {
    let event_loop = EVENT_LOOP.lock().unwrap();

    let primary_monitor = event_loop.get_primary_monitor();

    let dpi_factor = primary_monitor.scale_factor();

    let physical_size = primary_monitor.size();
    let logical_size = physical_size.to_logical::<i32>(dpi_factor);

    unsafe { make_fixnum(logical_size.height as i64) }
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
    _frame: LispObject,
) -> LispObject {
    let mut event_loop = EVENT_LOOP.lock().unwrap();

    let clipboard = event_loop.get_clipboard();

    let content = value.force_string().to_utf8();

    clipboard.set_contents(content).unwrap();

    value
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
    _terminal: LispObject,
) -> LispObject {
    let mut event_loop = EVENT_LOOP.lock().unwrap();

    let clipboard = event_loop.get_clipboard();

    let contents: &str = &clipboard.get_contents().unwrap_or_else(|_e| {
        #[cfg(debug_assertions)]
        message!("x_get_selection_internal: {}", _e);
        "".to_owned()
    });

    contents.into()
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
pub fn x_frame_edges(frame: LispObject, type_: LispObject) -> LispObject {
    frame_edges(frame, type_)
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

        let frame = window_frame_live_or_selected(Qnil);
        let output = frame.wr_output();
        let bits = webrender::CaptureBits::from_bits(bits_raw as _).unwrap();
        let revision_file_path = path.join("wr.txt");
        message!("Trying to save webrender capture under {:?}", &path);

        // api call here can possibly make Emacs panic. For example there isn't
        // enough disk space left. `panic::catch_unwind` isn't support here.
        if start_sequence.is_nil() {
            output.render_api.save_capture(path, bits);
        } else {
            output.render_api.start_capture_sequence(path, bits);
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
        message!("Stop capturing WR state");
        let frame = window_frame_live_or_selected(Qnil);
        let output = frame.wr_output();
        output.render_api.stop_capture_sequence();
    }
}

fn syms_of_wrfont() {
    unsafe {
        register_font_driver(&FONT_DRIVER.0, ptr::null_mut());
    }
}

#[no_mangle]
#[allow(unused_doc_comments)]
pub extern "C" fn syms_of_wrterm() {
    // pretend webrender as a X gui backend, so we can reuse the x-win.el logic
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

    let wr_keysym_table = unsafe {
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
    defvar_lisp!(Vwr_keysym_table, "wr-keysym-table", wr_keysym_table);

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

    syms_of_wrfont();
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/out/wrterm_exports.rs"
));
