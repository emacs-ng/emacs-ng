use crate::color::pixel_to_color;
use crate::frame::FrameRef;
use core::ptr::NonNull;
use gtk::glib::translate::FromGlibPtrNone;
use gtk::prelude::Cast;
use gtk::prelude::DisplayExtManual;
use gtk::prelude::ObjectType;
use gtk::prelude::WidgetExt;
use raw_window_handle::DisplayHandle;
use raw_window_handle::HandleError;
use raw_window_handle::HasDisplayHandle;
use raw_window_handle::HasWindowHandle;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use raw_window_handle::WaylandDisplayHandle;
use raw_window_handle::WaylandWindowHandle;
use raw_window_handle::WindowHandle;
use raw_window_handle::XlibDisplayHandle;
use raw_window_handle::XlibWindowHandle;
use std::ptr;
use webrender_api::ColorF;

impl FrameRef {
    pub fn cursor_color(&self) -> ColorF {
        let color = self.output().cursor_color;
        pixel_to_color(color)
    }

    pub fn scale_factor(&self) -> f64 {
        if !self.parent_frame.is_nil() {
            let parent: FrameRef = self.parent_frame.into();
            return parent.scale_factor();
        }

        // fallback using widget
        if let Some(widget) = self.edit_widget() {
            return widget.scale_factor() as f64;
        }

        1.0
    }

    pub fn cursor_foreground_color(&self) -> ColorF {
        let color = self.output().cursor_foreground_color;
        pixel_to_color(color)
    }
}

impl HasWindowHandle for FrameRef {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        if !self.parent_frame.is_nil() {
            message!("Pgtk child frame raw window handle!");
            return Err(HandleError::Unavailable);
        }
        if let Some(edit_widget) = self.edit_widget() {
            let window = unsafe { gtk_sys::gtk_widget_get_window(edit_widget.as_ptr()) };
            if self.is_wayland() {
                let surface = unsafe {
                    gdk_wayland_sys::gdk_wayland_window_get_wl_surface(
                        window as *mut _ as *mut gdk_wayland_sys::GdkWaylandWindow,
                    )
                };
                log::debug!("surface: {:?}", surface);
                match NonNull::new(surface) {
                    Some(surface) => {
                        let handle = WaylandWindowHandle::new(surface);
                        return Ok(unsafe {
                            WindowHandle::borrow_raw(RawWindowHandle::Wayland(handle))
                        });
                    }
                    None => return Err(HandleError::Unavailable),
                };
            } else {
                let window = unsafe { gdk_x11_sys::gdk_x11_window_get_xid(window as *mut _) };
                let mut handle = XlibWindowHandle::new(window);
                // Optionally set the visual ID.
                handle.visual_id = 0;
                return Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Xlib(handle)) });
            }
        }
        return Err(HandleError::Unavailable);
    }
}

impl HasDisplayHandle for FrameRef {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        if !self.parent_frame.is_nil() {
            message!("Pgtk child frame raw window handle!");
            return Err(HandleError::Unavailable);
        }
        if let Some(edit_widget) = self.edit_widget() {
            if self.is_wayland() {
                let display = unsafe {
                    gdk_wayland_sys::gdk_wayland_display_get_wl_display(
                        edit_widget.display().as_ptr() as *mut _,
                    )
                };
                match NonNull::new(display) {
                    Some(display) => {
                        let handle = WaylandDisplayHandle::new(display);
                        return Ok(unsafe {
                            DisplayHandle::borrow_raw(RawDisplayHandle::Wayland(handle))
                        });
                    }
                    None => return Err(HandleError::Unavailable),
                };
            } else {
                let handle = unsafe {
                    if let Ok(xlib) = x11_dl::xlib::Xlib::open() {
                        let display = (xlib.XOpenDisplay)(std::ptr::null());
                        XlibDisplayHandle::new(
                            NonNull::new(display as _),
                            (xlib.XDefaultScreen)(display) as _,
                        )
                    } else {
                        return Err(HandleError::Unavailable);
                    }
                };
                return Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Xlib(handle)) });
            }
        }
        return Err(HandleError::Unavailable);
    }
}

pub trait FrameExtPgtk {
    fn is_wayland(&self) -> bool;
    fn edit_widget(&self) -> Option<gtk::Widget>;
    fn fixed_widget(&self) -> Option<gtk::Fixed>;
}

impl FrameExtPgtk for FrameRef {
    fn edit_widget(&self) -> Option<gtk::Widget> {
        let output = self.output();
        let widget = output.edit_widget;
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
