use super::cursor::build_mouse_cursors;
use super::cursor::emacs_to_winit_cursor;
use crate::event_loop::WrEventLoop;
use crate::frame::LispFrameExt;
use crate::output::OutputRef;
use crate::window_system::api::dpi::LogicalPosition;
use crate::window_system::api::dpi::PhysicalSize;
use crate::window_system::api::monitor::MonitorHandle;
use emacs::globals::Qfullscreen;
use emacs::globals::Qmaximized;
use emacs::{
    bindings::{
        fullscreen_type, list4i, make_frame, make_frame_without_minibuffer, make_minibuffer_frame,
        output_method, winit_output, Emacs_Cursor,
    },
    frame::LispFrameRef,
    globals::{Qinner_edges, Qnil, Qnone, Qonly, Qouter_edges},
    keyboard::KeyboardRef,
    lisp::LispObject,
};
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use webrender::api::ColorF;

use crate::frame::LispFrameWindowSystemExt;
use crate::output::Output;

use crate::window_system::api::{dpi::PhysicalPosition, window::WindowBuilder};

use crate::display_info::DisplayInfoRef;

pub type FrameId = crate::window_system::api::window::WindowId;

pub trait LispFrameWinitExt {
    fn build(
        display: LispObject,
        dpyinfo: DisplayInfoRef,
        tem: LispObject,
        kb: KeyboardRef,
    ) -> Self;
    fn set_window(&self, handle: crate::window_system::api::window::Window);
    fn set_inner_size(&self, size: PhysicalSize<u32>);
    fn set_cursor_color(&self, color: ColorF);
    fn set_background_color(&self, color: ColorF);
    fn set_cursor_position(&self, pos: PhysicalPosition<f64>);
    fn set_visible_(&mut self, visible: bool);
    fn set_cursor_icon(&self, cursor: Emacs_Cursor);
    fn edges(&self, type_: LispObject) -> LispObject;
    fn fullscreen(&self);
    fn implicitly_set_name(&mut self, arg: LispObject, _old_val: LispObject);
    fn iconify(&mut self);
    fn current_monitor(&self) -> Option<MonitorHandle>;
    fn cursor_position(&self) -> LogicalPosition<i32>;
    fn winit_scale_factor(&self) -> f64;
}

impl LispFrameWindowSystemExt for LispFrameRef {
    fn output(&self) -> OutputRef {
        return OutputRef::new(unsafe { self.output_data.winit } as *mut Output);
    }

    fn cursor_color(&self) -> ColorF {
        self.output().inner().cursor_color
    }

    fn scale_factor(&self) -> f64 {
        self.output().inner().scale_factor
    }

    fn set_scale_factor(&mut self, scale_factor: f64) -> bool {
        if self.output().inner().scale_factor != scale_factor {
            self.output().inner().scale_factor = scale_factor;
            return true;
        }
        false
    }

    fn cursor_foreground_color(&self) -> ColorF {
        self.output().inner().cursor_foreground_color
    }

    fn window_handle(&self) -> Option<RawWindowHandle> {
        if let Some(window) = &self.output().inner().window {
            use raw_window_handle::HasRawWindowHandle;
            return Some(window.raw_window_handle());
        } else {
            return None;
        }
    }

    fn display_handle(&self) -> Option<RawDisplayHandle> {
        return self.output().display_info().get_inner().raw_display_handle;
    }

    fn unique_id(&self) -> FrameId {
        self.output()
            .inner()
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet")
            .id()
            .clone()
    }
}

impl LispFrameWinitExt for LispFrameRef {
    fn build(
        display: LispObject,
        mut dpyinfo: DisplayInfoRef,
        tem: LispObject,
        mut kb: KeyboardRef,
    ) -> Self {
        log::trace!("Winit creating new frame");
        let frame = if tem.eq(Qnone) || tem.is_nil() {
            unsafe { make_frame_without_minibuffer(Qnil, kb.as_mut(), display) }
        } else if tem.eq(Qonly) {
            unsafe { make_minibuffer_frame() }
        } else if tem.is_window() {
            unsafe { make_frame_without_minibuffer(tem, kb.as_mut(), display) }
        } else {
            unsafe { make_frame(true) }
        };

        let mut frame = LispFrameRef::new(frame);

        frame.terminal = dpyinfo.get_inner().terminal.as_mut();
        frame.set_output_method(output_method::output_winit);

        let event_loop = WrEventLoop::global().try_lock().unwrap();
        let window_builder = WindowBuilder::new().with_visible(true);
        let primary_monitor = event_loop.get_primary_monitor();
        let scale_factor = primary_monitor.scale_factor();

        let invocation_name: String = unsafe { emacs::bindings::globals.Vinvocation_name.into() };

        #[cfg(all(wayland_platform, use_winit))]
        let window_builder = {
            use crate::window_system::api::platform::wayland::WindowBuilderExtWayland;
            window_builder.with_name(&invocation_name, "")
        };
        #[cfg(use_tao)]
        let window_builder = window_builder.with_title(invocation_name);

        let window = window_builder.build(&event_loop.el()).unwrap();
        #[cfg(use_winit)]
        window.set_theme(None);
        #[cfg(use_winit)]
        window.set_title(&invocation_name);
        let mut output = Box::new(Output::default());
        build_mouse_cursors(&mut output.as_mut().as_raw());

        // TODO default frame size?
        log::trace!("frame total_cols {:?}", frame.total_cols);
        log::trace!("frame line_height {:?}", frame.line_height);

        frame.pixel_width = (window.inner_size().width as f64 / scale_factor).round() as i32;
        frame.pixel_height = (window.inner_size().height as f64 / scale_factor).round() as i32;

        // Remeber to destory the Output object when frame destoried.
        let output = Box::into_raw(output);
        frame.output_data.winit = output as *mut winit_output;
        frame.set_display_info(dpyinfo);

        frame.set_window(window);
        frame.set_scale_factor(scale_factor);
        dpyinfo.get_inner().frames.insert(frame.unique_id(), frame);
        log::trace!("create_frame done");
        frame
    }

    fn set_window(&self, window: crate::window_system::api::window::Window) {
        self.output().inner().set_window(window);
    }

    fn set_inner_size(&self, size: PhysicalSize<u32>) {
        if let Some(ref window) = self.output().inner().window {
            window.set_inner_size(size);
            self.canvas().update();
        }
    }

    fn set_cursor_position(&self, pos: PhysicalPosition<f64>) {
        self.output().inner().set_cursor_position(pos);
    }

    fn set_cursor_color(&self, color: ColorF) {
        self.output().inner().set_cursor_color(color);
    }

    fn set_background_color(&self, color: ColorF) {
        self.output().inner().set_background_color(color);
    }

    fn set_visible_(&mut self, is_visible: bool) {
        let _ = &self.set_visible(is_visible as u32);

        let inner = self.output().inner();
        let window = inner
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet");

        if is_visible {
            window.set_visible(true);
        } else {
            window.set_visible(false);
        }
    }

    fn set_cursor_icon(&self, cursor: Emacs_Cursor) {
        let inner = self.output().inner();
        let window = inner
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet");
        let cursor = emacs_to_winit_cursor(cursor);
        window.set_cursor_icon(cursor);
    }

    fn edges(&self, type_: LispObject) -> LispObject {
        let inner = self.output().inner();
        let window = inner
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet");

        let (left, top, right, bottom) = match type_ {
            Qouter_edges => {
                let pos = window
                    .outer_position()
                    .unwrap_or_else(|_| PhysicalPosition::<i32>::new(0, 0));

                let size = window.outer_size();

                let left = pos.x;
                let top = pos.y;
                let right = left + size.width as i32;
                let bottom = top + size.height as i32;

                (left, top, right, bottom)
            }
            Qinner_edges => {
                let pos = window
                    .inner_position()
                    .unwrap_or_else(|_| PhysicalPosition::<i32>::new(0, 0));
                let size = window.inner_size();
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
                let pos = window
                    .inner_position()
                    .unwrap_or_else(|_| PhysicalPosition::<i32>::new(0, 0));
                let size = window.inner_size();

                let left = pos.x;
                let top = pos.y;
                let right = left + size.width as i32;
                let bottom = top + size.height as i32;

                (left, top, right, bottom)
            }
        };
        unsafe { list4i(left as i64, top as i64, right as i64, bottom as i64) }
    }

    fn fullscreen(&self) {
        if !self.is_visible() {
            return;
        }

        if self.want_fullscreen() == fullscreen_type::FULLSCREEN_MAXIMIZED {
            let inner = self.output().inner();
            let window = inner
                .window
                .as_ref()
                .expect("frame doesnt have associated winit window yet");
            window.set_maximized(true);
            self.store_param(Qfullscreen, Qmaximized);
        }
    }
    fn implicitly_set_name(&mut self, arg: LispObject, _old_val: LispObject) {
        if self.name.eq(arg) {
            return;
        }

        self.name = arg;

        let title = format!("{}", arg.force_string());
        let inner = self.output().inner();
        let window = inner
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet");

        window.set_title(&title);
    }

    fn iconify(&mut self) {
        self.set_iconified(true);
        let inner = self.output().inner();
        let window = inner
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet");
        window.set_visible(false);
    }

    fn current_monitor(&self) -> Option<MonitorHandle> {
        let inner = self.output().inner();
        let window = inner
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet");
        window.current_monitor()
    }

    fn cursor_position(&self) -> LogicalPosition<i32> {
        let pos = self.output().inner().cursor_position;
        LogicalPosition::new(
            (pos.x / self.scale_factor()).round() as i32,
            (pos.y / self.scale_factor()).round() as i32,
        )
    }

    fn winit_scale_factor(&self) -> f64 {
        if let Some(monitor) = self.current_monitor() {
            return monitor.scale_factor();
        }

        if let Some(ref window) = self.output().inner().window {
            return window.scale_factor();
        }

        1.0
    }
}
