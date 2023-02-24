use crate::gl::context_impl::ContextImpl;
use gleam::gl::Gl;
use raw_window_handle::HasRawDisplayHandle;
use raw_window_handle::HasRawWindowHandle;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use std::rc::Rc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub type Context = ContextImpl;

pub trait ContextTrait {
    fn build(
        size: PhysicalSize<u32>,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Self;

    fn bind_framebuffer(&mut self, gl: &mut Rc<dyn Gl>);

    fn swap_buffers(&self);

    fn load_gl(&self) -> Rc<dyn Gl>;

    fn resize(&self, size: &PhysicalSize<u32>);

    fn ensure_is_current(&mut self);
}

impl From<&Window> for Context {
    fn from(window: &Window) -> Self {
        let display_handle = window.raw_display_handle();
        let window_handle = window.raw_window_handle();
        let size = window.inner_size();
        Context::build(size, display_handle, window_handle)
    }
}
