use emacs::{
    bindings::{
        list4i, make_frame, make_frame_without_minibuffer, make_minibuffer_frame, output_method,
        wr_output,
    },
    frame::{window_frame_live_or_selected, LispFrameRef},
    globals::{Qinner_edges, Qnil, Qnone, Qonly, Qouter_edges},
    keyboard::KeyboardRef,
    lisp::LispObject,
};
use winit::dpi::PhysicalPosition;

use crate::{event_loop::EVENT_LOOP, output::OutputRef};

use super::{display_info::DisplayInfoRef, output::Output};

pub fn create_frame(
    display: LispObject,
    dpyinfo: DisplayInfoRef,
    tem: LispObject,
    mut kb: KeyboardRef,
) -> LispFrameRef {
    let frame = if tem.eq(Qnone) || tem.is_nil() {
        unsafe { make_frame_without_minibuffer(Qnil, kb.as_mut(), display) }
    } else if tem.eq(Qonly) {
        unsafe { make_minibuffer_frame() }
    } else if tem.is_window() {
        unsafe { make_frame_without_minibuffer(tem, kb.as_mut(), display) }
    } else {
        unsafe { make_frame(true) }
    };

    let mut frame = LispFrameRef::new(frame);

    frame.terminal = dpyinfo.get_inner().terminal.as_mut();
    frame.set_output_method(output_method::output_wr);

    let mut event_loop = EVENT_LOOP.lock().unwrap();
    let mut output = Box::new(Output::build(&mut event_loop, frame));

    let window_id = output.get_window().id();

    output.set_display_info(dpyinfo);

    // Remeber to destory the Output object when frame destoried.
    let output = Box::into_raw(output);
    frame.output_data.wr = output as *mut wr_output;

    dpyinfo
        .get_inner()
        .outputs
        .insert(window_id, frame.wr_output());

    frame
}

pub fn frame_edges(frame: LispObject, type_: LispObject) -> LispObject {
    let frame = window_frame_live_or_selected(frame);

    let output = frame.wr_output();

    let window = output.get_window();

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
            let internal_border_width = frame.internal_border_width();

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

pub trait LispFrameExt {
    fn wr_output(&self) -> OutputRef;
    fn wr_display_info(&self) -> DisplayInfoRef;
}

impl LispFrameExt for LispFrameRef {
    fn wr_output(&self) -> OutputRef {
        let output: OutputRef = unsafe { self.output_data.wr.into() };
        output
    }

    fn wr_display_info(&self) -> DisplayInfoRef {
        self.wr_output().display_info()
    }
}
