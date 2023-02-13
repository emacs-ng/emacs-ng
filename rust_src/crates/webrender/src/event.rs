use crate::input::InputEvent;
use emacs::{
    bindings::{event_kind, input_event, scroll_bar_part},
    globals::{Qnil, Qt},
    lisp::LispObject,
};

pub fn create_emacs_event(kind: event_kind::Type, top_frame: LispObject) -> input_event {
    InputEvent {
        kind,
        part: scroll_bar_part::scroll_bar_nowhere,
        code: 0,
        modifiers: 0,
        x: 0.into(),
        y: 0.into(),
        timestamp: 0,
        frame_or_window: top_frame,
        arg: Qnil,
        device: Qt,
    }
    .into()
}
