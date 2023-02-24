use crate::gl::context_impl::ContextImpl;
use gleam::gl::Gl;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use std::rc::Rc;
use webrender::{self, api::units::*};

pub type GLContext = ContextImpl;

pub trait GLContextTrait {
    fn build(
        size: DeviceIntSize,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Self;

    fn bind_framebuffer(&mut self, gl: &mut Rc<dyn Gl>);

    fn swap_buffers(&self);

    fn load_gl(&self) -> Rc<dyn Gl>;

    fn resize(&self, size: &DeviceIntSize);

    fn ensure_is_current(&mut self);
}
