use crate::bindings::gl_renderer_fit_context;
use crate::frame::Frame;
use crate::frame::FrameRef;
use crate::gfx::context::GLContextTrait;
use crate::window_system::FrameExtPgtk;
use gleam::gl::ErrorCheckingGl;
use gleam::gl::Gl;
use gleam::gl::GlFns;
use gleam::gl::GlesFns;
use gtk::glib::translate::ToGlibPtr;
use gtk::prelude::*;
use gtk::GLArea;
use std::rc::Rc;
use webrender_api::units::DeviceIntSize;

pub struct ContextImpl {
    area: GLArea,
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
    fn build(frame: &FrameRef) -> Self {
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

            fixed.connect_size_allocate({
                move |widget, allocation| {
                    let scale_factor = widget.scale_factor() as f64;
                    let frame = fixed_wiget_to_frame(widget);
                    let size =
                        DeviceIntSize::new(allocation.width() as i32, allocation.height() as i32);
                    log::debug!("Gtk fixed size allocated {size:?} scale_factor: {scale_factor:?}");

                    unsafe { gl_renderer_fit_context(frame) };
                }
            });

            fixed.connect_scale_factor_notify(move |widget| {
                let frame = fixed_wiget_to_frame(widget);
                let scale_factor = widget.scale_factor() as f64;
                log::debug!("Gtk fixed scale_factor: {scale_factor:?}");
                unsafe { gl_renderer_fit_context(frame) };
            });

            (fixed, area)
        };

        gl_loader::init_gl();
        area.make_current();
        return Self { area, fixed };
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

fn fixed_wiget_to_frame(widget: &gtk::Fixed) -> *mut Frame {
    let widget: *mut gtk_sys::GtkWidget = <gtk::Fixed as AsRef<gtk::Widget>>::as_ref(widget)
        .to_glib_none()
        .0;
    unsafe { crate::bindings::pgtk_fixed_to_frame(widget) }
}
