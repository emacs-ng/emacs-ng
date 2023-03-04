use crate::color::pixel_to_color;
use crate::output::Output;
use crate::output::OutputRef;
use emacs::frame::LispFrameRef;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
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

    fn cursor_foreground_color(&self) -> ColorF {
        let color = self.output().as_raw().cursor_foreground_color;
        pixel_to_color(color)
    }

    fn window_handle(&self) -> Option<RawWindowHandle> {
        use raw_window_handle::WaylandWindowHandle;
        use std::ptr;
        let mut output = self.output();
        let widget = output.as_raw().edit_widget;
        if widget != ptr::null_mut() {
            let gwin = unsafe { gtk_sys::gtk_widget_get_window(widget) };
            let surface = unsafe {
                gdk_wayland_sys::gdk_wayland_window_get_wl_surface(
                    gwin as *mut _ as *mut gdk_wayland_sys::GdkWaylandWindow,
                )
            };
            log::debug!("surface: {:?}", surface);
            let mut window_handle = WaylandWindowHandle::empty();
            window_handle.surface = surface;
            return Some(RawWindowHandle::Wayland(window_handle));
        }
        return None;
    }

    fn display_handle(&self) -> Option<RawDisplayHandle> {
        use raw_window_handle::WaylandDisplayHandle;

        let display = unsafe {
            self.output()
                .display_info()
                .get_raw()
                .__bindgen_anon_1
                .display
        };
        let wl_display = unsafe {
            gdk_wayland_sys::gdk_wayland_display_get_wl_display(
                display as *mut _ as *mut gdk_wayland_sys::GdkWaylandDisplay,
            )
        };
        let mut display_handle = WaylandDisplayHandle::empty();
        display_handle.display = wl_display;
        return Some(RawDisplayHandle::Wayland(display_handle));
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
