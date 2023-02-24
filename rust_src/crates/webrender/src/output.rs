use crate::frame::LispFrameGlExt;
use crate::gl::context::GLContextTrait;
use emacs::lisp::ExternalPtr;
use std::fmt;
use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::frame::LispFrameExt;
use crate::WRFontRef;
use gleam::gl;
use std::collections::HashMap;

use webrender::{self, api::units::*, api::*, RenderApi, Renderer, Transaction};

use emacs::frame::LispFrameRef;

use super::texture::TextureResourceManager;
use super::util::HandyDandyRectBuilder;

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
        let size = frame.size();
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
        let device_size = DeviceIntSize::new(size.width as i32, size.height as i32);
        let document_id = api.add_document(device_size);
        api.send_transaction(document_id, txn);

        Self {
            fonts: HashMap::new(),
            font_instances: HashMap::new(),
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

    fn layout_size(&self) -> LayoutSize {
        LayoutSize::new(
            self.frame.pixel_width as f32,
            self.frame.pixel_height as f32,
        )
    }

    fn new_builder(&mut self, image: Option<(ImageKey, LayoutRect)>) -> DisplayListBuilder {
        let pipeline_id = self.pipeline_id;

        let layout_size = self.layout_size();
        let mut builder = DisplayListBuilder::new(pipeline_id);
        builder.begin();

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

    pub fn get_frame(&self) -> LispFrameRef {
        self.frame
    }

    pub fn device_size(&self) -> DeviceIntSize {
        DeviceIntSize::new(self.frame.pixel_width, self.frame.pixel_height)
    }

    pub fn display<F>(&mut self, f: F)
    where
        F: Fn(&mut DisplayListBuilder, SpaceAndClipInfo),
    {
        if self.display_list_builder.is_none() {
            let layout_size = self.layout_size();

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
            log::debug!("wr add font instance in {:?}", elapsed);
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
            log::debug!("wr add font in {:?}", elapsed);
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
        let font_id = font.face_info.id;
        let wr_font_key = self.fonts.get(&font_id);

        if let Some(key) = wr_font_key {
            return Some(*key);
        }

        let wr_font_key = {
            #[cfg(macos_platform)]
            {
                let app_locale = fontdb::Language::English_UnitedStates;
                let family = font
                    .face_info
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
                let font_result = font.cache().get_font(font.face_info.id);

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
            log::debug!("get_or_create_font in {:?}", elapsed);
        }

        None
    }

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
            log::debug!("get_or_create_font_instance in {:?}", elapsed);
        }
        key
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

    pub fn resize(&mut self, size: &DeviceIntSize) {
        log::trace!("resize {size:?}");
        let device_size = DeviceIntSize::new(size.width as i32, size.height as i32);
        self.frame.pixel_width = size.width;
        self.frame.pixel_height = size.height;

        let device_rect =
            DeviceIntRect::from_origin_and_size(DeviceIntPoint::new(0, 0), device_size);

        let mut txn = Transaction::new();
        txn.set_document_view(device_rect);
        self.render_api.send_transaction(self.document_id, txn);

        self.gl_context.resize(size);
    }

    pub fn deinit(mut self) {
        self.ensure_context_is_current();
        self.renderer.deinit();
    }
}

pub type CanvasRef = ExternalPtr<Canvas>;

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
