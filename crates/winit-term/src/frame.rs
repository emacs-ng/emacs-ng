use crate::cursor::emacs_to_winit_cursor;
use emacs_sys::bindings::do_pending_window_change;
use emacs_sys::bindings::gl_renderer_fit_context;
use emacs_sys::bindings::list4i;
use emacs_sys::bindings::Emacs_Cursor;
use emacs_sys::frame::FrameRef;
use emacs_sys::globals::Qfullboth;
use emacs_sys::globals::Qfullexclusive;
use emacs_sys::globals::Qfullscreen;
use emacs_sys::globals::Qinner_edges;
use emacs_sys::globals::Qmaximized;
use emacs_sys::globals::Qnil;
use emacs_sys::globals::Qouter_edges;
use emacs_sys::lisp::LispObject;

use webrender_api::units::*;
use webrender_api::ColorF;
use winit::dpi::LogicalPosition;
use winit::dpi::PhysicalSize;
use winit::window::Fullscreen;
use winit::window::Window;

use winit::dpi::PhysicalPosition;
use winit::window::WindowBuilder;

pub trait FrameExtWinit {
    fn setup_winit(&mut self);
    fn set_winit_window(&self, handle: winit::window::Window);
    fn set_inner_size(&self, size: PhysicalSize<u32>);
    fn set_cursor_color(&self, color: ColorF);
    fn set_background_color(&self, color: ColorF);
    fn set_cursor_position(&self, pos: PhysicalPosition<f64>);
    fn set_visible_(&mut self, visible: bool);
    fn set_cursor_icon(&self, cursor: Emacs_Cursor);
    fn edges(&self, type_: LispObject) -> LispObject;
    fn set_fullscreen(&self);
    fn implicitly_set_name(&mut self, arg: LispObject, _old_val: LispObject);
    fn iconify(&mut self);
    fn cursor_position(&self) -> LogicalPosition<i32>;
    fn handle_size_change(&mut self, size: DeviceIntSize, scale_factor: f64);
    fn handle_scale_factor_change(&mut self, _scale_factor: f64);
}

impl FrameExtWinit for FrameRef {
    fn setup_winit(&mut self) {
        let terminal = self.terminal();

        let window_builder = WindowBuilder::from(self.clone());

        let scale_factor = self.scale_factor();

        let invocation_name: String =
            unsafe { emacs_sys::bindings::globals.Vinvocation_name.into() };

        #[cfg(free_unix)]
        let window_builder = {
            use winit::platform::wayland::WindowBuilderExtWayland;
            window_builder.with_name(&invocation_name, "")
        };

        let window = terminal
            .winit_data()
            .and_then(|d| window_builder.build(&d.event_loop).ok())
            .unwrap();
        window.set_theme(None);
        window.set_title(&invocation_name);

        self.pixel_width = (window.inner_size().width as f64 / scale_factor).round() as i32;
        self.pixel_height = (window.inner_size().height as f64 / scale_factor).round() as i32;

        self.set_winit_window(window);
    }

    fn set_winit_window(&self, window: winit::window::Window) {
        self.winit_data().map(|mut d| d.window = Some(window));
    }

    fn set_inner_size(&self, size: PhysicalSize<u32>) {
        self.winit_data()
            .map(|d| d.window.as_ref().map(|w| w.request_inner_size(size)));
    }

    fn set_cursor_position(&self, pos: PhysicalPosition<f64>) {
        self.winit_data().map(|mut d| d.cursor_position = pos);
    }

    fn set_cursor_color(&self, color: ColorF) {
        self.winit_data().map(|mut d| d.cursor_color = color);
    }

    fn set_background_color(&self, color: ColorF) {
        self.winit_data().map(|mut d| d.background_color = color);
    }

    fn set_visible_(&mut self, is_visible: bool) {
        let _ = &self.set_visible(is_visible as u32);
        self.winit_data()
            .map(|d| d.window.as_ref().map(|w| w.set_visible(is_visible)));
    }

    fn set_cursor_icon(&self, cursor: Emacs_Cursor) {
        self.winit_data().map(|d| {
            d.window.as_ref().map(|w| {
                let cursor = emacs_to_winit_cursor(cursor);
                w.set_cursor_icon(cursor)
            })
        });
    }

    fn edges(&self, type_: LispObject) -> LispObject {
        let window_boundary = |w: &Window| {
            let size = w.outer_size();
            match type_ {
                Qouter_edges => {
                    let pos = w
                        .outer_position()
                        .unwrap_or_else(|_| PhysicalPosition::<i32>::new(0, 0));

                    let left = pos.x;
                    let top = pos.y;
                    let right = left + size.width as i32;
                    let bottom = top + size.height as i32;

                    (left, top, right, bottom)
                }
                Qinner_edges => {
                    let pos = w
                        .inner_position()
                        .unwrap_or_else(|_| PhysicalPosition::<i32>::new(0, 0));
                    let internal_border_width = self.internal_border_width();

                    // webrender window has no interanl menu_bar, tab_bar and tool_bar
                    let left = pos.x + internal_border_width;
                    let top = pos.x + internal_border_width;
                    let right = (left + size.width as i32) - internal_border_width;
                    let bottom = (top + size.height as i32) - internal_border_width;

                    (left, top, right, bottom)
                }
                // native edges
                _ => {
                    let pos = w
                        .inner_position()
                        .unwrap_or_else(|_| PhysicalPosition::<i32>::new(0, 0));

                    let left = pos.x;
                    let top = pos.y;
                    let right = left + size.width as i32;
                    let bottom = top + size.height as i32;

                    (left, top, right, bottom)
                }
            }
        };
        self.winit_data()
            .and_then(|d| {
                d.window.as_ref().and_then(|w| {
                    let (left, top, right, bottom) = window_boundary(w);
                    let edges =
                        unsafe { list4i(left as i64, top as i64, right as i64, bottom as i64) };
                    Some(edges)
                })
            })
            .unwrap_or(Qnil)
    }

    fn set_fullscreen(&self) {
        let fullscreen = self.fullscreen();
        let maximized = self.maximized();
        let param = match (&fullscreen, maximized) {
            (Some(f), _) => match f {
                Fullscreen::Borderless(_) => Qfullboth,
                Fullscreen::Exclusive(_) => Qfullexclusive,
            },
            (None, true) => Qmaximized,
            (_, _) => Qnil,
        };

        let _ = self.winit_data().and_then(|d| {
            d.window.as_ref().map(|w| {
                w.set_fullscreen(fullscreen);
                w.set_maximized(maximized);
            })
        });
        self.store_param(Qfullscreen, param);
    }
    fn implicitly_set_name(&mut self, arg: LispObject, _old_val: LispObject) {
        if self.name.eq(arg) {
            return;
        }

        self.name = arg;

        let title = format!("{}", arg.force_string());
        self.winit_data()
            .map(|d| d.window.as_ref().map(|w| w.set_title(&title)));
    }

    //(i.e. minimize)
    fn iconify(&mut self) {
        self.set_iconified(true);
        let _ = self
            .winit_data()
            .map(|d| d.window.as_ref().map(|w| w.set_minimized(true)));
    }

    fn cursor_position(&self) -> LogicalPosition<i32> {
        self.winit_data()
            .and_then(|d| Some(d.cursor_position.to_logical::<i32>(self.scale_factor())))
            .unwrap_or(LogicalPosition::new(0, 0))
    }

    fn handle_size_change(&mut self, size: DeviceIntSize, _scale_factor: f64) {
        log::trace!("frame handle_size_change: {size:?}");
        self.change_size(
            size.width as i32,
            size.height as i32 - self.menu_bar_height,
            false,
            true,
            false,
        );

        unsafe { do_pending_window_change(false) };
        unsafe { gl_renderer_fit_context(self.as_mut()) };
    }

    fn handle_scale_factor_change(&mut self, scale_factor: f64) {
        log::trace!("frame handle_scale_factor_change... {scale_factor:?}");
        unsafe { gl_renderer_fit_context(self.as_mut()) };
    }
}
