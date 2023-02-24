use crate::gl::context::GLContextTrait;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use webrender::api::units::DeviceIntSize;

use surfman::{Connection, GLApi, SurfaceType};

use euclid::Size2D;

use std::rc::Rc;

use gleam::gl::{ErrorCheckingGl, Gl, GlFns, GlesFns};

use webrender_surfman::WebrenderSurfman;

pub struct ContextImpl(WebrenderSurfman);

impl ContextImpl {
    #[track_caller]
    fn assert_gl_framebuffer_complete(&self, gl: &Rc<dyn Gl>) {
        debug_assert_eq!(
            (
                gl.get_error(),
                gl.check_frame_buffer_status(gleam::gl::FRAMEBUFFER)
            ),
            (gleam::gl::NO_ERROR, gleam::gl::FRAMEBUFFER_COMPLETE)
        );
    }
}

impl GLContextTrait for ContextImpl {
    fn build(
        size: DeviceIntSize,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Self {
        log::trace!("Initialize OpenGL context using Surfman");

        let connection = match Connection::from_raw_display_handle(display_handle) {
            Ok(connection) => connection,
            Err(error) => panic!("Device not open {:?}", error),
        };

        let adapter = connection
            .create_adapter()
            .expect("Failed to create adapter");

        let native_widget = connection
            .create_native_widget_from_rwh(window_handle)
            .expect("Failed to create native widget");

        let surface_type = SurfaceType::Widget { native_widget };

        let webrender_surfman = WebrenderSurfman::create(&connection, &adapter, surface_type)
            .expect("Failed to create WR surfman");

        webrender_surfman
            .resize(Size2D::new(size.width as i32, size.height as i32))
            .unwrap();

        webrender_surfman.make_gl_context_current().unwrap();

        Self(webrender_surfman)
    }

    fn bind_framebuffer(&mut self, gl: &mut Rc<dyn Gl>) {
        // Bind the webrender framebuffer
        self.ensure_is_current();

        let framebuffer_object = self
            .0
            .context_surface_info()
            .unwrap_or(None)
            .map(|info| info.framebuffer_object)
            .unwrap_or(0);
        gl.bind_framebuffer(gleam::gl::FRAMEBUFFER, framebuffer_object);
        self.assert_gl_framebuffer_complete(gl);
    }

    fn swap_buffers(&self) {
        // Perform the page flip. This will likely block for a while.
        if let Err(err) = self.0.present() {
            log::error!("Failed to present surface: {:?}", err);
        }
    }

    fn load_gl(&self) -> Rc<dyn Gl> {
        // Get GL bindings
        let gl = match self.0.connection().gl_api() {
            GLApi::GL => unsafe { GlFns::load_with(|s| self.0.get_proc_address(s)) },
            GLApi::GLES => unsafe { GlesFns::load_with(|s| self.0.get_proc_address(s)) },
        };

        ErrorCheckingGl::wrap(gl)
    }

    fn resize(&self, size: &DeviceIntSize) {
        self.0
            .resize(Size2D::new(size.width as i32, size.height as i32))
            .unwrap();
    }

    fn ensure_is_current(&mut self) {
        if let Err(err) = self.0.make_gl_context_current() {
            log::error!("Failed to make gl context current: {:?}", err);
        }
    }
}
