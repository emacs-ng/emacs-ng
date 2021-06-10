# Dynamic Modules

Emacs-ng is always built with dynamic modules support enabled, and is
*fully compatible* with dynamic modules written for "vanilla" Emacs.

On top of the existing `emacs-module.h` interface, Emacs-ng provides
additional extensions that allow dynamic modules to access *more of
Emacs's internals*. Dynamic modules can be written to take advantage
of these extra functionalities when they are available, while at the
same time being *fully compatible* with vanilla Emacs.

The additional extensions are exposed as a registry of named native
functions that can be looked up at run time. These native functions
are called *ng-module functions*:

```emacs-lisp
ELISP> (ng-module-function-address "ng_module_access_current_buffer_contents")
#<user-ptr ptr=0x10e31120d finalizer=0x0>
ELISP> (ng-module-function-address "non_existing_or_removed_function")
nil
```

Unlike normal module functions from `emacs_env`, these ng-module
functions have *globally stable addresses*. Therefore, the lookup can
(and should) be done once, at module load time, inside
`emacs_module_init`. Also note that, even though the lookup function
`ng-module-function-address` is available to Lisp code, it is intended
to be used by dynamic modules' native code. (Lisp code cannot
meaningfully use the returned address, anyway.)

Once an ng-module function is added, its signature will not change. If
a similar ng-module function with improved functionalities is added,
it will be given a different name. However, a ng-module function **can
be removed**.

## Direct access to buffer text

To access a buffer's text, a "vanilla" dynamic module has to call a
buffer-to-string function, like `buffer-substring`, then call
`emacs_env->copy_string_contents` (resulting in a `memcpy`). The
temporary Lisp string is typically discarded right away. This is a
potential performance bottleneck in hot code paths, like
[emacs-tree-sitter](https://github.com/ubolonton/emacs-tree-sitter)'s
parsing/querying.

A dynamic module can instead use the ng-module function
`ng_module_access_current_buffer_contents` to directly read a buffer's
text, without copying, or creating a Lisp string. It returns the
pointers to (and the sizes of) the 2 contiguous byte segments before
and after the buffer's gap.

The caller **must not write** through the returned pointers, and must
ensure that the data is **read before it is invalidated**. Some
operations that may invalidate the data are: buffer modifications,
garbage collection (which can be triggered by uses of `emacs_env`),
arena compaction (which can be triggered by `malloc` when Emacs is
built with `REL_ALLOC`).

Below is an example of how to use this function in a dynamic module
written in Rust:

```rust
use std::mem::{self, MaybeUninit};
use once_cell::sync::OnceCell;
use emacs::Env;

type AccessBufferContents = unsafe fn(*mut *const u8, *mut isize, *mut *const u8, *mut isize);

#[allow(non_upper_case_globals)]
pub static ng_module_access_current_buffer_contents: OnceCell<AccessBufferContents> = OnceCell::new();

#[emacs::module]
fn init(env: &Env) -> Result<()> {
    let get_addr = env.call("symbol-function", [env.intern("ng-module-function-address")?])?;
    // Got the registry.
    if get_addr.is_not_nil() {
        // Look up the ng-module function.
        match get_addr.call(("ng_module_access_current_buffer_contents",))?.into_rust::<Option<Value>>()? {
            Some(addr) => {
                // Got the pointer, "cast" it to the signature promised by ng-module.
                buffer::ng_module_access_current_buffer_contents.set(
                    unsafe { mem::transmute(addr.get_user_ptr()?) }
                ).unwrap();
            }
            None => (),
        }
    }
    Ok(())
}

pub unsafe fn current_buffer_contents(_: &Env) -> (&[u8], &[u8]) {
    let mut before_gap = MaybeUninit::uninit();
    let mut after_gap = MaybeUninit::uninit();
    let mut before_gap_size: isize = 0;
    let mut after_gap_size: isize = 0;
    let get_slices = ng_module_access_current_buffer_contents.get().unwrap();
    get_slices(
        before_gap.as_mut_ptr(),
        &mut before_gap_size,
        after_gap.as_mut_ptr(),
        &mut after_gap_size,
    );
    let before_gap_size = before_gap_size;
    let after_gap_size = after_gap_size;
    (
        if before_gap_size > 0 {
            std::slice::from_raw_parts(
                before_gap.assume_init(),
                before_gap_size as usize,
            )
        } else {
            &[]
        },
        if after_gap_size > 0 {
            std::slice::from_raw_parts(
                after_gap.assume_init(),
                after_gap_size as usize,
            )
        } else {
            &[]
        },
    )
}
```

A future version of
[emacs-module-rs](https://github.com/ubolonton/emacs-module-rs/) may
provide a more convenient wrapper for this function.
