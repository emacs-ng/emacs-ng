use crate::event_loop::global_event_buffer;
use crate::window_system::api::event_loop::ControlFlow;
use crate::window_system::api::event_loop::EventLoop;
use errno::{set_errno, Errno};
use nix::sys::signal::{self, Signal};
use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

#[cfg(all(x11_platform, use_winit))]
use crate::window_system::api::platform::x11::EventLoopWindowTargetExtX11;
use crate::window_system::api::{
    event::{Event, WindowEvent},
    platform::run_return::EventLoopExtRunReturn,
};
use libc::{fd_set, sigset_t, timespec};
#[cfg(use_tao)]
use tao::platform::unix::EventLoopWindowTargetExtUnix;

use emacs::bindings::make_timespec;

pub fn handle_select(
    event_loop: &mut EventLoop<i32>,
    nfds: i32,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    _exceptfds: *mut fd_set,
    timeout: *mut timespec,
    _sigmask: *mut sigset_t,
) -> i32 {
    let deadline = Instant::now()
        + unsafe { Duration::new((*timeout).tv_sec as u64, (*timeout).tv_nsec as u32) };

    let nfds_result = RefCell::new(0);

    if let Ok(mut event_buffer) = global_event_buffer().try_lock() {
        // We mush run winit in main thread, because the macOS platfrom limitation.
        event_loop.run_return(|e, _target, control_flow| {
            *control_flow = ControlFlow::WaitUntil(deadline);

            if let Event::WindowEvent { event, .. } = &e {
                // Print only Window events to reduce noise
                log::trace!("{:?}", event);
            }

            let mut keyboard_event = |e: Event<'_, i32>| {
                event_buffer.push(e.to_static().unwrap());
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

                *control_flow = ControlFlow::Exit;
            };

            match e {
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::Resized(_)
                    | WindowEvent::KeyboardInput { .. }
                    | WindowEvent::ModifiersChanged(_)
                    | WindowEvent::MouseInput { .. }
                    | WindowEvent::CursorMoved { .. }
                    | WindowEvent::ThemeChanged(_)
                    | WindowEvent::Focused(_)
                    | WindowEvent::MouseWheel { .. }
                    | WindowEvent::CloseRequested => {
                        keyboard_event(e);
                    }
                    #[cfg(use_winit)]
                    WindowEvent::ReceivedCharacter(_) => {
                        keyboard_event(e);
                    }

                    #[cfg(use_tao)]
                    WindowEvent::ReceivedImeText(_) => {
                        keyboard_event(e);
                    }
                    _ => {}
                },
                Event::UserEvent(nfds) => {
                    nfds_result.replace(nfds);
                    *control_flow = ControlFlow::Exit;
                }
                Event::RedrawRequested(_) => {
                    event_buffer.push(e.to_static().unwrap());
                    log::debug!("WindowEvent:: RedrawRequested");
                }
                Event::RedrawEventsCleared => {
                    event_buffer.push(e.to_static().unwrap());
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        });
    }

    let ret = nfds_result.into_inner();
    if ret == 0 {
        let timespec = unsafe { make_timespec(0, 0) };
        // Add some delay here avoding high cpu usage on macOS
        #[cfg(any(macos_platform, use_tao))]
        spin_sleep::sleep(Duration::from_millis(16));
        let nfds =
            unsafe { libc::pselect(nfds, readfds, writefds, _exceptfds, &timespec, _sigmask) };
        return nfds;
    }

    log::trace!("winit event run_return: {ret:?}");

    ret
}
