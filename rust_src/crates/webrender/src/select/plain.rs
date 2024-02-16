use crate::event_loop::global_event_buffer;
use crate::window_system::api::event_loop::ControlFlow;
use crate::window_system::api::event_loop::EventLoop;
use crate::window_system::api::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use errno::{set_errno, Errno};
use nix::sys::signal::{self, Signal};
use nix::sys::time::TimeSpec;
use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use crate::window_system::api::event::{Event, WindowEvent};
#[cfg(all(x11_platform, use_winit))]
use crate::window_system::api::platform::x11::EventLoopWindowTargetExtX11;
use libc::{fd_set, sigset_t, timespec};

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

    let timeout = Some(Duration::ZERO);
    if let Ok(mut event_buffer) = global_event_buffer().try_lock() {
        let status = event_loop.pump_events(timeout, |e, elwt| {
            if let Event::WindowEvent { event, .. } = &e {
                // Print only Window events to reduce noise
                log::trace!("{e:?}");
            }

            let mut keyboard_event = |e: Event<i32>| {
                event_buffer.push(e);
                // notify emacs's code that a keyboard event arrived.
                match signal::raise(Signal::SIGIO) {
                    Ok(_) => {}
                    Err(err) => log::error!("sigio err: {err:?}"),
                };
                // let _is_x11 = false;

                // #[cfg(x11_platform)]
                // let _is_x11 = elwt.is_x11();

                // if _is_x11 {
                //     nfds_result.replace(1);
                // } else {
                //     /* Pretend that `select' is interrupted by a signal.  */
                //     set_errno(Errno(libc::EINTR));
                //     debug_assert_eq!(nix::errno::errno(), libc::EINTR);
                //     nfds_result.replace(-1);
                // }
            };

            match e {
                Event::AboutToWait => {
                    unsafe { emacs::bindings::redisplay() };
                }
                Event::WindowEvent {
                    event, window_id, ..
                } => match event {
                    WindowEvent::Resized(_)
                    | WindowEvent::KeyboardInput { .. }
                    | WindowEvent::ModifiersChanged(_)
                    | WindowEvent::MouseInput { .. }
                    | WindowEvent::CursorMoved { .. }
                    | WindowEvent::ThemeChanged(_)
                    | WindowEvent::Focused(_)
                    | WindowEvent::MouseWheel { .. }
                    | WindowEvent::RedrawRequested
                    | WindowEvent::CloseRequested => {
                        keyboard_event(Event::WindowEvent { window_id, event });
                    }
                    #[cfg(use_tao)]
                    WindowEvent::ReceivedImeText(_) => {
                        keyboard_event(event);
                    }
                    _ => {}
                },
                Event::UserEvent(nfds) => {
                    nfds_result.replace(nfds);
                }
                _ => {}
            }
        });
        if let PumpStatus::Exit(exit_code) = status {
            // break 'main ExitCode::from(exit_code as u8);
        }
    }

    let ret = nfds_result.into_inner();
    if ret == 0 {
        let mut timespec = TimeSpec::from_duration(deadline - Instant::now());
        let nfds = unsafe {
            libc::pselect(
                nfds,
                readfds,
                writefds,
                _exceptfds,
                timespec.as_mut(),
                _sigmask,
            )
        };
        return nfds;
    }

    log::trace!("winit event run_return: {ret:?}");

    ret
}
