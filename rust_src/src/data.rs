/// Forwarding pointer to a LispObject variable.
/// This is allowed only in the value cell of a symbol,
/// and it means that the symbol's value really lives in the
/// specified variable.
pub type Lisp_Fwd_Type = u32;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Lisp_Objfwd {
    pub ty: Lisp_Fwd_Type, // = Lisp_Fwd_Obj
    pub objvar: *mut crate::lisp::LispObject,
}
