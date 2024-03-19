//! Generic frame functions.
use crate::bindings::adjust_frame_size;
use crate::bindings::change_frame_size;
use crate::bindings::face;
use crate::bindings::face_id;
use crate::bindings::frame;
use crate::bindings::frame_dimension;
use crate::bindings::init_frame_faces;
use crate::bindings::pvec_type;
use crate::bindings::store_frame_param;
use crate::bindings::update_face_from_frame_parameter;
use crate::bindings::Fassq;
use crate::bindings::Fselected_frame;
use crate::bindings::Lisp_Type;
use crate::bindings::Vframe_list;
use crate::globals::Qframe_live_p;
use crate::globals::Qframep;
use crate::globals::Qnil;
use crate::lisp::ExternalPtr;
use crate::lisp::LispObject;
use crate::list::LispConsCircularChecks;
use crate::list::LispConsEndChecks;
use crate::vector::LispVectorlikeRef;
use crate::window::LispWindowRef;

#[cfg(have_window_system)]
use {
    crate::bindings::{gui_default_parameter, resource_types, vertical_scroll_bar_type},
    crate::display_info::DisplayInfoRef,
    crate::font::FontRef,
    crate::output::{OutputExtWindowSystem, OutputRef},
    crate::terminal::TerminalRef,
    // raw_window_handle::{RawDisplayHandle, RawWindowHandle},
    std::ffi::CString,
    // webrender_api::units::{DeviceIntSize, LayoutSize},
};

pub type Frame = frame;

/// FrameRef is a reference to the Frame
/// However a reference is guaranteed to point to an existing frame
/// therefore no NULL checks are needed while using it
#[allow(dead_code)]
pub type FrameRef = ExternalPtr<Frame>;

impl FrameRef {
    pub fn root_window(self) -> LispWindowRef {
        self.root_window.into()
    }

    pub fn is_live(self) -> bool {
        !self.terminal.is_null()
    }

    // Pixel-width of internal border lines.
    pub fn internal_border_width(self) -> i32 {
        unsafe { frame_dimension(self.internal_border_width) }
    }

    pub fn is_visible(self) -> bool {
        self.visible() != 0
    }

    pub fn has_tooltip(self) -> bool {
        #[cfg(have_window_system)]
        {
            self.tooltip()
        }
        #[cfg(not(have_window_system))]
        {
            false
        }
    }

    pub fn total_fringe_width(self) -> i32 {
        self.left_fringe_width + self.right_fringe_width
    }

    pub fn vertical_scroll_bar_type(self) -> u32 {
        #[cfg(have_window_system)]
        {
            return (*self).vertical_scroll_bar_type();
        }
        #[cfg(not(have_window_system))]
        0
    }

    pub fn scroll_bar_area_width(self) -> i32 {
        #[cfg(have_window_system)]
        {
            match self.vertical_scroll_bar_type() {
                vertical_scroll_bar_type::vertical_scroll_bar_left
                | vertical_scroll_bar_type::vertical_scroll_bar_right => {
                    self.config_scroll_bar_width
                }
                _ => 0,
            }
        }
        #[cfg(not(have_window_system))]
        {
            0
        }
    }

    pub fn horizontal_scroll_bar_height(self) -> i32 {
        #[cfg(have_window_system)]
        {
            if self.horizontal_scroll_bars() {
                self.config_scroll_bar_height
            } else {
                0
            }
        }
        #[cfg(not(have_window_system))]
        {
            0
        }
    }

    pub fn top_margin_height(self) -> i32 {
        self.menu_bar_height + self.tool_bar_height
    }

    pub fn pixel_to_text_width(self, width: i32) -> i32 {
        width
            - self.scroll_bar_area_width()
            - self.total_fringe_width()
            - 2 * self.internal_border_width()
    }

    pub fn pixel_to_text_height(self, height: i32) -> i32 {
        height
            - self.top_margin_height()
            - self.horizontal_scroll_bar_height()
            - 2 * self.internal_border_width()
    }

    pub fn face_from_id(self, id: face_id) -> Option<*mut face> {
        let cache = self.face_cache;

        let faces_map: &[*mut face] =
            unsafe { std::slice::from_raw_parts_mut((*cache).faces_by_id, (*cache).used as usize) };

        faces_map.get(id as usize).copied()
    }

    pub fn get_param(self, prop: LispObject) -> LispObject {
        match unsafe { Fassq(prop, self.param_alist) }.as_cons() {
            Some(cons) => cons.cdr(),
            None => Qnil,
        }
    }

    #[cfg(have_window_system)]
    pub fn gui_default_parameter(
        mut self,
        alist: LispObject,
        prop: LispObject,
        default: LispObject,
        xprop: &str,
        xclass: &str,
        res_type: resource_types::Type,
    ) {
        let xprop = CString::new(xprop).unwrap();
        let xclass = CString::new(xclass).unwrap();

        unsafe {
            gui_default_parameter(
                self.as_mut(),
                alist,
                prop,
                default,
                xprop.as_ptr(),
                xclass.as_ptr(),
                res_type,
            );
        };
    }

    pub fn change_size(
        mut self,
        new_width: i32,
        new_height: i32,
        pretend: bool,
        delay: bool,
        safe: bool,
    ) {
        unsafe {
            change_frame_size(self.as_mut(), new_width, new_height, pretend, delay, safe);
        }
    }

    pub fn adjust_size(
        mut self,
        new_text_width: i32,
        new_text_height: i32,
        inhibit: i32,
        pretend: bool,
        parameter: LispObject,
    ) {
        unsafe {
            adjust_frame_size(
                self.as_mut(),
                new_text_width,
                new_text_height,
                inhibit,
                pretend,
                parameter,
            );
        }
    }

    pub fn store_param(mut self, prop: LispObject, val: LispObject) {
        unsafe { store_frame_param(self.as_mut(), prop, val) };
    }

    pub fn update_face_from_frame_param(mut self, prop: LispObject, new_val: LispObject) {
        unsafe { update_face_from_frame_parameter(self.as_mut(), prop, new_val) };
    }

    pub fn init_faces(mut self) {
        unsafe { init_frame_faces(self.as_mut()) };
    }

    #[allow(unreachable_code)]
    #[cfg(have_window_system)]
    pub fn output(&self) -> OutputRef {
        #[cfg(feature = "window-system-pgtk")]
        return OutputRef::new(unsafe { self.output_data.pgtk });
        #[cfg(feature = "window-system-winit")]
        return OutputRef::new(unsafe { self.output_data.winit });
        unimplemented!();
    }

    #[cfg(have_window_system)]
    pub fn font(&self) -> FontRef {
        FontRef::new(self.output().font as *mut _)
    }

    #[cfg(have_window_system)]
    pub fn fontset(&self) -> i32 {
        self.output().fontset
    }

    #[cfg(have_window_system)]
    pub fn set_font(&mut self, mut font: FontRef) {
        self.output().font = font.as_mut();
    }

    #[cfg(have_window_system)]
    pub fn set_fontset(&mut self, fontset: i32) {
        self.output().fontset = fontset;
    }

    #[cfg(have_window_system)]
    pub fn display_info(&self) -> DisplayInfoRef {
        self.output().display_info()
    }

    #[cfg(have_window_system)]
    pub fn set_display_info(&mut self, mut dpyinfo: DisplayInfoRef) {
        self.output().display_info = dpyinfo.as_mut();
    }

    #[cfg(have_window_system)]
    pub fn terminal(&self) -> TerminalRef {
        return TerminalRef::new(self.terminal);
    }
}

impl From<LispObject> for FrameRef {
    fn from(o: LispObject) -> Self {
        o.as_frame().unwrap_or_else(|| wrong_type!(Qframep, o))
    }
}

impl From<FrameRef> for LispObject {
    fn from(f: FrameRef) -> Self {
        Self::tag_ptr(f, Lisp_Type::Lisp_Vectorlike)
    }
}

impl From<LispObject> for Option<FrameRef> {
    fn from(o: LispObject) -> Self {
        o.as_vectorlike().and_then(LispVectorlikeRef::as_frame)
    }
}

impl LispObject {
    pub fn is_frame(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_FRAME))
    }

    pub fn as_frame(self) -> Option<FrameRef> {
        self.into()
    }

    pub fn as_live_frame(self) -> Option<FrameRef> {
        self.as_frame()
            .and_then(|f| if f.is_live() { Some(f) } else { None })
    }

    // Same as CHECK_LIVE_FRAME
    pub fn as_live_frame_or_error(self) -> FrameRef {
        self.as_live_frame()
            .unwrap_or_else(|| wrong_type!(Qframe_live_p, self))
    }
}

pub fn window_frame_live_or_selected(object: LispObject) -> FrameRef {
    // Cannot use LispFrameOrSelected because the selected frame is not
    // checked for live.
    if object.is_nil() {
        unsafe { Fselected_frame() }.into()
    } else if let Some(win) = object.as_valid_window() {
        // the window's frame does not need a live check
        win.frame.into()
    } else {
        object.as_live_frame_or_error()
    }
}

pub fn all_frames() -> impl Iterator<Item = FrameRef> {
    let frame_it =
        unsafe { Vframe_list.iter_cars(LispConsEndChecks::off, LispConsCircularChecks::off) }
            .map(FrameRef::from);

    frame_it
}
