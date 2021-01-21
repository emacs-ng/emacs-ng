use std::{rc::Rc, sync::mpsc::sync_channel, thread::JoinHandle};

use font_kit::handle::Handle as FontHandle;
use gleam::gl::{self, Gl};
use glutin::{
    self,
    dpi::{LogicalSize, PhysicalPosition},
    event::Event,
    event_loop::{ControlFlow, EventLoopProxy},
    monitor::MonitorHandle,
    window::Window,
    ContextWrapper, PossiblyCurrent,
};
use std::{
    ops::{Deref, DerefMut},
    ptr,
};

#[cfg(unix)]
use glutin::platform::unix::EventLoopExtUnix;
#[cfg(windows)]
use glutin::platform::windows::EventLoopExtUnix;

use webrender::{self, api::units::*, api::*};

use lisp::remacs_sys::wr_output;

use super::display_info::DisplayInfoRef;
use super::font::FontRef;

pub struct Output {
    // Extend `wr_output` struct defined in `wrterm.h`
    pub output: wr_output,

    pub font: FontRef,
    pub fontset: i32,

    pub render_api: RenderApi,
    pub loop_thread: JoinHandle<()>,
    pub document_id: DocumentId,

    pub display_list_builder: Option<DisplayListBuilder>,

    pub background_color: ColorF,

    window: Window,

    color_bits: u8,
}

impl Output {
    pub fn new() -> Self {
        let (api, document_id, window, loop_thread, color_bits) = Self::create_webrender_window();

        Self {
            output: wr_output::default(),
            font: FontRef::new(ptr::null_mut()),
            fontset: 0,
            render_api: api,
            loop_thread,
            document_id,
            display_list_builder: None,
            background_color: ColorF::WHITE,
            window,
            color_bits,
        }
    }

    fn create_webrender_window() -> (RenderApi, DocumentId, Window, JoinHandle<()>, u8) {
        let (webrender_tx, webrender_rx) = sync_channel(1);

        let window_loop_thread = std::thread::spawn(move || {
            let events_loop = glutin::event_loop::EventLoop::new_any_thread();
            let window_builder = glutin::window::WindowBuilder::new().with_maximized(true);

            let window_context = glutin::ContextBuilder::new()
                .build_windowed(window_builder, &events_loop)
                .unwrap();

            let current_context = unsafe { window_context.make_current() }.unwrap();
            let (current_context, window) = unsafe { current_context.split() };

            let gl = Self::get_gl_api(&current_context);

            gl.clear_color(1.0, 1.0, 1.0, 1.0);
            gl.clear(self::gl::COLOR_BUFFER_BIT);
            gl.flush();
            current_context.swap_buffers().ok();

            let events_loop_proxy = events_loop.create_proxy();

            let device_pixel_ratio = window.scale_factor() as f32;

            let device_size = {
                let size = window.inner_size();
                DeviceIntSize::new(size.width as i32, size.height as i32)
            };

            let webrender_opts = webrender::RendererOptions {
                device_pixel_ratio,
                clear_color: None,
                ..webrender::RendererOptions::default()
            };

            let notifier = Box::new(Notifier::new(events_loop_proxy));

            let (mut renderer, sender) =
                webrender::Renderer::new(gl.clone(), notifier, webrender_opts, None, device_size)
                    .unwrap();

            let api = sender.create_api();

            let document_id = api.add_document(device_size, 0 /* layer */);

            let color_bits = current_context.get_pixel_format().color_bits;

            webrender_tx
                .send((sender.create_api(), window, document_id, color_bits))
                .unwrap();

            events_loop.run(move |e, _, control_flow| {
                *control_flow = ControlFlow::Wait;

                match e {
                    Event::UserEvent(_) => {
                        renderer.update();
                        renderer.render(device_size).unwrap();
                        let _ = renderer.flush_pipeline_info();
                        current_context.swap_buffers().ok();
                    }
                    _ => {}
                };
            })
        });

        let (api, window, document_id, color_bits) = webrender_rx.recv().unwrap();

        let pipeline_id = PipelineId(0, 0);

        let mut txn = Transaction::new();
        txn.set_root_pipeline(pipeline_id);
        api.send_transaction(document_id, txn);

        (api, document_id, window, window_loop_thread, color_bits)
    }

    fn get_gl_api(window_context: &ContextWrapper<PossiblyCurrent, ()>) -> Rc<dyn Gl> {
        match window_context.get_api() {
            glutin::Api::OpenGl => unsafe {
                gl::GlFns::load_with(|symbol| window_context.get_proc_address(symbol) as *const _)
            },
            glutin::Api::OpenGlEs => unsafe {
                gl::GlesFns::load_with(|symbol| window_context.get_proc_address(symbol) as *const _)
            },
            glutin::Api::WebGl => unimplemented!(),
        }
    }

    fn get_size(window: &Window) -> (DeviceIntSize, LayoutSize) {
        let device_pixel_ratio = window.scale_factor() as f32;

        let physical_size = window.inner_size();

        let logical_size = physical_size.to_logical::<f32>(device_pixel_ratio as f64);

        let layout_size = LayoutSize::new(logical_size.width as f32, logical_size.height as f32);
        let device_size =
            DeviceIntSize::new(physical_size.width as i32, physical_size.height as i32);

        (device_size, layout_size)
    }

    pub fn show_window(&self) {
        self.window.set_visible(true);
    }
    pub fn hide_window(&self) {
        self.window.set_visible(false);
    }

    pub fn set_display_info(&mut self, mut dpyinfo: DisplayInfoRef) {
        self.output.display_info = dpyinfo.get_raw().as_mut();
    }

    pub fn display_info(&self) -> DisplayInfoRef {
        DisplayInfoRef::new(self.output.display_info as *mut _)
    }

    pub fn get_inner_size(&self) -> LogicalSize<f32> {
        let scale_factor = self.window.scale_factor();

        self.window.inner_size().to_logical(scale_factor)
    }

    pub fn display<F>(&mut self, f: F)
    where
        F: Fn(&mut DisplayListBuilder, SpaceAndClipInfo),
    {
        let pipeline_id = PipelineId(0, 0);
        if self.display_list_builder.is_none() {
            let (_, layout_size) = Self::get_size(&self.window);
            let builder = DisplayListBuilder::new(pipeline_id, layout_size);

            self.display_list_builder = Some(builder);
        }

        if let Some(builder) = &mut self.display_list_builder {
            let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

            f(builder, space_and_clip);
        }
    }

    pub fn flush(&mut self) {
        let builder = std::mem::replace(&mut self.display_list_builder, None);

        if let Some(builder) = builder {
            let (_, layout_size) = Self::get_size(&self.window);

            let epoch = Epoch(0);
            let mut txn = Transaction::new();

            txn.set_display_list(epoch, None, layout_size, builder.finalize(), true);

            txn.generate_frame();

            self.render_api.send_transaction(self.document_id, txn);
        }
    }

    pub fn clear_display_list_builder(&mut self) {
        let _ = std::mem::replace(&mut self.display_list_builder, None);
    }

    pub fn add_font_instance(&self, font_key: FontKey, pixel_size: i32) -> FontInstanceKey {
        let mut txn = Transaction::new();

        let font_instance_key = self.render_api.generate_font_instance_key();

        txn.add_font_instance(
            font_instance_key,
            font_key,
            app_units::Au::from_px(pixel_size),
            None,
            None,
            vec![],
        );

        self.render_api.send_transaction(self.document_id, txn);
        font_instance_key
    }

    pub fn add_font(&self, font_handle: &FontHandle) -> FontKey {
        let mut txn = Transaction::new();

        let font_key = self.render_api.generate_font_key();
        match font_handle {
            FontHandle::Path { path, font_index } => {
                let font = NativeFontHandle {
                    path: path.clone().into_os_string().into(),
                    index: *font_index,
                };
                txn.add_native_font(font_key, font);
            }
            FontHandle::Memory { bytes, font_index } => {
                txn.add_raw_font(font_key, bytes.to_vec(), *font_index);
            }
        }

        self.render_api.send_transaction(self.document_id, txn);

        font_key
    }

    pub fn get_color_bits(&self) -> u8 {
        self.color_bits
    }

    pub fn get_available_monitors(&self) -> impl Iterator<Item = MonitorHandle> {
        self.window.available_monitors()
    }

    pub fn get_primary_monitor(&self) -> MonitorHandle {
        self.window.primary_monitor()
    }

    pub fn get_position(&self) -> Option<PhysicalPosition<i32>> {
        self.window.outer_position().ok()
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
