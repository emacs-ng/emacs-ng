use errno::{set_errno, Errno};
use nix::sys::signal::{self, Signal};
use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use libc::{fd_set, sigset_t, timespec};
#[cfg(x11_platform)]
use winit::platform::x11::EventLoopWindowTargetExtX11;
use winit::{
    event::{Event, WindowEvent},
    platform::run_return::EventLoopExtRunReturn,
};

use emacs::bindings::make_timespec;

use crate::event_loop::EVENT_BUFFER;
use crate::event_loop::EVENT_LOOP;

pub fn handle_select(
    nfds: i32,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    _exceptfds: *mut fd_set,
    timeout: *mut timespec,
    _sigmask: *mut sigset_t,
) -> i32 {
    let mut event_loop = EVENT_LOOP.lock().unwrap();

    let deadline = Instant::now()
        + unsafe { Duration::new((*timeout).tv_sec as u64, (*timeout).tv_nsec as u32) };

    let nfds_result = RefCell::new(0);

    // We mush run winit in main thread, because the macOS platfrom limitation.
    event_loop.el.run_return(|e, _target, control_flow| {
        control_flow.set_wait_until(deadline);

        if let Event::WindowEvent { event, .. } = &e {
            // Print only Window events to reduce noise
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
                    EVENT_BUFFER.lock().unwrap().push(e.to_static().unwrap());
                    // notify emacs's code that a keyboard event arrived.
                    match signal::raise(Signal::SIGIO) {
                        Ok(_) => {}
                        Err(err) => log::error!("sigio err: {err:?}"),
                    };

                    let _is_x11 = false;

                    #[cfg(x11_platform)]
                    let _is_x11 = _target.is_x11();

                    if _is_x11 {
                        nfds_result.replace(1);
                    } else {
                        /* Pretend that `select' is interrupted by a signal.  */
                        set_errno(Errno(libc::EINTR));
                        debug_assert_eq!(nix::errno::errno(), libc::EINTR);
                        nfds_result.replace(-1);
                    }

                    control_flow.set_exit();
                }
                _ => {}
            },
            Event::UserEvent(nfds) => {
                nfds_result.replace(nfds);
                control_flow.set_exit();
            }
            Event::RedrawEventsCleared => {
                control_flow.set_exit();
            }
            _ => {}
        };
    });
    let ret = nfds_result.into_inner();
    if ret == 0 {
        let timespec = unsafe { make_timespec(0, 0) };
        // Add some delay here avoding high cpu usage on macOS
        #[cfg(macos_platform)]
        spin_sleep::sleep(Duration::from_millis(16));
        let nfds =
            unsafe { libc::pselect(nfds, readfds, writefds, _exceptfds, &timespec, _sigmask) };
        log::trace!("pselect: {nfds:?}");
        return nfds;
    }

    log::trace!("winit event run_return: {ret:?}");

    ret
}
