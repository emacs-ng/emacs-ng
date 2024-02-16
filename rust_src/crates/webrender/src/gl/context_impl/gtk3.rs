use crate::gl::context::GLContextTrait;
#[cfg(window_system_pgtk)]
use crate::window_system::frame::LispFramePgtkExt;
use emacs::frame::LispFrameRef;
use gleam::gl::ErrorCheckingGl;
use gleam::gl::Gl;
use gleam::gl::GlFns;
use gleam::gl::GlesFns;
use gtk::prelude::*;
use gtk::GLArea;
use std::rc::Rc;
use webrender::api::units::DeviceIntSize;

pub struct ContextImpl {
    area: GLArea,
    #[cfg(window_system_pgtk)]
    fixed: gtk::Fixed,
}

/// The API (OpenGL or OpenGL ES).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GLApi {
    /// OpenGL (full or desktop OpenGL).
    GL,
    /// OpenGL ES (embedded OpenGL).
    GLES,
}

impl ContextImpl {
    #[inline]
    fn get_proc_address(&self, addr: &str) -> *const core::ffi::c_void {
        gl_loader::get_proc_address(addr) as *const _
    }

    pub fn raw_handle(&self) -> &GLArea {
        &self.area
    }

    #[inline]
    fn gl_api(&self) -> GLApi {
        // TODO detect es
        // https://lazka.github.io/pgi-docs/Gtk-3.0/classes/GLArea.html#Gtk.GLArea.set_use_es

        GLApi::GL
    }
}

impl GLContextTrait for ContextImpl {
    fn build(frame: &LispFrameRef) -> Self {
        #[cfg(use_tao)]
        let area = {
            use crate::frame::LispFrameWindowSystemExt;
            use crate::window_system::api::platform::unix::WindowExtUnix;
            let frame_inner = frame.output().inner();
            let window = frame_inner.window.as_ref().expect("No window");

            let gtkwin = window.gtk_window();

            let vbox = gtkwin
                .children()
                .pop()
                .unwrap()
                .downcast::<gtk::Box>()
                .unwrap();
            // TODO config of pf_reqs and gl_attr
            let area = GLArea::builder().visible(true).has_alpha(true).build();
            vbox.pack_start(&area, true, true, 0);
            area.grab_focus();
            gtkwin.show_all();
            area
        };

        #[cfg(window_system_pgtk)]
        let (fixed, area) = {
            let fixed = frame.fixed_widget().expect("no fixed widget");

            let area = GLArea::builder()
                .width_request(fixed.allocated_width())
                .height_request(fixed.allocated_height())
                .visible(true)
                .has_alpha(true)
                .build();

            fixed.put(&area, 0, 0);
            area.grab_focus();
            frame.dynamic_resize();
            (fixed, area)
        };

        gl_loader::init_gl();
        area.make_current();
        #[cfg(window_system_pgtk)]
        return Self { area, fixed };
        #[cfg(use_tao)]
        return Self { area };
    }

    fn bind_framebuffer(&mut self, _gl: &mut Rc<dyn Gl>) {
        self.area.attach_buffers();
    }

    #[inline]
    fn swap_buffers(&self) {
        // GTK swaps the buffers after each "render" signal itself
        self.area.queue_render();
    }

    // Ignored because widget will be resized automatically
    fn resize(&self, _size: &DeviceIntSize) {
        #[cfg(window_system_pgtk)]
        {
            let p_alloc = self.fixed.allocation();
            let me_alloc = self.area.allocation();
            let allocation = gtk::Allocation::new(
                me_alloc.x(),
                me_alloc.y(),
                p_alloc.width(),
                p_alloc.height(),
            );
            log::debug!("gl_area allocation {allocation:?} to match fixed parent {p_alloc:?}.");
            self.area.size_allocate(&allocation);
            self.fixed.move_(&self.area, 0, 0);
        }
    }

    fn ensure_is_current(&mut self) {
        self.area.make_current();
    }

    fn load_gl(&self) -> Rc<dyn Gl> {
        let gl = match self.gl_api() {
            GLApi::GL => unsafe { GlFns::load_with(|s| self.get_proc_address(s)) },
            GLApi::GLES => unsafe { GlesFns::load_with(|s| self.get_proc_address(s)) },
        };
        ErrorCheckingGl::wrap(gl)
    }
}
