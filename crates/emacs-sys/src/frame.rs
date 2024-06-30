//! Generic frame functions.
#[cfg(use_webrender)]
use webrender_api::ColorF;

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
#[cfg(use_webrender)]
use crate::color::pixel_to_color;
use crate::display_traits::FaceRef;
#[cfg(have_window_system)]
use crate::display_traits::FrameParam;
#[cfg(have_window_system)]
use crate::display_traits::ImageCacheRef;
use crate::globals::*;
use crate::lisp::ExternalPtr;
use crate::lisp::LispObject;
use crate::list::LispConsCircularChecks;
use crate::list::LispConsEndChecks;
use crate::vector::LispVectorlikeRef;
use crate::window::WindowRef;

#[cfg(have_window_system)]
use crate::bindings::globals;
#[cfg(have_window_system)]
use crate::bindings::gui_default_parameter;
#[cfg(have_window_system)]
use crate::bindings::make_frame;
#[cfg(have_window_system)]
use crate::bindings::make_frame_without_minibuffer;
#[cfg(have_window_system)]
use crate::bindings::make_minibuffer_frame;
#[cfg(have_window_system)]
use crate::bindings::specbind;
#[cfg(have_window_system)]
use crate::bindings::vertical_scroll_bar_type;
#[cfg(have_window_system)]
use crate::bindings::Fcons;
#[cfg(have_window_system)]
use crate::display_info::DisplayInfoRef;
#[cfg(have_window_system)]
use crate::font::FontRef;
#[cfg(have_window_system)]
use crate::globals::Qnone;
#[cfg(have_window_system)]
use crate::globals::Qonly;
#[cfg(have_window_system)]
use crate::globals::Qx_resource_name;
#[cfg(have_window_system)]
use crate::output::OutputRef;
#[cfg(have_window_system)]
use crate::terminal::TerminalRef;

pub type Frame = frame;

/// FrameRef is a reference to the Frame
/// However a reference is guaranteed to point to an existing frame
/// therefore no NULL checks are needed while using it
#[allow(dead_code)]
pub type FrameRef = ExternalPtr<Frame>;

impl FrameRef {
    pub fn root_window(self) -> WindowRef {
        self.root_window.into()
    }

    pub fn minibufffer_window(self) -> WindowRef {
        self.minibuffer_window.into()
    }

    pub fn is_live(self) -> bool {
        !self.terminal.is_null()
    }

    pub fn child_frame_border_width(self) -> i32 {
        unsafe { frame_dimension(self.child_frame_border_width) }
    }

    // Pixel-width of internal border lines.
    #[cfg(not(have_window_system))]
    pub fn internal_border_width(self) -> i32 {
        unsafe { frame_dimension(self.internal_border_width) }
    }

    /* Pixel-width of internal border.  Uses child_frame_border_width for
    child frames if possible, and falls back on internal_border_width
    otherwise.  */
    #[cfg(have_window_system)]
    pub fn internal_border_width(self) -> i32 {
        match self.parent_frame() {
            Some(_) if self.child_frame_border_width() >= 0 => self.child_frame_border_width(),
            _ => unsafe { frame_dimension(self.internal_border_width) },
        }
    }

    pub fn set_internal_border_width(mut self, border: i32) {
        self.internal_border_width = border;
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

    pub fn tool_bar_bottom_height(&self) -> i32 {
        match self.tool_bar_position {
            Qbottom => self.tool_bar_lines,
            _ => 0,
        }
    }

    pub fn top_margin_height(self) -> i32 {
        self.menu_bar_height + self.tool_bar_height + self.tool_bar_lines
    }

    pub fn bottom_margin_height(self) -> i32 {
        self.tool_bar_bottom_height()
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

    pub fn face_from_id(self, id: impl Into<face_id>) -> Option<FaceRef> {
        let id: face_id = id.into();
        let cache = self.face_cache;

        let faces_map: &[*mut face] =
            unsafe { std::slice::from_raw_parts_mut((*cache).faces_by_id, (*cache).used as usize) };

        faces_map.get(id as usize).copied().map(|f| FaceRef::new(f))
    }

    #[cfg(use_webrender)]
    pub fn fg_color(&self) -> ColorF {
        pixel_to_color(self.foreground_pixel)
    }

    pub fn param(self, prop: impl Into<LispObject>) -> LispObject {
        match unsafe { Fassq(prop.into(), self.param_alist) }.as_cons() {
            Some(cons) => cons.cdr(),
            None => Qnil,
        }
    }

    pub fn is_minibuf_only(self) -> bool {
        self.root_window.eq(self.minibuffer_window)
    }

    pub fn parent_frame(self) -> Option<FrameRef> {
        if cfg!(window_system) {
            if self.parent_frame.is_not_nil() {
                Some(FrameRef::from(self.parent_frame))
            } else {
                None
            }
        } else {
            None
        }
    }

    #[cfg(have_window_system)]
    pub fn gui_default_parameter(
        mut self,
        params: LispObject,
        param: impl Into<FrameParam>,
        default: impl Into<LispObject>,
    ) -> LispObject {
        let param: FrameParam = param.into();

        let params_fallback = || {
            let lparam: LispObject = param.into();
            if unsafe { Fassq(lparam, params) }.is_nil() {
                let value = self.display_info().gui_arg(params, param);
                if !value.base_eq(Qunbound) {
                    return unsafe { Fcons(Fcons(param.into(), value), params) };
                }
                return params;
            }
            return params;
        };

        let params = match param {
            FrameParam::InternalBorderWidth | FrameParam::ChildFrameBorderWidth => {
                params_fallback()
            }
            _ => params,
        };

        let res_type = param.resource_type();
        let (xprop, xclass) = param.x_resource();

        unsafe {
            gui_default_parameter(
                self.as_mut(),
                params,
                param.into(),
                default.into(),
                xprop.as_ptr(),
                xclass.as_ptr(),
                res_type.into(),
            )
        }
    }

    #[cfg(have_window_system)]
    pub fn gui_default_parameter_no_x_resource(
        mut self,
        alist: LispObject,
        param: impl Into<FrameParam>,
        default: impl Into<LispObject>,
    ) -> LispObject {
        use std::ffi::CString;

        let param: FrameParam = param.into();
        let res_type = param.resource_type();
        let str = CString::new("").unwrap();
        unsafe {
            gui_default_parameter(
                self.as_mut(),
                alist,
                param.into(),
                default.into(),
                str.as_ptr(),
                str.as_ptr(),
                res_type.into(),
            )
        }
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

    pub fn set_name(mut self, name: LispObject) {
        unsafe { crate::bindings::fset_name(self.as_mut(), name) };
    }

    pub fn set_parent(mut self, parent: LispObject) {
        unsafe { crate::bindings::fset_parent_frame(self.as_mut(), parent) };
    }

    pub fn set_icon_name(mut self, icon_name: LispObject) {
        unsafe { crate::bindings::fset_icon_name(self.as_mut(), icon_name) };
    }

    pub fn set_undecorated_(mut self, undecorated: bool) {
        if cfg!(have_window_system) {
            self.set_undecorated(undecorated);
        }
    }

    pub fn set_override_redirect_(mut self, override_redirect: bool) {
        if cfg!(have_window_system) {
            self.set_override_redirect(override_redirect);
        }
    }

    pub fn store_param(mut self, prop: impl Into<LispObject>, val: impl Into<LispObject>) {
        unsafe { store_frame_param(self.as_mut(), prop.into(), val.into()) };
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
        self.output().font()
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

    pub fn is_current_window_system(&self) -> bool {
        if cfg!(have_winit) {
            return self.output_method() == crate::bindings::output_method::output_winit;
        } else if cfg!(have_pgtk) {
            return self.output_method() == crate::bindings::output_method::output_pgtk;
        }
        false
    }

    #[cfg(have_window_system)]
    pub fn image_cache(self) -> ImageCacheRef {
        ImageCacheRef::new(self.image_cache as *mut _)
    }

    #[cfg(have_window_system)]
    pub fn build(mut dpyinfo: DisplayInfoRef, params: LispObject) -> Self {
        let name = dpyinfo.gui_arg(params, FrameParam::Name);

        if !name.is_string() && !name.eq(Qunbound) && !name.is_nil() {
            error!("Invalid frame name--not a string or nil");
        }

        if name.is_string() {
            unsafe {
                globals.Vx_resource_name = name;
            }
        }

        /* Check if parent window is specified. Return early if parent_id is not number
        The validation is inside gui_arg func call*/
        let parent_id = dpyinfo.gui_arg(params, FrameParam::ParentId);

        let terminal = dpyinfo.terminal();

        if terminal.name == std::ptr::null_mut() {
            error!("Terminal is not live, can't create new frames on it");
        }

        let kb = terminal.kboard;

        let tem = dpyinfo.gui_arg(params, FrameParam::Minibuffer);
        let display_arg = dpyinfo.gui_arg(params, FrameParam::Display);

        let f = if tem.eq(Qnone) || tem.is_nil() {
            unsafe { make_frame_without_minibuffer(Qnil, kb, display_arg) }
        } else if tem.eq(Qonly) {
            unsafe { make_minibuffer_frame() }
        } else if tem.is_window() {
            unsafe { make_frame_without_minibuffer(tem, kb, display_arg) }
        } else {
            unsafe { make_frame(true) }
        };

        let mut f = Self::new(f);
        /* Set the name; the functions to which we pass f expect the name to
        be set.  */
        if name.base_eq(Qunbound) || name.is_nil() {
            // pgtk using dpyinfo->x_id_name here
            let default_name = "default frame name";
            let default_name: LispObject = default_name.to_string().into();
            f.set_name(default_name);
            f.set_explicit_name(false);
        } else {
            f.set_name(name);
            f.set_explicit_name(true);
            unsafe { specbind(Qx_resource_name, name) };
        }

        f.terminal = dpyinfo.terminal;
        f.set_icon_name(dpyinfo.gui_arg(params, FrameParam::IconName));

        let mut process_bool_arg = |param: FrameParam| {
            let value = dpyinfo.gui_arg(params, param);
            let value = value.is_not_nil() && !value.base_eq(Qunbound);
            if param == FrameParam::Undecorated {
                f.set_undecorated_(value);
            } else if param == FrameParam::OverrideRedirect {
                f.set_override_redirect_(value);
            }
            let value = if value { Qt } else { Qnil };
            f.store_param(param, value);
        };

        process_bool_arg(FrameParam::Undecorated);
        process_bool_arg(FrameParam::OverrideRedirect);

        let mut process_num_arg = |param: FrameParam| {
            let value = dpyinfo.gui_arg(params, param);
            if value.is_fixnum() {
                f.store_param(param, value);
            }
        };

        process_num_arg(FrameParam::MinWidth);
        process_num_arg(FrameParam::MinHeight);

        /* Accept parent-frame if parent-id was not specified.  */
        let parent_frame = if parent_id.is_nil() {
            dpyinfo.gui_arg(params, FrameParam::ParentFrame)
        } else {
            Qnil
        };
        f.set_parent(parent_frame);
        f.store_param(FrameParam::ParentFrame, parent_frame);

        if cfg!(have_winit) && parent_frame.is_not_nil() {
            error!("Winit currently doesn't support parent-frame parameter");
        }

        let unsplittable =
            f.is_minibuf_only() || dpyinfo.gui_arg(params, FrameParam::Unsplittable).is_t();
        f.set_no_split(unsplittable);

        f
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
