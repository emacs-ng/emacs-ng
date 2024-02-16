use crate::event_loop::global_event_buffer;
use std::{cell::RefCell, ptr, sync::Mutex};

use crate::window_system::api::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};
use libc::{fd_set, sigset_t, timespec};
use once_cell::sync::Lazy;
use tokio::{runtime::Runtime, time::Duration};

use crate::future::tokio_select_fds;

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

pub static TOKIO_RUNTIME: Lazy<Mutex<Runtime>> = Lazy::new(|| {
    Mutex::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .enable_time()
            .worker_threads(2)
            .max_blocking_threads(32)
            .build()
            .unwrap(),
    )
});

pub fn handle_select(
    event_loop: &mut EventLoop<i32>,
    nfds: i32,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    _exceptfds: *mut fd_set,
    timeout: *mut timespec,
    _sigmask: *mut sigset_t,
) -> i32 {
    let event_loop_proxy = event_loop.create_proxy();

    let deadline = unsafe { Duration::new((*timeout).tv_sec as u64, (*timeout).tv_nsec as u32) };
    let read_fds = FdSet(readfds);
    let write_fds = FdSet(writefds);
    let timeout = Timespec(timeout);

    let (select_stop_sender, mut select_stop_receiver) = tokio::sync::mpsc::unbounded_channel();

    // use tokio to mimic the pselect because it has cross platform supporting.
    let tokio_runtime = TOKIO_RUNTIME.lock().unwrap();
    tokio_runtime.spawn(async move {
        tokio::select! {

            nfds = tokio_select_fds(nfds, &read_fds, &write_fds, &timeout) => {
                let _ = event_loop_proxy.send_event(nfds);
            }

            // time out
            _ = tokio::time::sleep(deadline) => {
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
    event_loop.run_return(|e, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let mut event_buffer = global_event_buffer().lock().expect("whops..");
        let mut keyboard_event = |e: Event<'_, i32>| {
            event_buffer.push(e.to_static().unwrap());
            // notify emacs's code that a keyboard event arrived.
            unsafe { libc::raise(libc::SIGIO) };

            // stop tokio select
            let _ = select_stop_sender.send(());
        };

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
                    keyboard_event(e);
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
