//! Generic frame functions.
use crate::{
    bindings::{
        adjust_frame_size, change_frame_size, face, face_id, frame, frame_dimension,
        init_frame_faces, pvec_type, store_frame_param, update_face_from_frame_parameter, Fassq,
        Fselected_frame, Lisp_Type, Vframe_list,
    },
    globals::{Qframe_live_p, Qframep, Qnil},
    lisp::{ExternalPtr, LispObject},
    list::{LispConsCircularChecks, LispConsEndChecks},
    vector::LispVectorlikeRef,
    window::LispWindowRef,
};

#[cfg(feature = "window-system")]
use {
    crate::bindings::{gui_default_parameter, resource_types, vertical_scroll_bar_type},
    std::ffi::CString,
};

pub type Lisp_Frame = frame;

/// LispFrameRef is a reference to the LispFrame
/// However a reference is guaranteed to point to an existing frame
/// therefore no NULL checks are needed while using it
#[allow(dead_code)]
pub type LispFrameRef = ExternalPtr<Lisp_Frame>;

impl LispFrameRef {
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
        #[cfg(feature = "window-system")]
        {
            self.tooltip()
        }
        #[cfg(not(feature = "window-system"))]
        {
            false
        }
    }

    pub fn total_fringe_width(self) -> i32 {
        self.left_fringe_width + self.right_fringe_width
    }

    pub fn vertical_scroll_bar_type(self) -> u32 {
        #[cfg(feature = "window-system")]
        {
            (*self).vertical_scroll_bar_type()
        }
        #[cfg(not(feature = "window-system"))]
        0
    }

    pub fn scroll_bar_area_width(self) -> i32 {
        #[cfg(feature = "window-system")]
        {
            match self.vertical_scroll_bar_type() {
                vertical_scroll_bar_type::vertical_scroll_bar_left
                | vertical_scroll_bar_type::vertical_scroll_bar_right => {
                    self.config_scroll_bar_width
                }
                _ => 0,
            }
        }
        #[cfg(not(feature = "window-system"))]
        {
            0
        }
    }

    pub fn horizontal_scroll_bar_height(self) -> i32 {
        #[cfg(feature = "window-system")]
        {
            if self.horizontal_scroll_bars() {
                self.config_scroll_bar_height
            } else {
                0
            }
        }
        #[cfg(not(feature = "window-system"))]
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

    #[cfg(feature = "window-system")]
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
}

impl From<LispObject> for LispFrameRef {
    fn from(o: LispObject) -> Self {
        o.as_frame().unwrap_or_else(|| wrong_type!(Qframep, o))
    }
}

impl From<LispFrameRef> for LispObject {
    fn from(f: LispFrameRef) -> Self {
        Self::tag_ptr(f, Lisp_Type::Lisp_Vectorlike)
    }
}

impl From<LispObject> for Option<LispFrameRef> {
    fn from(o: LispObject) -> Self {
        o.as_vectorlike().and_then(LispVectorlikeRef::as_frame)
    }
}

impl LispObject {
    pub fn is_frame(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_FRAME))
    }

    pub fn as_frame(self) -> Option<LispFrameRef> {
        self.into()
    }

    pub fn as_live_frame(self) -> Option<LispFrameRef> {
        self.as_frame()
            .and_then(|f| if f.is_live() { Some(f) } else { None })
    }

    // Same as CHECK_LIVE_FRAME
    pub fn as_live_frame_or_error(self) -> LispFrameRef {
        self.as_live_frame()
            .unwrap_or_else(|| wrong_type!(Qframe_live_p, self))
    }
}

pub fn window_frame_live_or_selected(object: LispObject) -> LispFrameRef {
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

pub fn all_frames() -> impl Iterator<Item = LispFrameRef> {
    let frame_it =
        unsafe { Vframe_list.iter_cars(LispConsEndChecks::off, LispConsCircularChecks::off) }
            .map(LispFrameRef::from);

    frame_it
}
