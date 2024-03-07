use crate::frame::FrameRef;
use crate::gfx::context_impl::ContextImpl;
use gleam::gl::Gl;
use std::rc::Rc;
use webrender_api::units::DeviceIntSize;

pub type GLContext = ContextImpl;

pub trait GLContextTrait {
    fn build(frame: &FrameRef) -> Self;

    fn bind_framebuffer(&mut self, gl: &mut Rc<dyn Gl>);

    fn swap_buffers(&self);

    fn load_gl(&self) -> Rc<dyn Gl>;

    fn resize(&self, size: &DeviceIntSize);

    fn ensure_is_current(&mut self);
}

impl FrameRef {
    pub fn create_gl_context(&self) -> GLContext {
        GLContext::build(self)
    }
}
