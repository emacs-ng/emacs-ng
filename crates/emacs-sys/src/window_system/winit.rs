use crate::bindings::fullscreen_type;
use crate::display_traits::FrameParam;
use crate::frame::FrameRef;
use crate::lisp::ExternalPtr;
use crate::terminal::TerminalRef;
use arboard::Clipboard;
use raw_window_handle::HasDisplayHandle;
use raw_window_handle::HasWindowHandle;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use webrender_api::ColorF;
use winit::dpi::PhysicalPosition;
use winit::dpi::PhysicalSize;
use winit::dpi::Position;
use winit::event::Event;
use winit::event_loop::DeviceEvents;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopBuilder;
use winit::monitor::MonitorHandle;
use winit::window::Fullscreen;
use winit::window::WindowBuilder;
use winit::window::WindowButtons;
use winit::window::WindowLevel;

use std::ptr;

pub struct WinitFrameData {
    pub background_color: ColorF,
    pub cursor_color: ColorF,
    pub cursor_foreground_color: ColorF,
    pub window: Option<winit::window::Window>,
    pub cursor_position: winit::dpi::PhysicalPosition<f64>,
}

impl Default for WinitFrameData {
    fn default() -> Self {
        WinitFrameData {
            background_color: ColorF::WHITE,
            cursor_color: ColorF::BLACK,
            cursor_foreground_color: ColorF::WHITE,
            window: None,
            cursor_position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
        }
    }
}

pub type WinitFrameDataRef = ExternalPtr<WinitFrameData>;

pub struct WinitTermData {
    pub terminal: TerminalRef,
    pub focus_frame: FrameRef,
    pub clipboard: Clipboard,
    pub event_loop: EventLoop<i32>,
    pub pending_events: Vec<Event<i32>>,
}

pub fn current_winit_data() -> Option<WinitTermDataRef> {
    crate::frame::all_frames()
        .find(|f| f.is_current_window_system())
        .and_then(|f| f.terminal().winit_data())
}

impl Default for WinitTermData {
    fn default() -> Self {
        let event_loop = EventLoopBuilder::<i32>::with_user_event()
            .build()
            .ok()
            .unwrap();
        event_loop.listen_device_events(DeviceEvents::Never);
        let clipboard = Clipboard::new().unwrap();
        WinitTermData {
            terminal: TerminalRef::new(ptr::null_mut()),
            focus_frame: FrameRef::new(ptr::null_mut()),
            event_loop,
            clipboard,
            pending_events: Vec::new(),
        }
    }
}

pub type WinitTermDataRef = ExternalPtr<WinitTermData>;

impl TerminalRef {
    pub fn init_winit_data(&mut self) {
        assert_eq!(!self.is_null(), true);
        assert_eq!(self.winit.is_null(), true);
        let winit_data = Box::new(WinitTermData::default());
        self.winit = Box::into_raw(winit_data) as *mut libc::c_void;
    }

    pub fn winit_data(&self) -> Option<WinitTermDataRef> {
        if self.is_null() || self.winit.is_null() {
            return None;
        }
        Some(WinitTermDataRef::new(self.winit as *mut WinitTermData))
    }

    pub fn free_winit_data(&mut self) {
        assert_eq!(!self.is_null(), true);
        if self.winit != ptr::null_mut() {
            unsafe {
                let _ = Box::from_raw(self.winit as *mut WinitTermData);
            }
        }
    }

    pub fn available_monitors(&self) -> Option<impl Iterator<Item = MonitorHandle>> {
        Some(self.winit_data()?.event_loop.available_monitors())
    }

    pub fn primary_monitor(&self) -> Option<MonitorHandle> {
        self.winit_data()?.event_loop.primary_monitor()
    }

    pub fn get_color_bits(&self) -> u8 {
        24
    }

    pub fn raw_display_handle(&self) -> Option<RawDisplayHandle> {
        Some(
            self.winit_data()?
                .event_loop
                .display_handle()
                .ok()?
                .as_raw(),
        )
    }
}

impl FrameRef {
    // Using frame winit window display handle, fallback to terminal display handle
    pub fn raw_display_handle(&self) -> Option<RawDisplayHandle> {
        fn frame_display_handle(f: &FrameRef) -> Option<RawDisplayHandle> {
            Some(
                f.winit_data()?
                    .window
                    .as_ref()?
                    .display_handle()
                    .ok()?
                    .as_raw(),
            )
        }
        frame_display_handle(self).or_else(|| self.terminal().raw_display_handle())
    }

    pub fn raw_window_handle(&self) -> Option<RawWindowHandle> {
        self.winit_data().and_then(|data| {
            data.window
                .as_ref()
                .and_then(|w| w.window_handle().map(|handle| handle.as_raw()).ok())
        })
    }

    pub fn free_winit_data(self) {
        let _ = self
            .winit_data()
            .map(|mut d| unsafe { Box::from_raw(d.as_mut()) });

        self.output().winit = ptr::null_mut();
    }

    pub fn init_winit_data(self) {
        assert_eq!(!self.is_null(), true);
        assert_eq!(!self.output().is_null(), true);
        assert_eq!(self.output().winit.is_null(), true);
        let data = Box::new(WinitFrameData::default());
        self.output().winit = Box::into_raw(data) as *mut libc::c_void;
    }

    pub fn winit_data(&self) -> Option<WinitFrameDataRef> {
        if self.is_null() || self.output().is_null() {
            return None;
        }

        Some(WinitFrameDataRef::new(
            self.output().winit as *mut WinitFrameData,
        ))
    }

    pub fn cursor_color(&self) -> ColorF {
        self.winit_data()
            .and_then(|data| Some(data.cursor_color))
            .unwrap_or(ColorF::BLACK)
    }

    // This value may differ from MonitorHandle::scale_factor.
    pub fn scale_factor(&self) -> f64 {
        self.winit_data()
            .and_then(|d| d.window.as_ref().and_then(|w| Some(w.scale_factor())))
            .or_else(|| self.current_monitor().and_then(|m| Some(m.scale_factor())))
            .unwrap_or(1.0)
    }

    pub fn cursor_foreground_color(&self) -> ColorF {
        self.winit_data()
            .and_then(|data| Some(data.cursor_foreground_color))
            .unwrap_or(ColorF::WHITE)
    }

    pub fn current_monitor(&self) -> Option<MonitorHandle> {
        self.winit_data()
            .and_then(|data| data.window.as_ref().and_then(|w| w.current_monitor()))
    }

    pub fn fullscreen(&self) -> Option<Fullscreen> {
        match self.want_fullscreen() {
            fullscreen_type::FULLSCREEN_BOTH => {
                // TODO set fullscreen on other available_monitors
                Some(Fullscreen::Borderless(self.current_monitor()))
            }
            fullscreen_type::FULLSCREEN_EXCLUSIVE => {
                // TODO set fullscreen on other available_monitors
                if let Some(monitor_handle) = self.current_monitor() {
                    if let Some(mode) = monitor_handle.video_modes().next() {
                        return Some(Fullscreen::Exclusive(mode.clone()));
                    }

                    return None;
                }
                None
            }
            _ => None,
        }
    }

    pub fn maximized(&self) -> bool {
        match self.want_fullscreen() {
            fullscreen_type::FULLSCREEN_MAXIMIZED => true,
            fullscreen_type::FULLSCREEN_WIDTH | fullscreen_type::FULLSCREEN_HEIGHT => {
                message!("Winit currently not support fullscreen width");
                false
            }
            _ => false,
        }
    }

    pub fn parent_frame_handle(&self) -> Option<RawWindowHandle> {
        if self.parent_frame.is_nil() {
            return None;
        }

        let parent_frame = FrameRef::from(self.parent_frame);
        parent_frame.raw_window_handle()
    }
}

impl From<FrameRef> for WindowBuilder {
    fn from(f: FrameRef) -> WindowBuilder {
        let mut builder = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(f.pixel_width, f.pixel_height))
            .with_min_inner_size(PhysicalSize::<u32>::new(
                f.param(FrameParam::MinWidth).into(),
                f.param(FrameParam::MinHeight).into(),
            ))
            // .with_max_inner_size(_)
            .with_position(Position::Physical(PhysicalPosition::new(
                f.left_pos, f.top_pos,
            )))
            .with_resizable(true)
            .with_enabled_buttons(WindowButtons::all())
            .with_title(f.name)
            .with_fullscreen(f.fullscreen())
            .with_maximized(f.maximized())
            .with_visible(f.visible() != 0)
            .with_transparent(true) // TODO
            .with_blur(true) //TODO
            .with_decorations(!f.undecorated())
            .with_window_level(WindowLevel::Normal) // TODO
            .with_window_icon(None) //TODO
            .with_theme(None) //TODO
            // .with_resize_increments(_) //TODO
            .with_content_protected(false) //TODO only works on macOS
            .with_active(true); //TODO
                                // .with_cursor(CursorIcon::Default) //TODO

        builder = unsafe { builder.with_parent_window(f.parent_frame_handle()) };
        // startup notify
        // .with_activation_token()

        #[cfg(wayland_platform)]
        {
            use winit::platform::wayland::WindowBuilderExtWayland;
            builder = builder.with_name(f.name, f.name)
        }

        #[cfg(x11_platform)]
        {
            use winit::platform::x11::WindowBuilderExtX11;
            builder = builder
                .with_name(f.name, f.name)
                .with_override_redirect(f.override_redirect());
            // .with_x11_visual()
            // .with_x11_screen()
            // .with_x11_window_type()
            // .with_base_size()
            // .with_embed_parent_window()
        };
        builder
    }
}
