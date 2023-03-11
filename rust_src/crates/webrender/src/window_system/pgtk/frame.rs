use crate::color::pixel_to_color;
use crate::frame::LispFrameExt;
use crate::output::Output;
use crate::output::OutputRef;
use emacs::frame::LispFrameRef;
use gtk::glib::translate::FromGlibPtrNone;
use gtk::glib::translate::ToGlibPtr;
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
use webrender::api::units::DeviceIntSize;
use webrender::api::ColorF;

use crate::frame::LispFrameWindowSystemExt;

pub type FrameId = LispFrameRef;

impl LispFrameWindowSystemExt for LispFrameRef {
    fn output(&self) -> OutputRef {
        return OutputRef::new(unsafe { self.output_data.pgtk } as *mut Output);
    }

    fn cursor_color(&self) -> ColorF {
        let color = self.output().as_raw().cursor_color;
        pixel_to_color(color)
    }

    fn scale_factor(&self) -> f64 {
        let scale_factor = self.output().inner().scale_factor;
        if scale_factor != 0.0 {
            return scale_factor;
        }

        // fallback using parent frame
        if !self.parent_frame.is_nil() {
            let parent: LispFrameRef = self.parent_frame.into();
            return parent.scale_factor();
        }

        // fallback using widget
        if let Some(widget) = self.edit_widget() {
            return widget.scale_factor() as f64;
        }

        1.0
    }

    fn set_scale_factor(&mut self, scale_factor: f64) -> bool {
        if self.output().inner().scale_factor != scale_factor {
            self.output().inner().scale_factor = scale_factor;
            return true;
        }
        false
    }

    fn cursor_foreground_color(&self) -> ColorF {
        let color = self.output().as_raw().cursor_foreground_color;
        pixel_to_color(color)
    }

    fn window_handle(&self) -> Option<RawWindowHandle> {
        if !self.parent_frame.is_nil() {
            return None;
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
        if !self.parent_frame.is_nil() {
            return None;
        }
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

    fn unique_id(&self) -> FrameId {
        self.clone()
    }
}

pub trait LispFramePgtkExt {
    fn is_wayland(&self) -> bool;
    fn edit_widget(&self) -> Option<gtk::Widget>;
    fn fixed_widget(&self) -> Option<gtk::Fixed>;
    fn dynamic_resize(&self);
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

    fn dynamic_resize(&self) {
        let fixed = self.fixed_widget().expect("no fixed widget");
        fixed.connect_realize(move |widget| {
            let mut frame = fixed_wiget_to_frame(widget);
            frame.set_scale_factor(widget.scale_factor() as f64);
        });

        fixed.connect_size_allocate({
            move |widget, allocation| {
                let scale_factor = widget.scale_factor() as f64;
                let mut frame = fixed_wiget_to_frame(widget);
                let size =
                    DeviceIntSize::new(allocation.width() as i32, allocation.height() as i32);
                log::debug!("Gtk fixed size allocated {size:?} scale_factor: {scale_factor:?}");
                frame.handle_size_change(size, scale_factor);
            }
        });

        fixed.connect_scale_factor_notify(move |widget| {
            let mut frame = fixed_wiget_to_frame(widget);
            let scale_factor = widget.scale_factor() as f64;
            log::debug!("Gtk fixed scale_factor: {scale_factor:?}");
            frame.handle_scale_factor_change(widget.scale_factor().into());
        });
    }
}

fn fixed_wiget_to_frame(widget: &gtk::Fixed) -> LispFrameRef {
    let widget: *mut gtk_sys::GtkWidget = <gtk::Fixed as AsRef<gtk::Widget>>::as_ref(widget)
        .to_glib_none()
        .0;
    let frame = unsafe { emacs::bindings::pgtk_fixed_to_frame(widget) };
    frame.into()
}
