use crate::cursor::build_mouse_cursors;
use crate::cursor::emacs_to_winit_cursor;
use emacs::bindings::do_pending_window_change;
use emacs::bindings::fullscreen_type;
use emacs::bindings::gl_renderer_fit_context;
use emacs::bindings::list4i;
use emacs::bindings::make_frame;
use emacs::bindings::make_frame_without_minibuffer;
use emacs::bindings::make_minibuffer_frame;
use emacs::bindings::output_method;
use emacs::bindings::winit_output;
use emacs::bindings::Emacs_Cursor;
use emacs::frame::FrameRef;
use emacs::globals::Qfullboth;
use emacs::globals::Qfullexclusive;
use emacs::globals::Qfullscreen;
use emacs::globals::Qinner_edges;
use emacs::globals::Qmaximized;
use emacs::globals::Qnil;
use emacs::globals::Qnone;
use emacs::globals::Qonly;
use emacs::globals::Qouter_edges;
use emacs::keyboard::KeyboardRef;
use emacs::lisp::LispObject;
use emacs::terminal::TerminalRef;
use webrender_api::units::*;
use webrender_api::ColorF;
use winit::dpi::LogicalPosition;
use winit::dpi::PhysicalSize;
use winit::monitor::MonitorHandle;

use winit::dpi::PhysicalPosition;
use winit::window::WindowBuilder;

use emacs::display_info::DisplayInfoRef;
use emacs::output::Output;

pub trait FrameExtWinit {
    fn build(
        display: LispObject,
        dpyinfo: DisplayInfoRef,
        tem: LispObject,
        kb: KeyboardRef,
    ) -> Self;
    fn set_window(&self, handle: winit::window::Window);
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
    fn current_monitor(&self) -> Option<MonitorHandle>;
    fn cursor_position(&self) -> LogicalPosition<i32>;
    fn winit_scale_factor(&self) -> f64;
    fn handle_size_change(&mut self, size: DeviceIntSize, scale_factor: f64);
    fn handle_scale_factor_change(&mut self, _scale_factor: f64);
}

impl FrameExtWinit for FrameRef {
    fn build(
        display: LispObject,
        dpyinfo: DisplayInfoRef,
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

        let mut frame = FrameRef::new(frame);

        frame.terminal = dpyinfo.terminal;
        frame.set_output_method(output_method::output_winit);

        let mut terminal = TerminalRef::new(dpyinfo.terminal);

        let event_loop = &terminal.winit_term_data().event_loop;
        let window_builder = WindowBuilder::new().with_visible(true);
        let primary_monitor = terminal.primary_monitor();
        let scale_factor = primary_monitor.scale_factor();

        let invocation_name: String = unsafe { emacs::bindings::globals.Vinvocation_name.into() };

        #[cfg(free_unix)]
        let window_builder = {
            use winit::platform::wayland::WindowBuilderExtWayland;
            window_builder.with_name(&invocation_name, "")
        };

        let window = window_builder.build(&event_loop).unwrap();
        window.set_theme(None);
        window.set_title(&invocation_name);
        let mut output = Box::new(Output::default());
        build_mouse_cursors(&mut output.as_mut());

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
        terminal.winit_term_data().all_frames.push(frame);
        log::trace!("create_frame done");
        frame
    }

    fn set_window(&self, window: winit::window::Window) {
        self.output().winit_term_data().set_window(window);
    }

    fn set_inner_size(&self, size: PhysicalSize<u32>) {
        if let Some(ref window) = self.output().winit_term_data().window {
            let _ = window.request_inner_size(size);
        }
    }

    fn set_cursor_position(&self, pos: PhysicalPosition<f64>) {
        self.output().winit_term_data().set_cursor_position(pos);
    }

    fn set_cursor_color(&self, color: ColorF) {
        self.output().winit_term_data().set_cursor_color(color);
    }

    fn set_background_color(&self, color: ColorF) {
        self.output().winit_term_data().set_background_color(color);
    }

    fn set_visible_(&mut self, is_visible: bool) {
        let _ = &self.set_visible(is_visible as u32);

        let data = self.output().winit_term_data();
        let window = data
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
        let data = self.output().winit_term_data();
        let window = data
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet");
        let cursor = emacs_to_winit_cursor(cursor);
        window.set_cursor_icon(cursor);
    }

    fn edges(&self, type_: LispObject) -> LispObject {
        let data = self.output().winit_term_data();
        let window = data
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

    fn set_fullscreen(&self) {
        if !self.is_visible() {
            return;
        }

        let data = self.output().winit_term_data();
        let window = data
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet");

        use winit::window::Fullscreen;

        match self.want_fullscreen() {
            fullscreen_type::FULLSCREEN_MAXIMIZED => {
                window.set_fullscreen(None);
                window.set_maximized(true);
                self.store_param(Qfullscreen, Qmaximized);
            }
            fullscreen_type::FULLSCREEN_WIDTH => {
                error!("Winit currently not support fullscreen width");
            }
            fullscreen_type::FULLSCREEN_HEIGHT => {
                error!("Winit currently not support fullscreen height");
            }
            fullscreen_type::FULLSCREEN_BOTH => {
                // TODO set fullscreen on other available_monitors
                window.set_maximized(false);
                window.set_fullscreen(Some(Fullscreen::Borderless(self.current_monitor())));
                message!("Set borderless fullscreen using winit on current monitor");
                self.store_param(Qfullscreen, Qfullboth);
            }
            fullscreen_type::FULLSCREEN_EXCLUSIVE => {
                // TODO set fullscreen on other available_monitors
                if let Some(monitor_handle) = self.current_monitor() {
                    if let Some(mode) = monitor_handle.video_modes().next() {
                        message!("Set fullscreen on current monitor with video mode {}", mode);
                        let fullscreen = Some(Fullscreen::Exclusive(mode.clone()));
                        window.set_fullscreen(fullscreen);
                        self.store_param(Qfullscreen, Qfullexclusive);
                        return;
                    }

                    error!("No video mode found on current monitor");
                }
                error!("Current monitor not found");
            }
            fullscreen_type::FULLSCREEN_NONE => {
                window.set_maximized(false);
                window.set_fullscreen(None);
                self.store_param(Qfullscreen, Qnil);
            }
            _ => {}
        }
    }
    fn implicitly_set_name(&mut self, arg: LispObject, _old_val: LispObject) {
        if self.name.eq(arg) {
            return;
        }

        self.name = arg;

        let title = format!("{}", arg.force_string());
        let data = self.output().winit_term_data();
        let window = data
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet");

        window.set_title(&title);
    }

    fn iconify(&mut self) {
        self.set_iconified(true);
        let data = self.output().winit_term_data();
        let window = data
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet");
        window.set_visible(false);
    }

    fn current_monitor(&self) -> Option<MonitorHandle> {
        let data = self.output().winit_term_data();
        let window = data
            .window
            .as_ref()
            .expect("frame doesnt have associated winit window yet");
        window.current_monitor()
    }

    fn cursor_position(&self) -> LogicalPosition<i32> {
        let pos = self.output().winit_term_data().cursor_position;
        LogicalPosition::new(
            (pos.x / self.winit_scale_factor()).round() as i32,
            (pos.y / self.winit_scale_factor()).round() as i32,
        )
    }

    fn winit_scale_factor(&self) -> f64 {
        if let Some(monitor) = self.current_monitor() {
            return monitor.scale_factor();
        }

        if let Some(ref window) = self.output().winit_term_data().window {
            return window.scale_factor();
        }

        1.0
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
