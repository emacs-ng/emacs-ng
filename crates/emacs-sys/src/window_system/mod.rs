#[cfg(have_pgtk)]
mod pgtk;
#[cfg(have_pgtk)]
pub use pgtk::*;
#[cfg(have_winit)]
mod winit;
#[cfg(have_winit)]
pub use winit::*;

use crate::frame::FrameRef;
use webrender_api::units::DeviceIntSize;
use webrender_api::units::LayoutSize;

impl FrameRef {
    pub fn logical_size(&self) -> LayoutSize {
        LayoutSize::new(self.pixel_width as f32, self.pixel_height as f32)
    }

    pub fn physical_size(&self) -> DeviceIntSize {
        let size = self.logical_size() * euclid::Scale::new(self.scale_factor() as f32);
        size.to_i32()
    }
}

use raw_window_handle::RawDisplayHandle;

pub fn display_descriptor(display_handle: RawDisplayHandle) -> std::os::fd::RawFd {
    #[cfg(free_unix)]
    use raw_window_handle::WaylandDisplayHandle;
    #[cfg(x11_platform)]
    use raw_window_handle::XcbDisplayHandle;
    #[cfg(x11_platform)]
    use raw_window_handle::XlibDisplayHandle;
    #[cfg(free_unix)]
    use wayland_sys::client::wayland_client_handle;
    #[cfg(free_unix)]
    use wayland_sys::client::wl_display;

    match display_handle {
        #[cfg(free_unix)]
        RawDisplayHandle::Wayland(WaylandDisplayHandle { display, .. }) => {
            log::trace!("wayland display {display:?}");
            let fd = unsafe {
                (wayland_client_handle().wl_display_get_fd)(display.as_ptr() as *mut wl_display)
            };
            log::trace!("wayland display fd {fd:?}");
            fd
        }
        #[cfg(x11_platform)]
        RawDisplayHandle::Xlib(XlibDisplayHandle { display, .. }) => {
            log::trace!("xlib display {display:?}");
            let fd = unsafe {
                x11::xlib::XConnectionNumber(display.as_ptr() as *mut x11::xlib::Display)
            };
            log::trace!("xlib display fd {fd:?}");
            fd
        }
        #[cfg(x11_platform)]
        RawDisplayHandle::Xcb(XcbDisplayHandle { .. }) => {
            unimplemented!("display descriptor for xcb")
        } // How does this differs from xlib?
        _ => unimplemented!("display descriptor"),
    }
}
