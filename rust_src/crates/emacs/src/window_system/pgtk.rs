use crate::frame::FrameRef;
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
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::ptr;

unsafe impl HasRawWindowHandle for FrameRef {
    fn raw_window_handle(&self) -> RawWindowHandle {
        if !self.parent_frame.is_nil() {
            unimplemented!("Pgtk child frame raw window handle!")
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
                let mut window_handle = WaylandWindowHandle::empty();
                window_handle.surface = surface;
                return RawWindowHandle::Wayland(window_handle);
            } else {
                let mut window_handle = XlibWindowHandle::empty();
                unsafe {
                    window_handle.window = gdk_x11_sys::gdk_x11_window_get_xid(window as *mut _);
                }
                return RawWindowHandle::Xlib(window_handle);
            }
        }
        panic!("Pgtk edit widget not avaiable");
    }
}

unsafe impl HasRawDisplayHandle for FrameRef {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        if !self.parent_frame.is_nil() {
            unimplemented!("Pgtk child frame raw window handle!")
        }
        if let Some(edit_widget) = self.edit_widget() {
            if self.is_wayland() {
                let mut display_handle = WaylandDisplayHandle::empty();
                display_handle.display = unsafe {
                    gdk_wayland_sys::gdk_wayland_display_get_wl_display(
                        edit_widget.display().as_ptr() as *mut _,
                    )
                };
                return RawDisplayHandle::Wayland(display_handle);
            } else {
                let mut display_handle = XlibDisplayHandle::empty();
                unsafe {
                    if let Ok(xlib) = x11_dl::xlib::Xlib::open() {
                        let display = (xlib.XOpenDisplay)(std::ptr::null());
                        display_handle.display = display as _;
                        display_handle.screen = (xlib.XDefaultScreen)(display) as _;
                    }
                }

                return RawDisplayHandle::Xlib(display_handle);
            }
        }
        panic!("Pgtk edit widget not avaiable");
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
