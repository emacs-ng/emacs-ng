use crate::frame::FrameExtWinit;
use emacs_sys::sys::EmacsModifiers::ctrl_modifier;
use emacs_sys::sys::EmacsModifiers::meta_modifier;
use emacs_sys::sys::EmacsModifiers::shift_modifier;
use emacs_sys::sys::EmacsModifiers::super_modifier;
use winit::dpi::LogicalPosition;

use emacs_sys::bindings::event_kind;
use emacs_sys::bindings::input_event;
use emacs_sys::bindings::scroll_bar_part;
use emacs_sys::globals::Qnil;
use emacs_sys::globals::Qt;
use emacs_sys::lisp::LispObject;
use emacs_sys::sys::EmacsModifiers::down_modifier;
use emacs_sys::sys::EmacsModifiers::up_modifier;
use std::sync::LazyLock;
use std::sync::Mutex;
use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::MouseButton;
use winit::event::MouseScrollDelta;
use winit::event::TouchPhase;
use winit::keyboard::KeyCode;
use winit::keyboard::ModifiersState;
use winit::keyboard::PhysicalKey;

static INPUT_STATE: LazyLock<Mutex<InputProcessor>> =
    LazyLock::new(|| Mutex::new(InputProcessor::default()));
impl InputProcessor {
    pub fn snapshot() -> InputProcessor {
        INPUT_STATE.lock().map_or_else(
            |e| {
                log::error!("Failed to snapshot INPUT_STATE {e:?}");
                InputProcessor::default()
            },
            |s| s.clone(),
        )
    }

    fn update(new_state: InputProcessor) {
        log::trace!(
            "Input state changed:  {:?} {:?}",
            new_state.modifiers,
            new_state.total_delta
        );
        match INPUT_STATE.lock().and_then(|mut s| Ok(*s = new_state)) {
            Err(e) => log::error!("Failed to update INPUT_STATE: {e:?}"),
            _ => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct InputProcessor {
    modifiers: ModifiersState,
    total_delta: PhysicalPosition<f64>,
}

impl Default for InputProcessor {
    #[inline]
    fn default() -> Self {
        InputProcessor {
            modifiers: ModifiersState::default(),
            total_delta: PhysicalPosition::new(0.0, 0.9),
        }
    }
}

impl InputProcessor {
    pub fn handle_modifiers_changed(new_state: ModifiersState) {
        let snapshot = Self::snapshot();
        let mut modifiers = snapshot.modifiers.clone();

        if new_state.is_empty() {
            modifiers = new_state;
        } else if new_state.shift_key() {
            modifiers.set(ModifiersState::SHIFT, new_state.shift_key());
        } else if new_state.control_key() {
            modifiers.set(ModifiersState::CONTROL, new_state.control_key());
        } else if new_state.alt_key() {
            modifiers.set(ModifiersState::ALT, new_state.alt_key());
        } else if new_state.super_key() {
            modifiers.set(ModifiersState::SUPER, new_state.super_key());
        }

        Self::update(InputProcessor {
            modifiers,
            ..snapshot
        });
    }

    fn set_total_delta(total_delta: PhysicalPosition<f64>) {
        let snapshot = Self::snapshot();
        Self::update(InputProcessor {
            total_delta,
            ..snapshot
        });
    }

    fn get_modifiers() -> ModifiersState {
        let InputProcessor { modifiers, .. } = Self::snapshot();
        modifiers.clone()
    }
}

impl InputProcessor {
    pub fn handle_receive_char(c: char, top_frame: LispObject) -> Option<input_event> {
        let state = Self::snapshot();

        let iev: input_event = InputEvent {
            kind: event_kind::ASCII_KEYSTROKE_EVENT,
            part: scroll_bar_part::scroll_bar_nowhere,
            code: Self::remove_control(c) as u32,
            modifiers: to_emacs_modifiers(state.modifiers),
            x: 0.into(),
            y: 0.into(),
            timestamp: 0,
            frame_or_window: top_frame,
            arg: Qnil,
            device: Qt,
        }
        .into();

        Some(iev)
    }

    pub fn handle_key_pressed(
        physical_key: PhysicalKey,
        top_frame: LispObject,
    ) -> Option<input_event> {
        match physical_key {
            PhysicalKey::Unidentified(_native_key_code) => {
                //todo
                None
            }
            PhysicalKey::Code(key_code) => {
                let InputProcessor { modifiers, .. } = Self::snapshot();
                if keycode_to_emacs_key_name(key_code).is_null() {
                    return None;
                }

                let code = virtual_keycode(key_code);

                let iev: input_event = InputEvent {
                    kind: event_kind::NON_ASCII_KEYSTROKE_EVENT,
                    part: scroll_bar_part::scroll_bar_nowhere,
                    code,
                    modifiers: to_emacs_modifiers(modifiers.to_owned()),
                    x: 0.into(),
                    y: 0.into(),
                    timestamp: 0,
                    frame_or_window: top_frame,
                    arg: Qnil,
                    device: Qt,
                }
                .into();

                Some(iev)
            }
        }
    }

    pub fn handle_key_released() {}

    pub fn handle_mouse_pressed(
        button: MouseButton,
        state: ElementState,
        top_frame: LispObject,
    ) -> Option<input_event> {
        let c = match button {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
            MouseButton::Other(_) => 0,
            _ => todo!(),
        };

        let s = match state {
            ElementState::Pressed => down_modifier,
            ElementState::Released => up_modifier,
        };

        let mut pos = LogicalPosition::new(0, 0);

        if let Some(frame) = top_frame.as_frame() {
            pos = frame.cursor_position();
        }

        let InputProcessor { modifiers, .. } = Self::snapshot();
        let iev: input_event = InputEvent {
            kind: event_kind::MOUSE_CLICK_EVENT,
            part: scroll_bar_part::scroll_bar_nowhere,
            code: c as u32,
            modifiers: to_emacs_modifiers(modifiers.clone()) | s,
            x: pos.x.into(),
            y: pos.y.into(),
            timestamp: 0,
            frame_or_window: top_frame,
            arg: Qnil,
            device: Qt,
        }
        .into();

        Some(iev)
    }

    pub fn handle_mouse_wheel_scrolled(
        delta: MouseScrollDelta,
        phase: TouchPhase,
        top_frame: LispObject,
    ) -> Option<input_event> {
        if phase != TouchPhase::Moved {
            let _ = Self::set_total_delta(PhysicalPosition::new(0.0, 0.0));
        }

        let line_height = top_frame.as_frame().unwrap().line_height as f64;

        let event_meta = match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                if y == 0.0 && x == 0.0 {
                    None
                } else if y != 0.0 {
                    let lines = y.abs() as i32;
                    Some((event_kind::WHEEL_EVENT, y > 0.0, lines))
                } else {
                    let lines = x.abs() as i32;
                    Some((event_kind::HORIZ_WHEEL_EVENT, x > 0.0, lines))
                }
            }
            MouseScrollDelta::PixelDelta(pos) => {
                let mut total_delta = Self::snapshot().total_delta.clone();
                if phase != TouchPhase::Moved {
                    total_delta = PhysicalPosition::new(0.0, 0.0);
                }
                total_delta.y = total_delta.y + pos.y;
                total_delta.x = total_delta.x + pos.x;

                if total_delta.y.abs() >= total_delta.x.abs() && total_delta.y.abs() > line_height {
                    let lines = (total_delta.y / line_height).abs() as i32;

                    total_delta.y = total_delta.y % line_height;
                    total_delta.x = 0.0;

                    let _ = Self::set_total_delta(total_delta.clone());

                    Some((event_kind::WHEEL_EVENT, total_delta.y > 0.0, lines))
                } else if total_delta.x.abs() > total_delta.y.abs()
                    && total_delta.x.abs() > line_height
                {
                    let lines = (total_delta.x / line_height).abs() as i32;

                    total_delta.x = total_delta.x % line_height;
                    total_delta.y = 0.0;

                    let _ = Self::set_total_delta(total_delta.clone());

                    Some((event_kind::HORIZ_WHEEL_EVENT, total_delta.x > 0.0, lines))
                } else {
                    None
                }
            }
        };

        if event_meta.is_none() {
            return None;
        }

        let (kind, is_upper, lines) = event_meta.unwrap();

        let mut pos = LogicalPosition::new(0, 0);

        if let Some(frame) = top_frame.as_frame() {
            pos = frame.cursor_position();
        }

        let s = if is_upper { up_modifier } else { down_modifier };
        let iev: input_event = InputEvent {
            kind,
            part: scroll_bar_part::scroll_bar_nowhere,
            code: 0,
            modifiers: to_emacs_modifiers(Self::get_modifiers()) | s,
            x: pos.x.into(),
            y: pos.y.into(),
            timestamp: 0,
            frame_or_window: top_frame,
            arg: lines.into(),
            device: Qt,
        }
        .into();

        Some(iev)
    }

    fn remove_control(c: char) -> char {
        let mut c = c as u8;

        if c < 32 {
            let mut new_c = c + 64;

            if new_c >= 65 && new_c <= 90 {
                new_c += 32;
            }

            c = new_c;
        }

        c as char
    }
}

pub struct InputEvent {
    pub kind: event_kind::Type,
    pub part: scroll_bar_part::Type,
    pub code: ::libc::c_uint,
    pub modifiers: ::libc::c_uint,
    pub x: LispObject,
    pub y: LispObject,
    pub timestamp: emacs_sys::bindings::Time,
    pub frame_or_window: LispObject,
    pub arg: LispObject,
    pub device: LispObject,
}

impl From<InputEvent> for input_event {
    fn from(val: InputEvent) -> Self {
        let mut iev = input_event::default();
        iev.set_kind(val.kind);
        iev.set_part(val.part);
        iev.code = val.code;
        iev.modifiers = val.modifiers;
        iev.x = val.x;
        iev.y = val.y;
        iev.timestamp = val.timestamp;
        iev.frame_or_window = val.frame_or_window;
        iev.arg = val.arg;
        iev.device = val.device;
        iev
    }
}

pub fn virtual_keycode(code: KeyCode) -> u32 {
    code as u32
}

pub fn to_emacs_modifiers(modifiers: ModifiersState) -> u32 {
    let mut emacs_modifiers: u32 = 0;

    if modifiers.alt_key() {
        emacs_modifiers |= meta_modifier;
    }
    if modifiers.shift_key() {
        emacs_modifiers |= shift_modifier;
    }
    if modifiers.control_key() {
        emacs_modifiers |= ctrl_modifier;
    }
    if modifiers.super_key() {
        emacs_modifiers |= super_modifier;
    }

    emacs_modifiers
}

pub fn keysym_to_emacs_key_name(keysym: i32) -> *const libc::c_char {
    keycode_to_emacs_key_name(unsafe {
        std::mem::transmute::<i8, KeyCode>(keysym.try_into().unwrap())
    })
}

pub fn keycode_to_emacs_key_name(keycode: KeyCode) -> *const libc::c_char {
    match keycode {
        KeyCode::Escape => kn!("escape"),
        KeyCode::Backspace => kn!("backspace"),
        KeyCode::Delete => kn!("deletechar"),
        KeyCode::Enter | KeyCode::NumpadEnter => kn!("return"),
        KeyCode::Tab => kn!("tab"),

        KeyCode::Home => kn!("home"),
        KeyCode::End => kn!("end"),
        KeyCode::PageUp => kn!("prior"),
        KeyCode::PageDown => kn!("next"),

        KeyCode::ArrowLeft => kn!("left"),
        KeyCode::ArrowRight => kn!("right"),
        KeyCode::ArrowUp => kn!("up"),
        KeyCode::ArrowDown => kn!("down"),

        KeyCode::Insert => kn!("insert"),

        KeyCode::F1 => kn!("f1"),
        KeyCode::F2 => kn!("f2"),
        KeyCode::F3 => kn!("f3"),
        KeyCode::F4 => kn!("f4"),
        KeyCode::F5 => kn!("f5"),
        KeyCode::F6 => kn!("f6"),
        KeyCode::F7 => kn!("f7"),
        KeyCode::F8 => kn!("f8"),
        KeyCode::F9 => kn!("f9"),
        KeyCode::F10 => kn!("f10"),
        KeyCode::F11 => kn!("f11"),
        KeyCode::F12 => kn!("f12"),
        KeyCode::F13 => kn!("f13"),
        KeyCode::F14 => kn!("f14"),
        KeyCode::F15 => kn!("f15"),
        KeyCode::F16 => kn!("f16"),
        KeyCode::F17 => kn!("f17"),
        KeyCode::F18 => kn!("f18"),
        KeyCode::F19 => kn!("f19"),
        KeyCode::F20 => kn!("f20"),
        KeyCode::F21 => kn!("f21"),
        KeyCode::F22 => kn!("f22"),
        KeyCode::F23 => kn!("f23"),
        KeyCode::F24 => kn!("f24"),

        _ => std::ptr::null(), // null pointer
    }
}
