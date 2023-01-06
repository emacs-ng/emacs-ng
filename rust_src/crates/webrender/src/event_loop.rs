use crate::select::handle_select;
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

#[cfg(macos_platform)]
use copypasta::osx_clipboard::OSXClipboardContext;
#[cfg(windows_platform)]
use copypasta::windows_clipboard::WindowsClipboardContext;
use copypasta::ClipboardProvider;
#[cfg(free_unix)]
use copypasta::{
    wayland_clipboard::create_clipboards_from_external,
    x11_clipboard::{Clipboard, X11ClipboardContext},
};

use libc::{c_void, fd_set, pselect, sigset_t, timespec};
use once_cell::sync::Lazy;
#[cfg(wayland_platform)]
use winit::platform::wayland::EventLoopWindowTargetExtWayland;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    monitor::MonitorHandle,
    platform::run_return::EventLoopExtRunReturn,
    window::Window,
    window::WindowId,
};

use surfman::Connection;
use surfman::SurfaceType;
use webrender_surfman::WebrenderSurfman;

use emacs::bindings::{inhibit_window_system, thread_select};

pub type GUIEvent = Event<'static, i32>;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Platform {
    X11,
    Wayland(*mut c_void),
    MacOS,
    Windows,
}

unsafe impl Send for Platform {}

pub struct WrEventLoop {
    clipboard: Box<dyn ClipboardProvider>,
    pub el: EventLoop<i32>,
    pub connection: Option<Connection>,
}

unsafe impl Send for WrEventLoop {}
unsafe impl Sync for WrEventLoop {}

impl WrEventLoop {
    pub fn el(&self) -> &EventLoop<i32> {
        &self.el
    }

    pub fn connection(&mut self) -> &Connection {
        if self.connection.is_none() {
            self.open_native_display();
        }
        self.connection.as_ref().unwrap()
    }

    pub fn create_proxy(&self) -> EventLoopProxy<i32> {
        self.el.create_proxy()
    }

    pub fn new_webrender_surfman(&mut self, window: &Window) -> WebrenderSurfman {
        let connection = self.connection();
        let adapter = connection
            .create_adapter()
            .expect("Failed to create adapter");
        let native_widget = connection
            .create_native_widget_from_winit_window(&window)
            .expect("Failed to create native widget");
        let surface_type = SurfaceType::Widget { native_widget };
        let webrender_surfman = WebrenderSurfman::create(&connection, &adapter, surface_type)
            .expect("Failed to create WR surfman");

        webrender_surfman
    }

    pub fn open_native_display(&mut self) -> &Option<Connection> {
        let window_builder = winit::window::WindowBuilder::new().with_visible(false);
        let window = window_builder.build(&self.el).unwrap();

        // Initialize surfman
        let connection =
            Connection::from_winit_window(&window).expect("Failed to create connection");

        self.connection = Some(connection);

        &self.connection
    }

    pub fn wait_for_window_resize(&mut self, target_window_id: WindowId) {
        let deadline = Instant::now() + Duration::from_millis(100);
        self.el.run_return(|e, _, control_flow| match e {
            Event::NewEvents(StartCause::Init) => {
                *control_flow = ControlFlow::WaitUntil(deadline);
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                window_id,
            } => {
                if target_window_id == window_id {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        });
    }

    pub fn get_available_monitors(&self) -> impl Iterator<Item = MonitorHandle> {
        self.el.available_monitors()
    }

    pub fn get_primary_monitor(&self) -> MonitorHandle {
        self.el
            .primary_monitor()
            .unwrap_or_else(|| -> MonitorHandle { self.get_available_monitors().next().unwrap() })
    }

    pub fn get_clipboard(&mut self) -> &mut Box<dyn ClipboardProvider> {
        &mut self.clipboard
    }
}

fn build_clipboard(_event_loop: &EventLoop<i32>) -> Box<dyn ClipboardProvider> {
    #[cfg(free_unix)]
    {
        if _event_loop.is_wayland() {
            let wayland_display = _event_loop
                .wayland_display()
                .expect("Fetch Wayland display failed");
            let (_, clipboard) = unsafe { create_clipboards_from_external(wayland_display) };
            Box::new(clipboard)
        } else {
            Box::new(X11ClipboardContext::<Clipboard>::new().unwrap())
        }
    }
    #[cfg(windows_platform)]
    {
        return Box::new(WindowsClipboardContext::new().unwrap());
    }
    #[cfg(macos_platform)]
    {
        return Box::new(OSXClipboardContext::new().unwrap());
    }
}

pub static EVENT_LOOP: Lazy<Mutex<WrEventLoop>> = Lazy::new(|| {
    let el = winit::event_loop::EventLoopBuilder::<i32>::with_user_event().build();
    let clipboard = build_clipboard(&el);
    let connection = None;

    Mutex::new(WrEventLoop {
        clipboard,
        el,
        connection,
    })
});

pub static EVENT_BUFFER: Lazy<Mutex<Vec<GUIEvent>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[no_mangle]
pub extern "C" fn wr_select(
    nfds: i32,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    _exceptfds: *mut fd_set,
    timeout: *mut timespec,
    _sigmask: *mut sigset_t,
) -> i32 {
    if unsafe { inhibit_window_system } {
        return unsafe {
            thread_select(
                Some(pselect),
                nfds,
                readfds,
                writefds,
                _exceptfds,
                timeout,
                _sigmask,
            )
        };
    }

    handle_select(nfds, readfds, writefds, _exceptfds, timeout, _sigmask)
}
