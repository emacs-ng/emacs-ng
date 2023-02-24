use crate::gl::context::ContextTrait;
use gleam::gl::ErrorCheckingGl;
use glutin::{
    config::{Api, ConfigTemplateBuilder, GlConfig},
    context::{
        ContextApi, ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor,
        PossiblyCurrentContext, PossiblyCurrentContextGlSurfaceAccessor, Version,
    },
    display::{Display, DisplayApiPreference, GetGlDisplay, GlDisplay},
    prelude::GlSurface,
    surface::{Surface, SurfaceAttributesBuilder, WindowSurface},
};
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use winit::dpi::PhysicalSize;

use std::{ffi::CString, num::NonZeroU32};

use std::rc::Rc;

use gleam::gl::{Gl, GlFns, GlesFns};

pub struct ContextImpl {
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    gl: Rc<dyn Gl>,
}

impl ContextTrait for ContextImpl {
    fn build(
        size: PhysicalSize<u32>,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Self {
        log::trace!("Initialize OpenGL context using Glutin");
        let width = NonZeroU32::new(size.width as u32).unwrap();
        let height = NonZeroU32::new(size.height as u32).unwrap();

        // glutin
        let preference = DisplayApiPreference::Egl;
        let gl_display = unsafe { Display::new(display_handle, preference) }.unwrap();
        let template = ConfigTemplateBuilder::new().build(); // TODO do we need to do anything to this?
        let gl_config = unsafe {
            let configs = gl_display.find_configs(template).unwrap();
            // get best config
            configs
                .reduce(|accum, config| {
                    if config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        };

        let context_attributes = ContextAttributesBuilder::new().build(Some(window_handle));
        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try gles.
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(window_handle));

        // There are also some old devices that support neither modern OpenGL nor GLES.
        // To support these we can try and create a 2.1 context.
        let legacy_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
            .build(Some(window_handle));

        let mut context = Some(unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .unwrap_or_else(|_| {
                            gl_display
                                .create_context(&gl_config, &legacy_context_attributes)
                                .expect("failed to create context")
                        })
                })
        });

        let attrs =
            SurfaceAttributesBuilder::<WindowSurface>::new().build(window_handle, width, height);
        let surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };
        let context = context.take().unwrap().make_current(&surface).unwrap();
        // TODO haven't we done this above?
        surface.resize(&context, width, height);

        let gl = {
            let flags = gl_config.api();
            if flags.contains(Api::OPENGL) {
                unsafe {
                    GlFns::load_with(|symbol| {
                        gl_config
                            .display()
                            .get_proc_address(&CString::new(symbol).unwrap())
                            as *const _
                    })
                }
            } else if flags.intersects(Api::GLES1 | Api::GLES2 | Api::GLES3) {
                unsafe {
                    GlesFns::load_with(|symbol| {
                        gl_config
                            .display()
                            .get_proc_address(&CString::new(symbol).unwrap())
                            as *const _
                    })
                }
            } else {
                unimplemented!();
            }
        };

        Self {
            surface,
            context,
            gl,
        }
    }

    fn bind_framebuffer(&mut self, _gl: &mut Rc<dyn Gl>) {}

    fn swap_buffers(&self) {
        self.surface.swap_buffers(&self.context).ok();
    }

    fn resize(&self, size: &PhysicalSize<u32>) {
        self.surface.resize(
            &self.context,
            NonZeroU32::new(size.width as u32).unwrap(),
            NonZeroU32::new(size.height as u32).unwrap(),
        );
    }

    fn load_gl(&self) -> Rc<dyn Gl> {
        ErrorCheckingGl::wrap(self.gl.clone())
    }

    fn ensure_is_current(&mut self) {
        // Make sure the gl context is made current.
        if let Err(err) = self.context.make_current(&self.surface) {
            log::error!("Failed to make GL context current: {:?}", err);
        }
    }
}
