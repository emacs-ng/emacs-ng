use gleam::gl;
use glutin::{
    self,
    event_loop::{EventLoop, EventLoopProxy},
    window::Window,
    ContextWrapper, PossiblyCurrent,
};
use std::{
    ops::{Deref, DerefMut},
    ptr,
};
use webrender::{self, api::units::*, api::*, Renderer};

use lisp::remacs_sys::wr_output;

use super::display_info::DisplayInfoRef;
use super::font::FontRef;

pub struct Output {
    pub output: wr_output,
    pub font: FontRef,
    pub fontset: i32,

    pub window_context: ContextWrapper<PossiblyCurrent, Window>,
    pub renderer: Renderer,
    pub render_api: RenderApi,
    pub events_loop: EventLoop<()>,
}

impl Output {
    pub fn new() -> Self {
        let (api, renderer, window_context, events_loop) = Self::create_webrender_window();

        Self {
            output: wr_output::default(),
            font: FontRef::new(ptr::null_mut()),
            fontset: 0,
            window_context,
            renderer,
            render_api: api,
            events_loop,
        }
    }

    fn create_webrender_window() -> (
        RenderApi,
        Renderer,
        ContextWrapper<PossiblyCurrent, Window>,
        EventLoop<()>,
    ) {
        let events_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new();

        let window_context = glutin::ContextBuilder::new()
            .build_windowed(window_builder, &events_loop)
            .unwrap();

        let window_context = unsafe { window_context.make_current() }.unwrap();

        let gl = match window_context.get_api() {
            glutin::Api::OpenGl => unsafe {
                gl::GlFns::load_with(|symbol| window_context.get_proc_address(symbol) as *const _)
            },
            glutin::Api::OpenGlEs => unsafe {
                gl::GlesFns::load_with(|symbol| window_context.get_proc_address(symbol) as *const _)
            },
            glutin::Api::WebGl => unimplemented!(),
        };

        let gl_window = window_context.window();

        let device_pixel_ratio = gl_window.scale_factor() as f32;

        let device_size = {
            let size = gl_window.inner_size();
            DeviceIntSize::new(size.width as i32, size.height as i32)
        };

        let webrender_opts = webrender::RendererOptions {
            device_pixel_ratio,
            clear_color: Some(ColorF::WHITE),
            ..webrender::RendererOptions::default()
        };

        let notifier = Box::new(Notifier::new(events_loop.create_proxy()));
        let (renderer, sender) =
            webrender::Renderer::new(gl.clone(), notifier, webrender_opts, None, device_size)
                .unwrap();

        let api = sender.create_api();

        (api, renderer, window_context, events_loop)
    }

    pub fn show_window(&self) {
        self.window_context.window().set_visible(true);
    }
    pub fn hide_window(&self) {
        self.window_context.window().set_visible(false);
    }

    pub fn set_display_info(&mut self, mut dpyinfo: DisplayInfoRef) {
        self.output.display_info = dpyinfo.get_raw().as_mut();
    }

    pub fn display_info(&self) -> DisplayInfoRef {
        DisplayInfoRef::new(self.output.display_info as *mut _)
    }
}

#[repr(transparent)]
pub struct OutputRef(*mut Output);

impl Copy for OutputRef {}

// Derive fails for this type so do it manually
impl Clone for OutputRef {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl OutputRef {
    pub const fn new(p: *mut Output) -> Self {
        Self(p)
    }

    pub fn as_mut(&mut self) -> *mut wr_output {
        self.0 as *mut wr_output
    }
}

impl Deref for OutputRef {
    type Target = Output;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for OutputRef {
    fn deref_mut(&mut self) -> &mut Output {
        unsafe { &mut *self.0 }
    }
}

impl From<*mut wr_output> for OutputRef {
    fn from(o: *mut wr_output) -> Self {
        Self::new(o as *mut Output)
    }
}

struct Notifier {
    event_loop_proxy: EventLoopProxy<()>,
}

impl Notifier {
    fn new(event_loop_proxy: EventLoopProxy<()>) -> Notifier {
        Notifier { event_loop_proxy }
    }
}

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<dyn RenderNotifier> {
        Box::new(Notifier {
            event_loop_proxy: self.event_loop_proxy.clone(),
        })
    }

    fn wake_up(&self) {
        let _ = self.event_loop_proxy.send_event(());
    }

    fn new_frame_ready(
        &self,
        _: DocumentId,
        _scrolled: bool,
        _composite_needed: bool,
        _render_time: Option<u64>,
    ) {
        self.wake_up();
    }
}
