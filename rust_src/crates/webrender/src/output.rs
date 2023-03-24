use super::display_info::DisplayInfoRef;
use super::font::FontRef;
use super::image_cache::ImageHash;
use crate::font_db::FontDB;
use crate::frame::LispFrameWindowSystemExt;
use crate::gl::context::GLContextTrait;
use crate::window_system::output::output;
use crate::window_system::output::OutputInner;
use crate::window_system::output::OutputInnerRef;
use emacs::bindings::Emacs_Pixmap;
use emacs::lisp::ExternalPtr;
use std::fmt;
use std::ptr;
use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::frame::LispFrameExt;
use crate::WRFontRef;
use gleam::gl;
use std::collections::HashMap;

use webrender::{self, api::units::*, api::*, RenderApi, Renderer, Transaction};

use emacs::frame::LispFrameRef;

use super::texture::TextureResourceManager;

pub struct Canvas {
    fonts: HashMap<fontdb::ID, FontKey>,
    font_instances: HashMap<
        (
            FontKey,
            FontSize,
            FontInstanceFlags,
            Option<ColorU>,
            SyntheticItalics,
        ),
        FontInstanceKey,
    >,
    images: HashMap<ImageHash, ImageKey>,
    font_render_mode: Option<FontRenderMode>,
    allow_mipmaps: bool,
    pub render_api: RenderApi,
    pub document_id: DocumentId,
    pipeline_id: PipelineId,
    epoch: Epoch,
    display_list_builder: Option<DisplayListBuilder>,
    previous_frame_image: Option<ImageKey>,
    texture_resources: Rc<RefCell<TextureResourceManager>>,
    renderer: Renderer,
    gl_context: crate::gl::context::GLContext,
    gl: Rc<dyn gl::Gl>,
    frame: LispFrameRef,
}

impl fmt::Debug for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "canvas")
    }
}

impl Canvas {
    pub fn build(frame: LispFrameRef) -> Self {
        let mut gl_context = frame.create_gl_context();
        let gl = gl_context.load_gl();
        gl_context.ensure_is_current();

        // webrender
        let webrender_opts = webrender::WebRenderOptions {
            clear_color: ColorF::new(1.0, 1.0, 1.0, 1.0),
            ..webrender::WebRenderOptions::default()
        };

        let notifier = Box::new(Notifier::new());
        let (mut renderer, sender) =
            webrender::create_webrender_instance(gl.clone(), notifier, webrender_opts, None)
                .unwrap();

        let texture_resources = Rc::new(RefCell::new(TextureResourceManager::new(
            gl.clone(),
            sender.create_api(),
        )));

        let external_image_handler = texture_resources.borrow_mut().new_external_image_handler();

        renderer.set_external_image_handler(external_image_handler);

        let epoch = Epoch(0);
        let pipeline_id = PipelineId(0, 0);

        // Some thing to do with Wayland?
        let mut txn = Transaction::new();
        txn.set_root_pipeline(pipeline_id);
        let mut api = sender.create_api();
        let device_size = frame.physical_size();
        gl_context.resize(&device_size);
        let document_id = api.add_document(device_size);
        api.send_transaction(document_id, txn);

        Self {
            fonts: HashMap::new(),
            font_instances: HashMap::new(),
            images: HashMap::new(),
            font_render_mode: None,
            allow_mipmaps: false,
            render_api: api,
            document_id,
            pipeline_id,
            epoch,
            display_list_builder: None,
            previous_frame_image: None,
            renderer,
            gl_context,
            gl,
            texture_resources,
            frame,
        }
    }

    fn copy_framebuffer_to_texture(&self, device_rect: DeviceIntRect) -> ImageKey {
        let mut origin = device_rect.min;

        let device_size = self.device_size();

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

        let gl = &self.gl;
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

    pub fn scale(&self) -> f32 {
        self.frame.scale_factor() as f32
    }

    fn layout_size(&self) -> LayoutSize {
        let device_size = self.device_size();
        LayoutSize::new(device_size.width as f32, device_size.height as f32)
    }

    fn new_builder(&mut self, image: Option<(ImageKey, LayoutRect)>) -> DisplayListBuilder {
        let pipeline_id = self.pipeline_id;

        let layout_size = self.layout_size();
        let mut builder = DisplayListBuilder::new(pipeline_id);
        builder.begin();

        if let Some((image_key, image_rect)) = image {
            let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

            let bounds = LayoutRect::from_size(layout_size);

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

    pub fn device_size(&self) -> DeviceIntSize {
        self.frame.physical_size()
    }

    pub fn display<F>(&mut self, f: F)
    where
        F: Fn(&mut DisplayListBuilder, SpaceAndClipInfo, f32),
    {
        if self.display_list_builder.is_none() {
            let layout_size = self.layout_size();

            let image_and_pos = self
                .previous_frame_image
                .map(|image_key| (image_key, LayoutRect::from_size(layout_size)));

            self.display_list_builder = Some(self.new_builder(image_and_pos));
        }

        let pipeline_id = PipelineId(0, 0);
        let scale = self.scale();

        if let Some(builder) = &mut self.display_list_builder {
            let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

            f(builder, space_and_clip, scale);
        }

        self.assert_no_gl_error();
    }

    fn ensure_context_is_current(&mut self) {
        self.gl_context.ensure_is_current();
        self.assert_no_gl_error();
    }

    #[track_caller]
    fn assert_no_gl_error(&self) {
        debug_assert_eq!(self.gl.get_error(), gleam::gl::NO_ERROR);
    }

    pub fn flush(&mut self) {
        self.assert_no_gl_error();
        self.ensure_context_is_current();

        let builder = std::mem::replace(&mut self.display_list_builder, None);

        if let Some(mut builder) = builder {
            let layout_size = self.layout_size();

            let epoch = self.epoch;
            let mut txn = Transaction::new();

            txn.set_display_list(epoch, None, layout_size.to_f32(), builder.end());
            txn.set_root_pipeline(self.pipeline_id);
            txn.generate_frame(0, RenderReasons::NONE);

            self.display_list_builder = None;

            self.render_api.send_transaction(self.document_id, txn);

            self.render_api.flush_scene_builder();

            let device_size = self.device_size();

            self.gl_context.bind_framebuffer(&mut self.gl);

            self.renderer.update();

            self.assert_no_gl_error();

            self.renderer.render(device_size, 0).unwrap();
            let _ = self.renderer.flush_pipeline_info();

            self.texture_resources.borrow_mut().clear();

            let image_key = self.copy_framebuffer_to_texture(DeviceIntRect::from_size(device_size));
            self.previous_frame_image = Some(image_key);

            self.gl_context.swap_buffers();
        }
    }

    pub fn get_previous_frame(&self) -> Option<ImageKey> {
        self.previous_frame_image
    }

    pub fn clear_display_list_builder(&mut self) {
        let _ = std::mem::replace(&mut self.display_list_builder, None);
    }

    pub fn wr_add_font_instance(
        &mut self,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
        render_mode: Option<FontRenderMode>,
        bg_color: Option<ColorU>,
        synthetic_italics: SyntheticItalics,
    ) -> FontInstanceKey {
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();

        let key = self.render_api.generate_font_instance_key();
        let mut txn = Transaction::new();
        let mut options: FontInstanceOptions = Default::default();
        options.flags |= flags;
        if let Some(render_mode) = render_mode {
            options.render_mode = render_mode;
        }
        if let Some(bg_color) = bg_color {
            options.bg_color = bg_color;
        }
        options.synthetic_italics = synthetic_italics;
        txn.add_font_instance(key, font_key, size, Some(options), None, Vec::new());
        self.render_api.send_transaction(self.document_id, txn);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let elapsed = now.elapsed();
            log::trace!("wr add font instance in {:?}", elapsed);
        }
        key
    }

    #[allow(dead_code)]
    pub fn wr_delete_font_instance(&mut self, key: FontInstanceKey) {
        let mut txn = Transaction::new();
        txn.delete_font_instance(key);
        self.render_api.send_transaction(self.document_id, txn);
    }

    pub fn wr_add_font(&mut self, data: FontTemplate) -> FontKey {
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();
        let font_key = self.render_api.generate_font_key();
        let mut txn = Transaction::new();
        match data {
            FontTemplate::Raw(ref bytes, index) => {
                txn.add_raw_font(font_key, bytes.to_vec(), index)
            }
            FontTemplate::Native(native_font) => txn.add_native_font(font_key, native_font),
        }

        self.render_api.send_transaction(self.document_id, txn);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let elapsed = now.elapsed();
            log::trace!("wr add font in {:?}", elapsed);
        }
        font_key
    }

    pub fn allow_mipmaps(&mut self, allow_mipmaps: bool) {
        self.allow_mipmaps = allow_mipmaps;
    }

    pub fn set_font_render_mode(&mut self, render_mode: Option<FontRenderMode>) {
        self.font_render_mode = render_mode;
    }

    pub fn get_or_create_font(&mut self, font: WRFontRef) -> Option<FontKey> {
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();
        let font_id = font.face_id;
        let wr_font_key = self.fonts.get(&font_id);

        if let Some(key) = wr_font_key {
            return Some(*key);
        }

        let wr_font_key = {
            #[cfg(macos_platform)]
            {
                let app_locale = fontdb::Language::English_UnitedStates;
                let face_info = FontDB::global()
                    .db()
                    .face(font.face_id)
                    .expect("Failed to find face info");
                let family = face_info
                    .families
                    .iter()
                    .find(|family| family.1 == app_locale);

                if let Some((name, _)) = family {
                    let key = self.wr_add_font(FontTemplate::Native(NativeFontHandle {
                        name: name.to_owned(),
                    }));
                    Some(key)
                } else {
                    None
                }
            }

            #[cfg(not(macos_platform))]
            {
                let font_result = FontDB::global().get_font(font.face_id);

                if font_result.is_none() {
                    return None;
                }

                let font_result = font_result.unwrap();
                let (font_bytes, face_index) = (font_result.data, font_result.info.index);
                Some(self.wr_add_font(FontTemplate::Raw(
                    Arc::new(font_bytes.to_vec()),
                    face_index.try_into().unwrap(),
                )))
            }
        };

        if let Some(key) = wr_font_key {
            self.fonts.insert(font_id, key);
            return Some(key);
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let elapsed = now.elapsed();
            log::trace!("get_or_create_font in {:?}", elapsed);
        }

        None
    }

    // Create font instance with scaled size
    pub fn get_or_create_font_instance(&mut self, font: WRFontRef, size: f32) -> FontInstanceKey {
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();
        let font_key = self
            .get_or_create_font(font)
            .expect("Failed to obtain wr fontkey");
        let bg_color = None;
        let flags = FontInstanceFlags::empty();
        let synthetic_italics = SyntheticItalics::disabled();
        let font_render_mode = self.font_render_mode;
        let hash_map_key = (font_key, size.into(), flags, bg_color, synthetic_italics);
        let font_instance_key = self.font_instances.get(&hash_map_key);
        let key = match font_instance_key {
            Some(instance_key) => *instance_key,
            None => {
                let instance_key = self.wr_add_font_instance(
                    font_key,
                    size,
                    flags,
                    font_render_mode,
                    bg_color,
                    synthetic_italics,
                );
                self.font_instances.insert(hash_map_key, instance_key);
                instance_key
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let elapsed = now.elapsed();
            log::trace!("get_or_create_font_instance in {:?}", elapsed);
        }
        key
    }

    pub fn add_image(&mut self, descriptor: ImageDescriptor, data: ImageData) -> ImageKey {
        let image_key = self.render_api.generate_image_key();
        let mut txn = Transaction::new();

        txn.add_image(image_key, descriptor, data, None);

        self.render_api.send_transaction(self.document_id, txn);

        image_key
    }

    pub fn update_image(&mut self, key: ImageKey, descriptor: ImageDescriptor, data: ImageData) {
        let mut txn = Transaction::new();

        txn.update_image(key, descriptor, data, &DirtyRect::All);

        self.render_api.send_transaction(self.document_id, txn);
    }

    pub fn delete_image(&mut self, image_key: ImageKey) {
        let mut txn = Transaction::new();

        txn.delete_image(image_key);

        self.render_api.send_transaction(self.document_id, txn);
    }

    pub fn delete_image_by_pixmap(&mut self, _pixmap: Emacs_Pixmap) {
        // We cache image by source from image_cache.rs
        // transform(rotate, resize(scale)) on the fly
        // loop images, compare pixmap,find image_key
        log::warn!("TODO free pixmap");
    }

    // Create glyph raster image instance with scaled size
    pub fn add_or_update_image(
        &mut self,
        hash: &ImageHash,
        descriptor: ImageDescriptor,
        data: ImageData,
    ) -> ImageKey {
        let image_key = self.image_key(&hash);

        if let Some(key) = image_key {
            self.update_image(key, descriptor, data);
            return key;
        }

        let key = self.add_image(descriptor, data);
        self.images.insert(*hash, key);
        key
    }

    pub fn image_key(&self, hash: &ImageHash) -> Option<ImageKey> {
        self.images.get(hash).copied()
    }

    pub fn update(&mut self) {
        let size = self.device_size();
        let device_rect =
            DeviceIntRect::from_origin_and_size(DeviceIntPoint::new(0, 0), size.clone());
        log::debug!("resize {size:?} rect {device_rect:?}");
        let mut txn = Transaction::new();
        txn.set_document_view(device_rect);
        self.render_api.send_transaction(self.document_id, txn);

        self.gl_context.resize(&size);
    }

    pub fn deinit(mut self) {
        self.ensure_context_is_current();
        self.renderer.deinit();
    }
}

pub type CanvasRef = ExternalPtr<Canvas>;

#[derive(Default)]
#[repr(transparent)]
pub struct Output(output);
pub type OutputRef = ExternalPtr<Output>;

impl Output {
    pub fn new() -> Self {
        let ret = Output::default();
        ret
    }

    pub fn empty_inner(&mut self) {
        let _ = unsafe { Box::from_raw(self.inner().as_mut()) };
        self.0.inner = ptr::null_mut();
    }

    pub fn set_inner(&mut self, inner: Box<OutputInner>) {
        self.0.inner = Box::into_raw(inner) as *mut libc::c_void;
    }

    pub fn set_font(&mut self, mut font: FontRef) {
        self.0.font = font.as_mut();
    }

    pub fn set_fontset(&mut self, fontset: i32) {
        self.0.fontset = fontset;
    }

    pub fn display_info(&self) -> DisplayInfoRef {
        DisplayInfoRef::new(self.0.display_info as *mut _)
    }

    pub fn canvas(&mut self) -> CanvasRef {
        self.inner().canvas
    }

    pub fn inner(&mut self) -> OutputInnerRef {
        if self.0.inner.is_null() {
            self.set_inner(Box::new(OutputInner::default()));
        }

        OutputInnerRef::new(self.0.inner as *mut OutputInner)
    }

    pub fn as_raw(&mut self) -> ExternalPtr<output> {
        (&mut self.0 as *mut output).into()
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        if self.0.inner != ptr::null_mut() {
            if OutputInnerRef::new(self.0.inner as *mut OutputInner)
                .canvas
                .as_mut()
                != ptr::null_mut()
            {
                unsafe {
                    let _ = Box::from_raw(
                        OutputInnerRef::new(self.0.inner as *mut OutputInner)
                            .canvas
                            .as_mut(),
                    );
                }
            }

            unsafe {
                let _ = Box::from_raw(self.0.inner as *mut OutputInner);
            }
        }
    }
}

struct Notifier {}

impl Notifier {
    fn new() -> Notifier {
        Notifier {}
    }
}

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<dyn RenderNotifier> {
        Box::new(Notifier {})
    }

    fn wake_up(&self, _composite_needed: bool) {}

    fn new_frame_ready(&self, _: DocumentId, _scrolled: bool, composite_needed: bool) {
        self.wake_up(composite_needed);
    }
}
