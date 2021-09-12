use std::{cell::RefCell, os::unix::prelude::AsRawFd, ptr, sync::Mutex, time::Instant};

#[cfg(macos)]
use copypasta::osx_clipboard::OSXClipboardContext;
#[cfg(windows)]
use copypasta::windows_clipboard::WindowsClipboardContext;
use copypasta::ClipboardProvider;
#[cfg(unix)]
use copypasta::{
    wayland_clipboard::create_clipboards_from_external,
    x11_clipboard::{Clipboard, X11ClipboardContext},
};
use futures::future::FutureExt;
#[cfg(unix)]
use glutin::platform::unix::EventLoopWindowTargetExtUnix;
use glutin::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    monitor::MonitorHandle,
    platform::run_return::EventLoopExtRunReturn,
    window::{WindowBuilder, WindowId},
    ContextBuilder, ContextCurrentState, CreationError, NotCurrent, WindowedContext,
};
use libc::{c_void, fd_set, pselect, sigset_t, timespec};
use once_cell::sync::Lazy;
use tokio::{io::unix::AsyncFd, runtime::Runtime, time::Duration};

use crate::future::batch_select;

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
    pub fn build_window<'a, T: ContextCurrentState>(
        &mut self,
        window_builder: WindowBuilder,
        context_builder: ContextBuilder<'a, T>,
    ) -> Result<WindowedContext<NotCurrent>, CreationError> {
        context_builder.build_windowed(window_builder, &self.el)
    }

    pub fn create_proxy(&self) -> EventLoopProxy<i32> {
        self.el.create_proxy()
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

fn build_clipboard(event_loop: &EventLoop<i32>) -> Box<dyn ClipboardProvider> {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if event_loop.is_wayland() {
            let wayland_display = event_loop
                .wayland_display()
                .expect("Fetch Wayland display failed");
            let (_, clipboard) = unsafe { create_clipboards_from_external(wayland_display) };
            Box::new(clipboard)
        } else {
            Box::new(X11ClipboardContext::<Clipboard>::new().unwrap())
        }
    }
    #[cfg(target_os = "windows")]
    {
        return Box::new(WindowsClipboardContext::new().unwrap());
    }
    #[cfg(target_os = "macos")]
    {
        return Box::new(OSXClipboardContext::new().unwrap());
    }
}

pub static EVENT_LOOP: Lazy<Mutex<WrEventLoop>> = Lazy::new(|| {
    let el = glutin::event_loop::EventLoop::with_user_event();
    let clipboard = build_clipboard(&el);

    Mutex::new(WrEventLoop { clipboard, el })
});

pub static TOKIO_RUNTIME: Lazy<Mutex<Runtime>> =
    Lazy::new(|| Mutex::new(tokio::runtime::Runtime::new().unwrap()));

pub static EVENT_BUFFER: Lazy<Mutex<Vec<GUIEvent>>> = Lazy::new(|| Mutex::new(Vec::new()));

struct FdSet(*mut fd_set);

unsafe impl Send for FdSet {}
unsafe impl Sync for FdSet {}

impl FdSet {
    fn clear(&self) {
        if self.0 != ptr::null_mut() {
            unsafe { libc::FD_ZERO(self.0) };
        }
    }
}

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

    let mut event_loop = EVENT_LOOP.lock().unwrap();

    let event_loop_proxy = event_loop.create_proxy();

    let read_fds = FdSet(readfds);
    let write_fds = FdSet(writefds);

    let timeout = unsafe { Duration::new((*timeout).tv_sec as u64, (*timeout).tv_nsec as u32) };

    let (select_stop_sender, mut select_stop_receiver) = tokio::sync::mpsc::unbounded_channel();

    // use tokio to mimic the pselect because it has cross platform supporting.
    TOKIO_RUNTIME.lock().unwrap().spawn(async move {
        tokio::select! {
            (readables, writables) = tokio_select_fds(nfds, &read_fds , &write_fds) => {
                let nfds = readables.len() + writables.len();

                async_fds_to_fd_set(readables, &read_fds);
                async_fds_to_fd_set(writables, &write_fds);

                let _ = event_loop_proxy.send_event(nfds as i32);
            }

            // time out
            _ = tokio::time::sleep(timeout) => {
                read_fds.clear();
                write_fds.clear();

                let _ = event_loop_proxy.send_event(0);
            }

            // received stop command from winit event_loop
            _ = select_stop_receiver.recv() => {
                read_fds.clear();
                write_fds.clear();

                let _ = event_loop_proxy.send_event(1);
            }

        }
    });

    let nfds_result = RefCell::new(0);

    // We mush run winit in main thread, because the macOS platfrom limitation.
    event_loop.el.run_return(|e, _, control_flow| {
        *control_flow = ControlFlow::Wait;

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
                    EVENT_BUFFER.lock().unwrap().push(e.to_static().unwrap());

                    // notify emacs's code that a keyboard event arrived.
                    unsafe { libc::raise(libc::SIGIO) };

                    // stop tokio select
                    let _ = select_stop_sender.send(());
                }
                _ => {}
            },

            Event::UserEvent(nfds) => {
                nfds_result.replace(nfds);
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        };
    });

    return nfds_result.into_inner();
}

fn fd_set_to_async_fds(nfds: i32, fds: &FdSet) -> Vec<AsyncFd<i32>> {
    if fds.0 == ptr::null_mut() {
        return Vec::new();
    }

    let mut async_fds = Vec::new();

    for fd in 0..nfds {
        unsafe {
            if libc::FD_ISSET(fd, fds.0) {
                async_fds.push(AsyncFd::new(fd).unwrap())
            }
        }
    }

    async_fds
}

fn async_fds_to_fd_set(fds: Vec<i32>, fd_set: &FdSet) {
    if fd_set.0 == ptr::null_mut() {
        return;
    }

    unsafe { libc::FD_ZERO(fd_set.0) }

    for f in fds {
        unsafe { libc::FD_SET(f, fd_set.0) }
    }
}

async fn tokio_select_fds(nfds: i32, readfds: &FdSet, writefds: &FdSet) -> (Vec<i32>, Vec<i32>) {
    let read_fds = fd_set_to_async_fds(nfds, readfds);
    let write_fds = fd_set_to_async_fds(nfds, writefds);

    let mut fd_futures = Vec::new();

    for f in read_fds.iter() {
        fd_futures.push(f.readable().boxed())
    }

    for f in write_fds.iter() {
        fd_futures.push(f.writable().boxed())
    }

    let read_fds_count = read_fds.len();

    let readliness = batch_select(fd_futures).await;

    let mut readable_result = Vec::new();
    let mut writable_result = Vec::new();

    for (result, index) in readliness {
        if result.is_err() {
            continue;
        }

        if index < read_fds_count {
            readable_result.push(read_fds[index].as_raw_fd())
        } else {
            writable_result.push(write_fds[index - read_fds_count].as_raw_fd())
        }
    }

    (readable_result, writable_result)
}
