# Types

## LispObject and Lisp_Object

Lisp_Object (LispObject in Rust) is an integer aka EMACS_INT (EmacsInt
in Rust). It is also used as a pointer and bitmasking is used to add
extra data to the Lisp_Object.  Numbers

## EmacsInt and EMACS_INT

There are two types of Integers in Emacs Lisp -- Fixnums and
Natnums. A fixnum is equivalent to a i64/i32 in Rust and a natnum is
u64/u32. In general, use a fixnum unless there is a reason to use a
natnum. A fixnum is represented by a EmacsInt and natnum by a
EmacsUint.

## char and i8/u8

Emacs uses the C char type to represent single byte values. Since char
is unsigned by default on ARM platforms and signed on x86 and others,
it's important to use the libc::c_char when writing functions that
interoperate with the char type.

## ENUM_BF and bool_bf

Because part of emacs-ng is written in Rust, while the bulk of it is
still in C, both the Rust and the C code must be able to call
functions written in the other language. Rust makes this fairly
painless. It has excellent support for C FFI.

However, manually translating function and structure declarations into
Rust can be quite painful. Worse, any tiny mistake will come back to
haunt you later. Crashes and weird bugs that don't make sense are a
very real problem. We had several itermittent bugs that were
introduced when a complicated struct was incorrectly translated into
Rust, so that parts of the code were stepping on each other.

We've fixed this problem by using Bindgen to generate these bindings
for us.

Aside from saving us a lot of time, Bindgen also gives us relatively
nice ways to handle C enums, unions, bitfields, and variable-length
structures. Emacs frequently uses these, so this is a great help.

First allow me to show you a fairly important C structure called
Lisp_Symbol. This struct holds all of the information that Emacs knows
about a Lisp symbol. It's got a number of bit fields as well as an
internal union. Note that I've elided the comments from this
declaration:

```C
struct Lisp_Symbol
{
  bool_bf gcmarkbit : 1;
  ENUM_BF (symbol_redirect) redirect : 3;
  ENUM_BF (symbol_trapped_write) trapped_write : 2;
  unsigned interned : 2;
  bool_bf declared_special : 1;
  bool_bf pinned : 1;
  Lisp_Object name;
  union {
    Lisp_Object value;
    struct Lisp_Symbol *alias;
    struct Lisp_Buffer_Local_Value *blv;
    union Lisp_Fwd *fwd;
  } val;
  Lisp_Object function;
  Lisp_Object plist;
  struct Lisp_Symbol *next;
};
```

ENUM_BF and bool_bf are C preprocessor hacks that allow the code to be
compiled even when the compiler doesn't support enums or bools as
bitfield types. Bindgen generates the following Rust struct:

```rust
#[repr(C)]
pub struct Lisp_Symbol {
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 2usize], u8>,
    pub name: Lisp_Object,
    pub val: Lisp_Symbol__bindgen_ty_1,
    pub function: Lisp_Object,
    pub plist: Lisp_Object,
    pub next: *mut Lisp_Symbol,
}
#[repr(C)]
pub union Lisp_Symbol__bindgen_ty_1 {
    pub value: Lisp_Object,
    pub alias: *mut Lisp_Symbol,
    pub blv: *mut Lisp_Buffer_Local_Value,
    pub fwd: *mut Lisp_Fwd,
    _bindgen_union_align: u64,
}
```

As you can see, the bitfields become rather opaque, they're no longer
listed in the struct (you can, however, still see that they occupy two
bytes in the struct). Instead, Bindgen creates getter and setter
methods and adds them to the impl Lisp_Symbol. I'll just show an
excerpt here:

```rust
impl Lisp_Symbol {
    #[inline]
    pub fn gcmarkbit(&self) -> bool_bf {
        unsafe { ::std::mem::transmute(self._bitfield_1.get(0usize, 1u8) as u8) }
    }
    #[inline]
    pub fn set_gcmarkbit(&mut self, val: bool_bf) {
        unsafe {
            let val: u8 = ::std::mem::transmute(val);
            self._bitfield_1.set(0usize, 1u8, val as u64)
        }
    }
    // ---8<---
}
```

The union is also a little more verbose than before, as it cannot be
put anonymously into the rest of the struct. Rust requires that it
have a proper name, and so Bindgen has generated one. It's not great,
but it'll suffice.
