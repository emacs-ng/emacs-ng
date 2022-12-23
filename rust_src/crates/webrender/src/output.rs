use std::{cell::RefCell, mem::MaybeUninit, rc::Rc, sync::Arc};

use gleam::gl::{self, Gl};
use glutin::{
    self,
    dpi::PhysicalSize,
    window::{CursorIcon, Window},
    ContextWrapper, PossiblyCurrent,
};
use std::{
    ops::{Deref, DerefMut},
    ptr,
};

#[cfg(not(any(target_os = "macos", windows)))]
use glutin::platform::unix::WindowBuilderExtUnix;

use webrender::{self, api::units::*, api::*, RenderApi, Renderer, Transaction};

use emacs::{
    bindings::{wr_output, Emacs_Cursor},
    frame::LispFrameRef,
};

use crate::event_loop::WrEventLoop;

use super::texture::TextureResourceManager;
use super::util::HandyDandyRectBuilder;
use super::{cursor::emacs_to_winit_cursor, display_info::DisplayInfoRef};
use super::{cursor::winit_to_emacs_cursor, font::FontRef};

#[cfg(all(feature = "wayland", not(any(target_os = "macos", windows))))]
use emacs::{bindings::globals, multibyte::LispStringRef};

pub struct Output {
    // Extend `wr_output` struct defined in `wrterm.h`
    pub output: wr_output,

    pub font: FontRef,
    pub fontset: i32,

    pub render_api: RenderApi,
    pub document_id: DocumentId,

    display_list_builder: Option<DisplayListBuilder>,
    previous_frame_image: Option<ImageKey>,

    pub background_color: ColorF,
    pub cursor_color: ColorF,
    pub cursor_foreground_color: ColorF,

    color_bits: u8,

    // The drop order is important here.

    // Need to dropped before window context
    texture_resources: Rc<RefCell<TextureResourceManager>>,

    // Need to droppend before window context
    renderer: Renderer,

    window_context: ContextWrapper<PossiblyCurrent, Window>,

    frame: LispFrameRef,
}

impl Output {
    pub fn build(event_loop: &mut WrEventLoop, frame: LispFrameRef) -> Self {
        let window_builder = winit::window::WindowBuilder::new().with_visible(true);

        #[cfg(all(feature = "wayland", not(any(target_os = "macos", windows))))]
        let window_builder = {
            let invocation_name: LispStringRef = unsafe { globals.Vinvocation_name.into() };
            let invocation_name = invocation_name.to_utf8();
            window_builder.with_app_id(invocation_name)
        };

        let context_builder = glutin::ContextBuilder::new();

        let window_context = event_loop
            .build_window(window_builder, context_builder)
            .unwrap();

        let window_id = window_context.window().id();

        event_loop.wait_for_window_resize(window_id);

        let window_context = unsafe { window_context.make_current() }.unwrap();

        let window = window_context.window();

        window_context.resize(window.inner_size());

        let gl = Self::get_gl_api(&window_context);

        let webrender_opts = webrender::RendererOptions {
            clear_color: None,
            ..webrender::RendererOptions::default()
        };

        let notifier = Box::new(Notifier::new());

        let (mut renderer, sender) =
            webrender::Renderer::new(gl.clone(), notifier, webrender_opts, None).unwrap();

        let color_bits = window_context.get_pixel_format().color_bits;

        let texture_resources = Rc::new(RefCell::new(TextureResourceManager::new(
            gl.clone(),
            sender.create_api(),
        )));

        let external_image_handler = texture_resources.borrow_mut().new_external_image_handler();

        renderer.set_external_image_handler(external_image_handler);

        let pipeline_id = PipelineId(0, 0);
        let mut txn = Transaction::new();
        txn.set_root_pipeline(pipeline_id);

        let device_size = {
            let size = window.inner_size();
            DeviceIntSize::new(size.width as i32, size.height as i32)
        };

        let mut api = sender.create_api();

        let document_id = api.add_document(device_size);
        api.send_transaction(document_id, txn);

        let mut output = Self {
            output: wr_output::default(),
            font: FontRef::new(ptr::null_mut()),
            fontset: 0,
            render_api: api,
            document_id,
            display_list_builder: None,
            previous_frame_image: None,
            background_color: ColorF::WHITE,
            cursor_color: ColorF::BLACK,
            cursor_foreground_color: ColorF::WHITE,
            color_bits,
            renderer,
            window_context,
            texture_resources,
            frame,
        };

        Self::build_mouse_cursors(&mut output);

        output
    }

    fn copy_framebuffer_to_texture(&self, device_rect: DeviceIntRect) -> ImageKey {
        let mut origin = device_rect.min;

        let device_size = self.get_deivce_size();

        if !self.renderer.device.surface_origin_is_top_left() {
            origin.y = device_size.height - origin.y - device_rect.height();
        }

        let fb_rect = FramebufferIntRect::from_origin_and_size(
            FramebufferIntPoint::from_untyped(origin.to_untyped()),
            FramebufferIntSize::from_untyped(device_rect.size().to_untyped()),
        );

        let need_flip = !self.renderer.device.surface_origin_is_top_left();

        let (image_key, texture_id) = self.texture_resources.borrow_mut().new_image(
            self.document_id,
            fb_rect.size(),
            need_flip,
        );

        let gl = Self::get_gl_api(&self.window_context);
        gl.bind_texture(gl::TEXTURE_2D, texture_id);

        gl.copy_tex_sub_image_2d(
            gl::TEXTURE_2D,
            0,
            0,
            0,
            fb_rect.min.x,
            fb_rect.min.y,
            fb_rect.size().width,
            fb_rect.size().height,
        );

        gl.bind_texture(gl::TEXTURE_2D, 0);

        image_key
    }

    fn get_gl_api(window_context: &ContextWrapper<PossiblyCurrent, Window>) -> Rc<dyn Gl> {
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

    fn get_size(window: &Window) -> LayoutSize {
        let physical_size = window.inner_size();
        let device_size = LayoutSize::new(physical_size.width as f32, physical_size.height as f32);
        device_size
    }

    fn new_builder(&mut self, image: Option<(ImageKey, LayoutRect)>) -> DisplayListBuilder {
        let pipeline_id = PipelineId(0, 0);

        let layout_size = Self::get_size(&self.get_window());
        let mut builder = DisplayListBuilder::new(pipeline_id);

        if let Some((image_key, image_rect)) = image {
            let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

            let bounds = (0, 0).by(layout_size.width as i32, layout_size.height as i32);

            builder.push_image(
                &CommonItemProperties::new(bounds, space_and_clip),
                image_rect,
                ImageRendering::Auto,
                AlphaType::PremultipliedAlpha,
                image_key,
                ColorF::WHITE,
            );
        }

        builder
    }

    pub fn show_window(&self) {
        self.get_window().set_visible(true);
    }
    pub fn hide_window(&self) {
        self.get_window().set_visible(false);
    }

    pub fn maximize(&self) {
        self.get_window().set_maximized(true);
    }

    pub fn set_title(&self, title: &str) {
        self.get_window().set_title(title);
    }

    pub fn set_display_info(&mut self, mut dpyinfo: DisplayInfoRef) {
        self.output.display_info = dpyinfo.get_raw().as_mut();
    }

    pub fn get_frame(&self) -> LispFrameRef {
        self.frame
    }

    pub fn display_info(&self) -> DisplayInfoRef {
        DisplayInfoRef::new(self.output.display_info as *mut _)
    }

    pub fn get_inner_size(&self) -> PhysicalSize<u32> {
        self.get_window().inner_size()
    }

    fn get_deivce_size(&self) -> DeviceIntSize {
        let size = self.get_window().inner_size();
        DeviceIntSize::new(size.width as i32, size.height as i32)
    }

    pub fn display<F>(&mut self, f: F)
    where
        F: Fn(&mut DisplayListBuilder, SpaceAndClipInfo),
    {
        if self.display_list_builder.is_none() {
            let layout_size = Self::get_size(&self.get_window());

            let image_and_pos = self
                .previous_frame_image
                .map(|image_key| (image_key, LayoutRect::from_size(layout_size)));

            self.display_list_builder = Some(self.new_builder(image_and_pos));
        }

        let pipeline_id = PipelineId(0, 0);

        if let Some(builder) = &mut self.display_list_builder {
            let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

            f(builder, space_and_clip);
        }
    }

    fn ensure_context_is_current(&mut self) {
        let window_context = std::mem::replace(&mut self.window_context, unsafe {
            MaybeUninit::uninit().assume_init()
        });
        let window_context = unsafe { window_context.make_current() }.unwrap();

        let temp_context = std::mem::replace(&mut self.window_context, window_context);
        std::mem::forget(temp_context);
    }

    pub fn flush(&mut self) {
        let builder = std::mem::replace(&mut self.display_list_builder, None);

        if let Some(builder) = builder {
            let layout_size = Self::get_size(&self.get_window());

            let epoch = Epoch(0);
            let mut txn = Transaction::new();

            txn.set_display_list(epoch, None, layout_size.to_f32(), builder.finalize(), true);

            txn.generate_frame(0);

            self.render_api.send_transaction(self.document_id, txn);

            self.render_api.flush_scene_builder();

            let device_size = self.get_deivce_size();

            self.renderer.update();

            self.ensure_context_is_current();

            self.renderer.render(device_size, 0).unwrap();
            let _ = self.renderer.flush_pipeline_info();

            self.window_context.swap_buffers().ok();

            self.texture_resources.borrow_mut().clear();

            let image_key = self.copy_framebuffer_to_texture(DeviceIntRect::from_size(device_size));
            self.previous_frame_image = Some(image_key);
        }
    }

    pub fn get_previous_frame(&self) -> Option<ImageKey> {
        self.previous_frame_image
    }

    pub fn clear_display_list_builder(&mut self) {
        let _ = std::mem::replace(&mut self.display_list_builder, None);
    }

    pub fn add_font_instance(&mut self, font_key: FontKey, pixel_size: i32) -> FontInstanceKey {
        let mut txn = Transaction::new();

        let font_instance_key = self.render_api.generate_font_instance_key();

        txn.add_font_instance(
            font_instance_key,
            font_key,
            pixel_size as f32,
            None,
            None,
            vec![],
        );

        self.render_api.send_transaction(self.document_id, txn);
        font_instance_key
    }

    pub fn add_font(&mut self, font_bytes: Rc<Vec<u8>>, font_index: u32) -> FontKey {
        let mut txn = Transaction::new();

        let font_key = self.render_api.generate_font_key();

        txn.add_raw_font(font_key, font_bytes.to_vec(), font_index);

        self.render_api.send_transaction(self.document_id, txn);

        font_key
    }

    pub fn get_color_bits(&self) -> u8 {
        self.color_bits
    }

    pub fn get_window(&self) -> &Window {
        self.window_context.window()
    }

    fn build_mouse_cursors(output: &mut Output) {
        output.output.text_cursor = winit_to_emacs_cursor(CursorIcon::Text);
        output.output.nontext_cursor = winit_to_emacs_cursor(CursorIcon::Arrow);
        output.output.modeline_cursor = winit_to_emacs_cursor(CursorIcon::Hand);
        output.output.hand_cursor = winit_to_emacs_cursor(CursorIcon::Hand);
        output.output.hourglass_cursor = winit_to_emacs_cursor(CursorIcon::Progress);

        output.output.horizontal_drag_cursor = winit_to_emacs_cursor(CursorIcon::ColResize);
        output.output.vertical_drag_cursor = winit_to_emacs_cursor(CursorIcon::RowResize);

        output.output.left_edge_cursor = winit_to_emacs_cursor(CursorIcon::WResize);
        output.output.right_edge_cursor = winit_to_emacs_cursor(CursorIcon::EResize);
        output.output.top_edge_cursor = winit_to_emacs_cursor(CursorIcon::NResize);
        output.output.bottom_edge_cursor = winit_to_emacs_cursor(CursorIcon::SResize);

        output.output.top_left_corner_cursor = winit_to_emacs_cursor(CursorIcon::NwResize);
        output.output.top_right_corner_cursor = winit_to_emacs_cursor(CursorIcon::NeResize);

        output.output.bottom_left_corner_cursor = winit_to_emacs_cursor(CursorIcon::SwResize);
        output.output.bottom_right_corner_cursor = winit_to_emacs_cursor(CursorIcon::SeResize);
    }

    pub fn set_mouse_cursor(&self, cursor: Emacs_Cursor) {
        let cursor = emacs_to_winit_cursor(cursor);

        self.get_window().set_cursor_icon(cursor)
    }

    pub fn add_image(&mut self, width: i32, height: i32, image_data: Arc<Vec<u8>>) -> ImageKey {
        let image_key = self.render_api.generate_image_key();

        self.update_image(image_key, width, height, image_data);

        image_key
    }

    pub fn update_image(
        &mut self,
        image_key: ImageKey,
        width: i32,
        height: i32,
        image_data: Arc<Vec<u8>>,
    ) {
        let mut txn = Transaction::new();

        txn.add_image(
            image_key,
            ImageDescriptor::new(
                width,
                height,
                ImageFormat::RGBA8,
                ImageDescriptorFlags::empty(),
            ),
            ImageData::Raw(image_data),
            None,
        );

        self.render_api.send_transaction(self.document_id, txn);
    }

    pub fn delete_image(&mut self, image_key: ImageKey) {
        let mut txn = Transaction::new();

        txn.delete_image(image_key);

        self.render_api.send_transaction(self.document_id, txn);
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        let device_size = DeviceIntSize::new(size.width as i32, size.height as i32);

        let device_rect =
            DeviceIntRect::from_origin_and_size(DeviceIntPoint::new(0, 0), device_size);

        let mut txn = Transaction::new();
        txn.set_document_view(device_rect);
        self.render_api.send_transaction(self.document_id, txn);

        self.window_context.resize(size.clone());
    }
}

#[derive(PartialEq)]
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

    pub fn as_rust_ptr(&mut self) -> *mut Output {
        self.0 as *mut Output
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

struct Notifier;

impl Notifier {
    fn new() -> Notifier {
        Notifier
    }
}

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<dyn RenderNotifier> {
        Box::new(Notifier)
    }

    fn wake_up(&self, _composite_needed: bool) {}

    fn new_frame_ready(
        &self,
        _: DocumentId,
        _scrolled: bool,
        _composite_needed: bool,
        _render_time: Option<u64>,
    ) {
    }
}
