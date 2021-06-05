use crate::{
    bindings::{
        glyph_row_area::{self, TEXT_AREA},
        pvec_type, window, window_box, window_box_left, Lisp_Type,
    },
    frame::LispFrameRef,
    globals::Qwindowp,
    lisp::{ExternalPtr, LispObject},
    vector::LispVectorlikeRef,
};

pub type Lisp_Window = window;

pub type LispWindowRef = ExternalPtr<Lisp_Window>;

impl LispWindowRef {
    /// A window of any sort, leaf or interior, is "valid" if its
    /// contents slot is non-nil.
    pub fn is_valid(self) -> bool {
        self.contents.is_not_nil()
    }

    // Equivalent to WINDOW_RIGHTMOST_P
    /// True if window W has no other windows to its right on its frame.
    pub fn is_rightmost(self) -> bool {
        self.right_pixel_edge() == self.get_frame().root_window().right_pixel_edge()
    }

    pub fn get_frame(self) -> LispFrameRef {
        self.frame.into()
    }

    #[cfg(not(feature = "window-system-webrender"))]
    pub fn is_menu_bar(self) -> bool {
        unimplemented!();
    }

    #[cfg(feature = "window-system-webrender")]
    pub fn is_menu_bar(self) -> bool {
        false
    }

    #[cfg(not(feature = "window-system-webrender"))]
    pub fn is_tool_bar(self) -> bool {
        unimplemented!();
    }

    #[cfg(feature = "window-system-webrender")]
    pub fn is_tool_bar(self) -> bool {
        false
    }

    pub fn top_edge_y(self) -> i32 {
        let mut y = self.top_pixel_edge();
        if !(self.is_menu_bar() || self.is_tool_bar()) {
            y += self.get_frame().internal_border_width();
        }
        y
    }

    /// The pixel value where the text (or left fringe) in window starts.
    pub fn left_pixel_edge(self) -> i32 {
        self.pixel_left
    }

    /// The top pixel edge at which the window starts.
    /// This includes a header line, if any.
    pub fn top_pixel_edge(self) -> i32 {
        self.pixel_top
    }

    /// Return the right pixel edge before which window W ends.
    /// This includes a right-hand scroll bar, if any.
    pub fn right_pixel_edge(self) -> i32 {
        self.left_pixel_edge() + self.pixel_width
    }

    /// Width of the bottom divider of the window
    pub fn right_divider_width(self) -> i32 {
        if self.is_rightmost() {
            0
        } else {
            self.get_frame().right_divider_width
        }
    }

    /// Convert window relative pixel Y to frame pixel coordinates.
    pub fn frame_pixel_y(self, y: i32) -> i32 {
        y + self.top_edge_y()
    }

    /// Convert window text relative pixel X to frame pixel coordinates.
    pub fn text_to_frame_pixel_x(mut self, x: i32) -> i32 {
        x + unsafe { window_box_left(self.as_mut(), TEXT_AREA) }
    }

    pub fn area_box(mut self, area: glyph_row_area::Type) -> (i32, i32, i32, i32) {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut width: i32 = 0;
        let mut height: i32 = 0;

        unsafe {
            window_box(self.as_mut(), area, &mut x, &mut y, &mut width, &mut height);
        }

        (x, y, width, height)
    }
}

impl LispObject {
    pub fn is_window(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_WINDOW))
    }

    pub fn as_window(self) -> Option<LispWindowRef> {
        self.into()
    }

    pub fn as_valid_window(self) -> Option<LispWindowRef> {
        self.as_window()
            .and_then(|w| if w.is_valid() { Some(w) } else { None })
    }
}

impl From<LispObject> for LispWindowRef {
    fn from(o: LispObject) -> Self {
        o.as_window().unwrap_or_else(|| wrong_type!(Qwindowp, o))
    }
}

impl From<LispWindowRef> for LispObject {
    fn from(w: LispWindowRef) -> Self {
        Self::tag_ptr(w, Lisp_Type::Lisp_Vectorlike)
    }
}

impl From<LispObject> for Option<LispWindowRef> {
    fn from(o: LispObject) -> Self {
        o.as_vectorlike().and_then(LispVectorlikeRef::as_window)
    }
}
