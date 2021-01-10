use crate::{
    lisp::{ExternalPtr, LispObject},
    remacs_sys::{allocate_kboard, KBOARD},
};

pub type Keyboard = KBOARD;
pub type KeyboardRef = ExternalPtr<KBOARD>;

impl KeyboardRef {
    pub fn add_ref(&mut self) {
        (*self).reference_count = (*self).reference_count + 1;
    }
}

pub fn allocate_keyboard(keyboard_type: LispObject) -> KeyboardRef {
    KeyboardRef::new(unsafe { allocate_kboard(keyboard_type) })
}
