use crate::gl::context_impl::ContextImpl;
use emacs::frame::FrameRef;
use gleam::gl::Gl;
use std::rc::Rc;
use webrender::{self, api::units::*};

pub type GLContext = ContextImpl;

pub trait GLContextTrait {
    fn build(frame: &FrameRef) -> Self;

    fn bind_framebuffer(&mut self, gl: &mut Rc<dyn Gl>);

    fn swap_buffers(&self);

    fn load_gl(&self) -> Rc<dyn Gl>;

    fn resize(&self, size: &DeviceIntSize);

    fn ensure_is_current(&mut self);
}
