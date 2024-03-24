use super::input::InputEvent;
use emacs_sys::bindings::event_kind;
use emacs_sys::bindings::input_event;
use emacs_sys::bindings::scroll_bar_part;
use emacs_sys::globals::Qnil;
use emacs_sys::globals::Qt;
use emacs_sys::lisp::LispObject;

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
