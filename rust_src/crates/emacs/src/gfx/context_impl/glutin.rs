use crate::frame::FrameRef;
use crate::gfx::context::GLContextTrait;
use gleam::gl::ErrorCheckingGl;
use glutin::config::Api;
use glutin::config::ConfigTemplateBuilder;
use glutin::config::GlConfig;
use glutin::context::ContextApi;
use glutin::context::ContextAttributesBuilder;
use glutin::context::NotCurrentGlContext;
use glutin::context::PossiblyCurrentContext;
use glutin::context::PossiblyCurrentGlContext;
use glutin::context::Version;
use glutin::display::Display;
use glutin::display::DisplayApiPreference;
use glutin::display::GetGlDisplay;
use glutin::display::GlDisplay;
use glutin::prelude::GlSurface;
use glutin::surface::Surface;
use glutin::surface::SurfaceAttributesBuilder;
use glutin::surface::WindowSurface;

use webrender_api::units::DeviceIntSize;

use std::ffi::CString;
use std::num::NonZeroU32;

use std::rc::Rc;

use gleam::gl::Gl;
use gleam::gl::GlFns;
use gleam::gl::GlesFns;

pub struct ContextImpl {
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    gl: Rc<dyn Gl>,
}

impl GLContextTrait for ContextImpl {
    fn build(frame: &FrameRef) -> Self {
        log::trace!("Initialize OpenGL context using Glutin");

        let display_handle = frame.raw_display_handle().expect("None raw display handle");
        let window_handle = frame.raw_window_handle().expect("None raw window handle");
        let size = frame.physical_size();

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

    fn resize(&self, size: &DeviceIntSize) {
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
