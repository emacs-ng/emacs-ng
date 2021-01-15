//! Generic frame functions.
use crate::{
    lisp::{ExternalPtr, LispObject},
    remacs_sys::{frame_dimension, Lisp_Frame, Lisp_Type},
    vector::LispVectorlikeRef,
};

#[cfg(feature = "window-system")]
use {
    crate::remacs_sys::{gui_default_parameter, resource_types, vertical_scroll_bar_type},
    std::ffi::CString,
};

/// LispFrameRef is a reference to the LispFrame
/// However a reference is guaranteed to point to an existing frame
/// therefore no NULL checks are needed while using it
#[allow(dead_code)]
pub type LispFrameRef = ExternalPtr<Lisp_Frame>;

impl LispFrameRef {
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
        let xprop = CString::new(xprop).unwrap().as_ptr();
        let xclass = CString::new(xclass).unwrap().as_ptr();

        unsafe {
            gui_default_parameter(self.as_mut(), alist, prop, default, xprop, xclass, res_type);
        };
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
