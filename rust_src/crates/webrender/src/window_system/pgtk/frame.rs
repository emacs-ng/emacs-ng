use crate::color::pixel_to_color;
use crate::output::Output;
use crate::output::OutputRef;
use emacs::bindings::xg_frame_resized;
use emacs::frame::LispFrameRef;
use gtk::glib::translate::FromGlibPtrNone;
use gtk::prelude::Cast;
use gtk::prelude::DisplayExtManual;
use gtk::prelude::ObjectType;
use gtk::prelude::WidgetExt;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use raw_window_handle::WaylandDisplayHandle;
use raw_window_handle::WaylandWindowHandle;
use raw_window_handle::XlibDisplayHandle;
use raw_window_handle::XlibWindowHandle;
use std::ptr;
use webrender::api::ColorF;

use crate::frame::LispFrameWindowSystemExt;

pub type FrameId = u64;

impl LispFrameWindowSystemExt for LispFrameRef {
    fn output(&self) -> OutputRef {
        return OutputRef::new(unsafe { self.output_data.pgtk } as *mut Output);
    }

    fn cursor_color(&self) -> ColorF {
        let color = self.output().as_raw().cursor_color;
        pixel_to_color(color)
    }

    // PGTK compute glyphs using unscale font etc
    // Then scale rediplay output all together?
    // While winit/tao compute scaled glyphs
    // then directly draw rediplay output with on scale needed
    fn scale_factor(&self) -> f64 {
        let scale_factor = unsafe { (*self.output_data.pgtk).watched_scale_factor };
        if scale_factor != 0.0 {
            return scale_factor;
        }
        1.0
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        unsafe { (*self.output_data.pgtk).watched_scale_factor = scale_factor };
        if let Some(widget) = self.edit_widget() {
            unsafe {
                xg_frame_resized(
                    self.as_mut(),
                    (widget.allocated_width() as f64 * scale_factor).round() as i32,
                    (widget.allocated_height() as f64 * scale_factor).round() as i32,
                )
            };
        }
    }

    fn cursor_foreground_color(&self) -> ColorF {
        let color = self.output().as_raw().cursor_foreground_color;
        pixel_to_color(color)
    }

    fn window_handle(&self) -> Option<RawWindowHandle> {
        if let Some(edit_widget) = self.edit_widget() {
            let window = unsafe { gtk_sys::gtk_widget_get_window(edit_widget.as_ptr()) };
            if self.is_wayland() {
                let surface = unsafe {
                    gdk_wayland_sys::gdk_wayland_window_get_wl_surface(
                        window as *mut _ as *mut gdk_wayland_sys::GdkWaylandWindow,
                    )
                };
                log::debug!("surface: {:?}", surface);
                let mut window_handle = WaylandWindowHandle::empty();
                window_handle.surface = surface;
                return Some(RawWindowHandle::Wayland(window_handle));
            } else {
                let mut window_handle = XlibWindowHandle::empty();
                unsafe {
                    window_handle.window = gdk_x11_sys::gdk_x11_window_get_xid(window as *mut _);
                }
                return Some(RawWindowHandle::Xlib(window_handle));
            }
        }
        return None;
    }

    fn display_handle(&self) -> Option<RawDisplayHandle> {
        if let Some(edit_widget) = self.edit_widget() {
            if self.is_wayland() {
                let mut display_handle = WaylandDisplayHandle::empty();
                display_handle.display = unsafe {
                    gdk_wayland_sys::gdk_wayland_display_get_wl_display(
                        edit_widget.display().as_ptr() as *mut _,
                    )
                };
                return Some(RawDisplayHandle::Wayland(display_handle));
            } else {
                let mut display_handle = XlibDisplayHandle::empty();
                unsafe {
                    if let Ok(xlib) = x11_dl::xlib::Xlib::open() {
                        let display = (xlib.XOpenDisplay)(std::ptr::null());
                        display_handle.display = display as _;
                        display_handle.screen = (xlib.XDefaultScreen)(display) as _;
                    }
                }

                return Some(RawDisplayHandle::Xlib(display_handle));
            }
        }
        None
    }

    fn unique_id(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        self.window_handle().hash(&mut hasher);
        hasher.finish()
    }
}

pub trait LispFramePgtkExt {
    fn is_wayland(&self) -> bool;
    fn edit_widget(&self) -> Option<gtk::Widget>;
    fn fixed_widget(&self) -> Option<gtk::Fixed>;
}

impl LispFramePgtkExt for LispFrameRef {
    fn edit_widget(&self) -> Option<gtk::Widget> {
        let mut output = self.output();
        let widget = output.as_raw().edit_widget;
        if widget != ptr::null_mut() {
            return Some(unsafe { gtk::Widget::from_glib_none(widget) });
        }
        None
    }

    fn fixed_widget(&self) -> Option<gtk::Fixed> {
        match self.edit_widget() {
            Some(widget) => Some(unsafe { widget.unsafe_cast() }),
            None => None,
        }
    }

    fn is_wayland(&self) -> bool {
        match self.edit_widget() {
            Some(widget) => widget.display().backend().is_wayland(),
            None => false,
        }
    }
}
