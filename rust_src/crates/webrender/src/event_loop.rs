use crate::select::handle_select;
#[cfg(use_winit)]
use crate::window_system::api::event_loop::EventLoopBuilder;
use crate::window_system::clipboard::Clipboard;
use crate::window_system::clipboard::ClipboardExt;
use std::sync::OnceLock;
use std::{
    cell::RefCell,
    ptr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::window_system::api::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    event_loop::EventLoop,
    monitor::MonitorHandle,
    platform::run_return::EventLoopExtRunReturn,
};
use emacs::bindings::{inhibit_window_system, thread_select};
use libc::{c_void, fd_set, pselect, sigset_t, timespec};

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
    clipboard: Clipboard,
    el: EventLoop<i32>,
}

unsafe impl Send for WrEventLoop {}
unsafe impl Sync for WrEventLoop {}

impl WrEventLoop {
    pub fn el(&self) -> &EventLoop<i32> {
        &self.el
    }

    pub fn get_available_monitors(&self) -> impl Iterator<Item = MonitorHandle> {
        self.el.available_monitors()
    }

    pub fn get_primary_monitor(&self) -> MonitorHandle {
        self.el
            .primary_monitor()
            .unwrap_or_else(|| -> MonitorHandle { self.get_available_monitors().next().unwrap() })
    }

    pub fn get_clipboard(&mut self) -> &mut Clipboard {
        &mut self.clipboard
    }
}

pub static EVENT_LOOP: OnceLock<Arc<Mutex<WrEventLoop>>> = OnceLock::new();
impl WrEventLoop {
    pub fn global() -> &'static Arc<Mutex<WrEventLoop>> {
        EVENT_LOOP.get_or_init(|| {
            log::trace!("wr event loop is being created...");
            let (el, clipboard) = {
                #[cfg(use_winit)]
                let el = EventLoopBuilder::<i32>::with_user_event().build();
                #[cfg(use_tao)]
                let el = EventLoop::<i32>::with_user_event();

                let clipboard = Clipboard::build(&el);
                (el, clipboard)
            };

            Arc::new(Mutex::new(Self { clipboard, el }))
        })
    }
}

pub fn global_event_buffer() -> &'static Mutex<Vec<GUIEvent>> {
    static EVENT_BUFFER: OnceLock<Mutex<Vec<GUIEvent>>> = OnceLock::new();
    EVENT_BUFFER.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn flush_events() -> Vec<GUIEvent> {
    let event_buffer = global_event_buffer().try_lock();

    if event_buffer.is_err() {
        return Vec::new();
    }

    let mut event_buffer = event_buffer.ok().unwrap();
    let events = event_buffer.clone();
    event_buffer.clear();
    events
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FdSet(pub *mut fd_set);

unsafe impl Send for FdSet {}
unsafe impl Sync for FdSet {}

impl FdSet {
    fn clear(&self) {
        if self.0 != ptr::null_mut() {
            unsafe { libc::FD_ZERO(self.0) };
        }
    }
}

impl Drop for FdSet {
    fn drop(&mut self) {
        self.clear()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Timespec(pub *mut timespec);

unsafe impl Send for Timespec {}
unsafe impl Sync for Timespec {}

#[no_mangle]
pub extern "C" fn winit_select(
    nfds: i32,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    _exceptfds: *mut fd_set,
    timeout: *mut timespec,
    _sigmask: *mut sigset_t,
) -> i32 {
    log::trace!("winit select");
    let lock_result = WrEventLoop::global().try_lock();

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
    let result = WrEventLoop::global().try_lock();
    if result.is_err() {
        log::trace!("failed to grab a EVENT_LOOP lock");
        return None;
    }
    let mut event_loop = result.unwrap();
    let deadline = Instant::now() + timeout;
    let result = RefCell::new(None);
    event_loop.el.run_return(|e, _target, control_flow| {
        *control_flow = ControlFlow::WaitUntil(deadline);

        if let Event::WindowEvent { event, .. } = &e {
            log::trace!("{:?}", event);
        }

        match e {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(_)
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::ModifiersChanged(_)
                | WindowEvent::MouseInput { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::Focused(_)
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::CloseRequested => {
                    result.replace(Some(e.to_static().unwrap()));
                    *control_flow = ControlFlow::Exit;
                }
                #[cfg(use_tao)]
                WindowEvent::ReceivedImeText(_) => {
                    result.replace(Some(e.to_static().unwrap()));
                    *control_flow = ControlFlow::Exit;
                }

                #[cfg(use_winit)]
                WindowEvent::ReceivedCharacter(_) => {
                    result.replace(Some(e.to_static().unwrap()));
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                result.replace(Some(e.to_static().unwrap()));
                log::debug!("WindowEvent:: RedrawRequested");
            }
            Event::RedrawEventsCleared => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        };
    });
    result.into_inner()
}

#[cfg(use_tao)]
pub fn ensure_window(id: crate::window_system::api::window::WindowId) {
    let now = std::time::Instant::now();
    log::trace!("ensure window is created {:?}", id);
    let result = WrEventLoop::global().try_lock();
    if result.is_err() {
        log::trace!("failed to grab a EVENT_LOOP lock");
        return;
    }
    let mut event_loop = result.unwrap();
    event_loop.el.run_return(|e, _target, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent { event, .. } = &e {
            log::trace!("{:?}", event);
        }

        match e {
            Event::WindowEvent {
                ref event,
                window_id,
                ..
            } => match event {
                WindowEvent::Focused(is_focused) => {
                    if id == window_id {
                        if *is_focused {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    });
    let elapsed = now.elapsed();
    log::trace!("window creation takes for {:?} in {:?}", id, elapsed);
}
