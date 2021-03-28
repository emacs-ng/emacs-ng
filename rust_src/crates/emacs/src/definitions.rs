pub type EmacsInt = libc::c_long;
pub type EmacsUint = libc::c_ulong;
pub const EMACS_INT_MAX: EmacsInt = 0x7FFFFFFFFFFFFFFF_i64;
pub const EMACS_INT_SIZE: EmacsInt = 8;
pub type EmacsDouble = f64;
pub const EMACS_FLOAT_SIZE: EmacsInt = 8;
pub type BoolBF = bool;
pub const USE_LSB_TAG: bool = true;
