use crate::bindings::draw_phys_cursor_glyph;
use crate::bindings::get_phys_cursor_geometry;
use crate::bindings::get_phys_cursor_glyph;
use crate::bindings::glyph_row_area::TEXT_AREA;
use crate::bindings::glyph_row_area::{self};
use crate::bindings::pvec_type;
use crate::bindings::window;
use crate::bindings::window_box;
use crate::bindings::window_box_left;
use crate::bindings::Lisp_Type;
use crate::display_traits::DrawGlyphsFace;
use crate::display_traits::GlyphRef;
use crate::display_traits::GlyphRowRef;
use crate::frame::FrameRef;
use crate::globals::Qwindowp;
use crate::lisp::ExternalPtr;
use crate::lisp::LispObject;
use crate::vector::LispVectorlikeRef;

pub type Window = window;

pub type WindowRef = ExternalPtr<Window>;

impl WindowRef {
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

    pub fn get_frame(self) -> FrameRef {
        self.frame.into()
    }

    #[cfg(not(any(feature = "window-system-winit", feature = "window-system-pgtk")))]
    pub fn is_menu_bar(self) -> bool {
        unimplemented!();
    }

    #[cfg(all(feature = "window-system-pgtk", not(feature = "window-system-winit")))]
    pub fn is_menu_bar(self) -> bool {
        false
    }

    #[cfg(feature = "window-system-winit")]
    pub fn is_menu_bar(self) -> bool {
        false
    }

    #[cfg(not(any(feature = "window-system-winit", feature = "window-system-pgtk")))]
    pub fn is_tool_bar(self) -> bool {
        unimplemented!();
    }

    #[cfg(all(feature = "window-system-pgtk", not(feature = "window-system-winit")))]
    pub fn is_tool_bar(self) -> bool {
        false
    }

    #[cfg(feature = "window-system-winit")]
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

    pub fn area_box(mut self, area: impl Into<glyph_row_area::Type>) -> (i32, i32, i32, i32) {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut width: i32 = 0;
        let mut height: i32 = 0;

        unsafe {
            window_box(
                self.as_mut(),
                area.into(),
                &mut x,
                &mut y,
                &mut width,
                &mut height,
            );
        }

        (x, y, width, height)
    }

    pub fn phys_cursor_glyph(mut self) -> GlyphRef {
        unsafe { get_phys_cursor_glyph(self.as_mut()) }.into()
    }

    pub fn phys_cursor_geometry(mut self, mut row: GlyphRowRef) -> Option<(i32, i32, i32)> {
        let mut cursor_glyph = self.phys_cursor_glyph();

        if cursor_glyph.is_null() {
            return None;
        }

        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut height: i32 = 0;
        unsafe {
            get_phys_cursor_geometry(
                self.as_mut(),
                row.as_mut(),
                cursor_glyph.as_mut(),
                &mut x,
                &mut y,
                &mut height,
            )
        };
        Some((x, y, height))
    }

    pub fn draw_phys_cursor_glyph(mut self, mut row: GlyphRowRef) {
        unsafe {
            draw_phys_cursor_glyph(self.as_mut(), row.as_mut(), DrawGlyphsFace::Cursor.into())
        };
    }
}

impl LispObject {
    pub fn is_window(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_WINDOW))
    }

    pub fn as_window(self) -> Option<WindowRef> {
        self.into()
    }

    pub fn as_valid_window(self) -> Option<WindowRef> {
        self.as_window()
            .and_then(|w| if w.is_valid() { Some(w) } else { None })
    }
}

impl From<LispObject> for WindowRef {
    fn from(o: LispObject) -> Self {
        o.as_window().unwrap_or_else(|| wrong_type!(Qwindowp, o))
    }
}

impl From<WindowRef> for LispObject {
    fn from(w: WindowRef) -> Self {
        Self::tag_ptr(w, Lisp_Type::Lisp_Vectorlike)
    }
}

impl From<LispObject> for Option<WindowRef> {
    fn from(o: LispObject) -> Self {
        o.as_vectorlike().and_then(LispVectorlikeRef::as_window)
    }
}
