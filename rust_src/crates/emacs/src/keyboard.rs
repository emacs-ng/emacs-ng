use crate::bindings::allocate_kboard;
use crate::bindings::KBOARD;
use crate::lisp::ExternalPtr;
use crate::lisp::LispObject;

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
