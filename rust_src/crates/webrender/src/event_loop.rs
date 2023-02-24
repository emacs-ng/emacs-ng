use crate::select::handle_select;
use raw_window_handle::{HasRawDisplayHandle, RawDisplayHandle};
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
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
#[cfg(free_unix)]
use winit::platform::wayland::EventLoopWindowTargetExtWayland;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    monitor::MonitorHandle,
    platform::run_return::EventLoopExtRunReturn,
    window::WindowId,
};

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
    el: EventLoop<i32>,
}

unsafe impl Send for WrEventLoop {}
unsafe impl Sync for WrEventLoop {}

impl WrEventLoop {
    pub fn el(&self) -> &EventLoop<i32> {
        &self.el
    }

    pub fn create_proxy(&self) -> EventLoopProxy<i32> {
        self.el.create_proxy()
    }

    pub fn open_native_display(&mut self) -> RawDisplayHandle {
        let window_builder = winit::window::WindowBuilder::new().with_visible(false);
        let window = window_builder.build(&self.el).unwrap();
        let rwh = window.raw_display_handle();

        rwh
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

pub static EVENT_LOOP: Lazy<Arc<Mutex<WrEventLoop>>> = Lazy::new(|| {
    let el = winit::event_loop::EventLoopBuilder::<i32>::with_user_event().build();
    let clipboard = build_clipboard(&el);

    Arc::new(Mutex::new(WrEventLoop { clipboard, el }))
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
    let lock_result = EVENT_LOOP.try_lock();

    if lock_result.is_err() || unsafe { inhibit_window_system } {
        if lock_result.is_err() {
            log::debug!("Failed to grab a lock {:?}", lock_result.err());
        }

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

    let mut event_loop = lock_result.unwrap();

    handle_select(
        &mut event_loop.el,
        nfds,
        readfds,
        writefds,
        _exceptfds,
        timeout,
        _sigmask,
    )
}

// Polling C-g when emacs is blocked
pub fn poll_a_event(timeout: Duration) -> Option<GUIEvent> {
    log::trace!("poll a event {:?}", timeout);
    let result = EVENT_LOOP.try_lock();
    if result.is_err() {
        log::trace!("failed to grab a EVENT_LOOP lock");
        return None;
    }
    let mut event_loop = result.unwrap();
    let deadline = Instant::now() + timeout;
    let result = RefCell::new(None);
    event_loop.el.run_return(|e, _target, control_flow| {
        control_flow.set_wait_until(deadline);

        if let Event::WindowEvent { event, .. } = &e {
            log::trace!("{:?}", event);
        }

        match e {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(_)
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::ReceivedCharacter(_)
                | WindowEvent::ModifiersChanged(_)
                | WindowEvent::MouseInput { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::Focused(_)
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::CloseRequested => {
                    result.replace(Some(e.to_static().unwrap()));
                    control_flow.set_exit();
                }
                _ => {}
            },
            Event::RedrawEventsCleared => {
                control_flow.set_exit();
            }
            _ => {}
        };
    });
    result.into_inner()
}
