use crate::window_system::api::event::ModifiersState;
use crate::window_system::api::event::VirtualKeyCode;
use emacs::sys::EmacsModifiers::{ctrl_modifier, meta_modifier, shift_modifier, super_modifier};

pub fn virtual_keycode(code: VirtualKeyCode) -> u32 {
    code as u32
}

pub fn to_emacs_modifiers(modifiers: ModifiersState) -> u32 {
    let mut emacs_modifiers: u32 = 0;

    if modifiers.alt() {
        emacs_modifiers |= meta_modifier;
    }
    if modifiers.shift() {
        emacs_modifiers |= shift_modifier;
    }
    if modifiers.ctrl() {
        emacs_modifiers |= ctrl_modifier;
    }
    if modifiers.logo() {
        emacs_modifiers |= super_modifier;
    }

    emacs_modifiers
}

pub fn keysym_to_emacs_key_name(keysym: i32) -> *const libc::c_char {
    keycode_to_emacs_key_name(unsafe { std::mem::transmute::<i32, VirtualKeyCode>(keysym) })
}

pub fn keycode_to_emacs_key_name(keycode: VirtualKeyCode) -> *const libc::c_char {
    match keycode {
        VirtualKeyCode::Escape => kn!("escape"),
        VirtualKeyCode::Back => kn!("backspace"),
        VirtualKeyCode::Return => kn!("return"),
        VirtualKeyCode::Tab => kn!("tab"),

        VirtualKeyCode::Home => kn!("home"),
        VirtualKeyCode::End => kn!("end"),
        VirtualKeyCode::PageUp => kn!("prior"),
        VirtualKeyCode::PageDown => kn!("next"),

        VirtualKeyCode::Left => kn!("left"),
        VirtualKeyCode::Right => kn!("right"),
        VirtualKeyCode::Up => kn!("up"),
        VirtualKeyCode::Down => kn!("down"),

        VirtualKeyCode::Insert => kn!("insert"),

        VirtualKeyCode::F1 => kn!("f1"),
        VirtualKeyCode::F2 => kn!("f2"),
        VirtualKeyCode::F3 => kn!("f3"),
        VirtualKeyCode::F4 => kn!("f4"),
        VirtualKeyCode::F5 => kn!("f5"),
        VirtualKeyCode::F6 => kn!("f6"),
        VirtualKeyCode::F7 => kn!("f7"),
        VirtualKeyCode::F8 => kn!("f8"),
        VirtualKeyCode::F9 => kn!("f9"),
        VirtualKeyCode::F10 => kn!("f10"),
        VirtualKeyCode::F11 => kn!("f11"),
        VirtualKeyCode::F12 => kn!("f12"),
        VirtualKeyCode::F13 => kn!("f13"),
        VirtualKeyCode::F14 => kn!("f14"),
        VirtualKeyCode::F15 => kn!("f15"),
        VirtualKeyCode::F16 => kn!("f16"),
        VirtualKeyCode::F17 => kn!("f17"),
        VirtualKeyCode::F18 => kn!("f18"),
        VirtualKeyCode::F19 => kn!("f19"),
        VirtualKeyCode::F20 => kn!("f20"),
        VirtualKeyCode::F21 => kn!("f21"),
        VirtualKeyCode::F22 => kn!("f22"),
        VirtualKeyCode::F23 => kn!("f23"),
        VirtualKeyCode::F24 => kn!("f24"),

        _ => std::ptr::null(), // null pointer
    }
}
