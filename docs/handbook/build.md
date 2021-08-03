# emacs-ng build details

## Overview

Since emacs-ng is only an additive layer, we have to extend the emacs
build system by including the rust code base as a library.

The most important emacs files are:

- `configure.ac`
- `Makefile.in`
- `src/Makefile.in`

These files contain additions which allow us to dynamically
enable the different emacs-ng features.

In order to compile the lisp functionality defined in rust, we have to
iterate the relevant rust code to find definitions of lisp globals.

## Cargo build script (build.rs)

This file is the build script of the main crate. In this script we
check which features are enabled and create include files that hold
bindings for functions and lisp globals.

There is a main `c_exports.rs` file for each crate. This file is
included in a crate's `lib.rs` file. It contains declarations for
public rust functions so they can be used from C.

Additionally it defines a *_init_syms function that is called by the
main init_syms function from the main crate. Unlike the other crates
bindings, the related file is under `OUT_DIR`.

Example for main init_syms function with webrender and javascript
enabled:

```rust
#[no_mangle]
pub extern "C" fn rust_init_syms() {
    webrender::webrender_init_syms();
    js::js_init_syms();
    ng_module::ng_module_init_syms();
    ng_async::ng_async_init_syms();
}
```

## Generate include files

In the init_syms of a crate, we can find the lisp globals.

Files that have `include!` macro calls at the end have an exports file
that is located in the `out` directory of a crate.

Example:

```rust
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/out/javascript_exports.rs"
));
```

The corresponding include file then contains:

```
export_lisp_fns! {
    async_handler,
    call_async_echo,
    call_async_data_echo,
    async_send_message,
    async_close_stream
}
```

Here you can see the lisp functions that are defined by the file
javascript.rs.

## Generating rust bindings for C functions with bindgen

The crate that calls bindgen is `ng-bindgen` which gets called in
`src/Makefile.in`:

```
DEFINITIONS_FILE=$(rust_srcdir)/crates/emacs/src/definitions.rs
BINDINGS_FILE=$(rust_srcdir)/crates/emacs/src/bindings.rs
GLOBALS_FILE=$(rust_srcdir)/crates/emacs/src/globals.rs
# macuvs.h has a rule to regenerate it which depends on emacs being
# built. That creates a circular dependency which we can break by not
# depending on it as this is not a file that changes under normal
# operation.
$(DEFINITIONS_FILE) $(BINDINGS_FILE) $(GLOBALS_FILE): $(filter-out ./macuvs.h,$(HEADERS)) $(rust_srcdir)/wrapper.h \
							$(rust_srcdir)/ng-bindgen/Cargo.toml \
							$(rust_srcdir)/ng-bindgen/src/main.rs
	$(MKDIR_P) $(dir $@)
	RUSTFLAGS="$(RUSTFLAGS)" \
	EMACS_CFLAGS="$(EMACS_CFLAGS)" \
	$(CARGO_RUN) --release --manifest-path=$(rust_srcdir)/ng-bindgen/Cargo.toml -- $(basename $(notdir $@)) $@
	touch $@
```

Only C functions are listed in `rust_src/wrapper.h` are considered.
We also blacklist several items and define them in rust(for different
reasons).

The bindings created in each build and can be found in the
emacs crate. There are three files:

- bindings.rs: functions, structs, enums and more
- defintions.rs: important types like `EmacsInt` and `USE_LSB_TAG`
- globals.rs: `emacs_globals` and symbols (e.g. `Qnil`)

We ignore these files in git, since they differ depending on the
enabled features.

## remacs-lib(soon ng-docfile)

This crate's purpose is currently only providing the function
`scan_rust_file`. It contains(maybe we removed it by now) additional
functionality that was used in remacs. Maybe we will use the content
at some point again.

The function is called by `scan_file` in `make-docfile.c`. Besides
extracting doc strings from elisp functions, we also use it to find,
generate and add the lisp globals we defined in rust.

```c
/* Read file FILENAME and output its doc strings to stdout.
   Return true if file is found, false otherwise.  */

static void
scan_file (char *filename)
{
  ptrdiff_t len = strlen (filename);

  if (!generate_globals)
    put_filename (filename);
  if (len > 4 && !strcmp (filename + len - 4, ".elc"))
    scan_lisp_file (filename, "rb");
  else if (len > 3 && !strcmp (filename + len - 3, ".el"))
    scan_lisp_file (filename, "r");
  else if (len > 3 && !strcmp (filename + len - 3, ".rs"))
    scan_rust_file (filename, generate_globals, add_global);
  else
    scan_c_file (filename, "r");
}
```
