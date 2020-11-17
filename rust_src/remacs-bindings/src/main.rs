extern crate bindgen;

use std::cmp::max;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::process;

#[cfg(feature = "wide-emacs-int")]
const WIDE_EMACS_INT: bool = true;

#[cfg(not(feature = "wide-emacs-int"))]
const WIDE_EMACS_INT: bool = false;

#[cfg(feature = "ns-impl-gnustep")]
const NS_IMPL_GNUSTEP: bool = true;

#[cfg(not(feature = "ns-impl-gnustep"))]
const NS_IMPL_GNUSTEP: bool = false;

fn integer_max_constant(len: usize) -> &'static str {
    match len {
        1 => "0x7F_i8",
        2 => "0x7FFF_i16",
        4 => "0x7FFFFFFF_i32",
        8 => "0x7FFFFFFFFFFFFFFF_i64",
        16 => "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_i128",
        _ => panic!("nonstandard int size {}", len),
    }
}

#[derive(Eq, PartialEq)]
enum ParseState {
    ReadingGlobals,
    ReadingSymbols,
    Complete,
}

fn generate_definitions(mut file: &File) {
    // let out_path = PathBuf::from(path);


    // signed and unsigned size shall be the same.
    let integer_types = [
        ("libc::c_int", "libc::c_uint", size_of::<libc::c_int>()),
        ("libc::c_long", "libc::c_ulong", size_of::<libc::c_long>()),
        (
            "libc::c_longlong",
            "libc::c_ulonglong",
            size_of::<libc::c_longlong>(),
        ),
    ];
    let actual_ptr_size = size_of::<libc::intptr_t>();
    let usable_integers_narrow = ["libc::c_int", "libc::c_long", "libc::c_longlong"];
    let usable_integers_wide = ["libc::c_longlong"];
    let usable_integers = if !WIDE_EMACS_INT {
        usable_integers_narrow.as_ref()
    } else {
        usable_integers_wide.as_ref()
    };
    let integer_type_item = integer_types
        .iter()
        .find(|&&(n, _, l)| {
            actual_ptr_size <= l && usable_integers.iter().find(|&x| x == &n).is_some()
        })
        .expect("build.rs: intptr_t is too large!");

    let float_types = [("f64", size_of::<f64>())];

    let float_type_item = &float_types[0];

    write!(file, "pub type EmacsInt = {};\n", integer_type_item.0).expect("Write error!");
    write!(file, "pub type EmacsUint = {};\n", integer_type_item.1).expect("Write error!");
    write!(
        file,
        "pub const EMACS_INT_MAX: EmacsInt = {};\n",
        integer_max_constant(integer_type_item.2)
    )
    .expect("Write error!");

    write!(
        file,
        "pub const EMACS_INT_SIZE: EmacsInt = {};\n",
        integer_type_item.2
    )
    .expect("Write error!");

    write!(file, "pub type EmacsDouble = {};\n", float_type_item.0).expect("Write error!");
    write!(
        file,
        "pub const EMACS_FLOAT_SIZE: EmacsInt = {};\n",
        max(float_type_item.1, actual_ptr_size)
    )
    .expect("Write error!");

    if NS_IMPL_GNUSTEP {
        write!(file, "pub type BoolBF = libc::c_uint;\n").expect("Write error!");
    } else {
        // There is no such thing as a libc::cbool
        // See https://users.rust-lang.org/t/is-rusts-bool-compatible-with-c99--bool-or-c-bool/3981
        write!(file, "pub type BoolBF = bool;\n").expect("Write error!");
    }

    let bits = 8; // bits in a byte.
    let gc_type_bits = 3;
    let uint_max_len = integer_type_item.2 * bits;
    let int_max_len = uint_max_len - 1;
    let val_max_len = int_max_len - (gc_type_bits - 1);
    let use_lsb_tag = val_max_len - 1 < int_max_len;
    write!(
        file,
        "pub const USE_LSB_TAG: bool = {};\n",
        if use_lsb_tag { "true" } else { "false" }
    )
    .expect("Write error!");
}

fn generate_globals(mut out_file: &File) {
    let in_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("..")
        .join("src")
        .join("globals.h");
    let in_file = BufReader::new(File::open(in_path).expect("Failed to open globals file"));
    // let out_path = PathBuf::from(path);
    // let mut out_file = File::create(out_path).expect("Failed to create definition file");
    let mut parse_state = ParseState::ReadingGlobals;

    write!(out_file, "#[allow(unused)]\n").expect("Write error!");
    write!(out_file, "#[repr(C)]\n").expect("Write error!");
    write!(out_file, "pub struct emacs_globals {{\n").expect("Write error!");

    for line in in_file.lines() {
        let line = line.expect("Read error!");
        match parse_state {
            ParseState::ReadingGlobals => {
                if line.starts_with("  ") {
                    let mut parts = line.trim().trim_matches(';').split(' ');
                    let vtype = parts.next().unwrap();
                    let vname = parts.next().unwrap().splitn(2, "_").nth(1).unwrap();

                    write!(
                        out_file,
                        "    pub {}: {},\n",
                        vname,
                        match vtype {
                            "EMACS_INT" => "EmacsInt",
                            "bool_bf" => "BoolBF",
                            "Lisp_Object" => "LispObject",
                            t => t,
                        }
                    )
                    .expect("Write error!");
                }
                if line.starts_with('}') {
                    write!(out_file, "}}\n").expect("Write error!");
                    parse_state = ParseState::ReadingSymbols;
                    continue;
                }
            }

            ParseState::ReadingSymbols => {
                if line.trim().starts_with("#define") {
                    let mut parts = line.split(' ');
                    let _ = parts.next().unwrap(); // The #define
                                                   // Remove the i in iQnil
                    let (_, symbol_name) = parts.next().unwrap().split_at(1);
                    let value = parts.next().unwrap();
                    write!(
                        out_file,
                        "pub const {}: LispObject = crate::lisp::LispObject( \
                         {} * (std::mem::size_of::<Lisp_Symbol>() as EmacsInt));\n",
                        symbol_name, value
                    )
                    .expect("Write error in reading symbols stage");
                } else if line.trim().starts_with("_Noreturn") {
                    parse_state = ParseState::Complete
                }
            }

            ParseState::Complete => {
                break;
            }
        };
    }
}

fn run_bindgen(mut file: &File, path: &str) {
    let out_path = PathBuf::from(path);
    let skip = std::env::var_os("SKIP_BINDINGS");
    let cflags = std::env::var_os("EMACS_CFLAGS");
    match cflags {
        None => {
            if out_path.exists() {
                println!("No EMACS_CFLAGS specified, but {:?} already exists so we'll just skip the bindgen step this time.", out_path);
            } else {
                panic!("No EMACS_CFLAGS were specified, and we need them in order to run bindgen.");
            }
        }
        Some(cflags) => {
            let mut builder = bindgen::Builder::default()
                .rust_target(bindgen::RustTarget::Nightly)
                .generate_comments(true);

            let cflags_str = cflags.to_string_lossy();
            let mut processed_args: Vec<String> = Vec::new();
            for arg in cflags_str.split(' ') {
                if arg.starts_with("-I") {
                    // we're running clang from a different directory, so we have to adjust any relative include paths
                    let path = Path::new("../src").join(arg.get(2..).unwrap());
                    let buf = std::fs::canonicalize(path).unwrap();
                    processed_args.push(String::from("-I") + &buf.to_string_lossy());
                } else {
                    if !arg.is_empty() && !arg.starts_with("-M") && !arg.ends_with(".d") {
                        processed_args.push(arg.into());
                    }
                };
            }
            builder = builder.clang_args(processed_args);
            if cfg!(target_os = "windows") {
                builder = builder.clang_arg("-I../nt/inc");
                builder =
                    builder.clang_arg("-Ic:\\Program Files\\LLVM\\lib\\clang\\6.0.0\\include");
            }

            builder = builder
                .clang_arg("-Demacs")
                .clang_arg("-DEMACS_EXTERN_INLINE")
                .header("../rust_src/wrapper.h")
                .generate_inline_functions(true)
                .derive_default(true)
                .ctypes_prefix("::libc")
                // we define these ourselves, for various reasons
                .blacklist_item("Lisp_Object")
                .blacklist_item("emacs_globals")
                .blacklist_item("Q.*") // symbols like Qnil and so on
                .blacklist_item("USE_LSB_TAG")
                .blacklist_item("VALMASK")
                .blacklist_item("PSEUDOVECTOR_FLAG")
                .blacklist_item("Fmapc")
                // these two are found by bindgen on mac, but not linux
                .blacklist_item("EMACS_INT_MAX")
                .blacklist_item("VAL_MAX")
                // this is wallpaper for a bug in bindgen, we don't lose much by it
                // https://github.com/servo/rust-bindgen/issues/687
                .blacklist_item("BOOL_VECTOR_BITS_PER_CHAR")
                // this is wallpaper for a function argument that shadows a static of the same name
                // https://github.com/servo/rust-bindgen/issues/804
                .blacklist_item("face_change")
                // these never return, and bindgen doesn't yet detect that, so we will do them manually
                .blacklist_item("error")
                .blacklist_item("circular_list")
                .blacklist_item("wrong_type_argument")
                .blacklist_item("nsberror")
                .blacklist_item("emacs_abort")
                .blacklist_item("Fsignal")
                .blacklist_item("memory_full")
                .blacklist_item("wrong_choice")
                .blacklist_item("wrong_range")
                // these are defined in data.rs
                // .blacklist_item("Lisp_Fwd")
                // .blacklist_item("Lisp_.*fwd")
                // these are defined in remacs_lib
                .blacklist_item("timespec")
                .blacklist_item("current_timespec")
                .blacklist_item("timex")
                .blacklist_item("clock_adjtime")
                // by default we want C enums to be converted into a Rust module with constants in it
                .default_enum_style(bindgen::EnumVariation::ModuleConsts)
                // enums with only one variant are better as simple constants
                .constified_enum("EMACS_INT_WIDTH")
                .constified_enum("BOOL_VECTOR_BITS_PER_CHAR")
                .constified_enum("BITS_PER_BITS_WORD")
                // TODO(db48x): verify that these enums meet Rust's requirements (primarily that they have no duplicate variants)
                .rustified_enum("Lisp_Misc_Type")
                .rustified_enum("Lisp_Type")
                .rustified_enum("case_action")
                .rustified_enum("face_id")
                .rustified_enum("output_method")
                .rustified_enum("pvec_type")
                .rustified_enum("symbol_redirect")
                .rustified_enum("syntaxcode")
                .rustified_enum("VTermProp")
                .rustified_enum("VTermColorType");

            if cfg!(target_os = "windows") {
                builder = builder
                    .rustified_enum("_HEAP_INFORMATION_CLASS")
                    .rustified_enum("SECURITY_IMPERSONATION_LEVEL")
                    .rustified_enum("TOKEN_INFORMATION_CLASS");
            }

            let bindings = builder
                .rustfmt_bindings(true)
                .rustfmt_configuration_file(std::fs::canonicalize("rustfmt.toml").ok())
                .generate()
                .expect("Unable to generate bindings");

            // https://github.com/rust-lang-nursery/rust-bindgen/issues/839
            let source = bindings.to_string();
            let re = regex::Regex::new(
                r"pub use self\s*::\s*gnutls_cipher_algorithm_t as gnutls_cipher_algorithm\s*;",
            );
            let munged = re.unwrap().replace_all(&source, "");
            // let file = File::create(out_path);
            file.write_all(munged.into_owned().as_bytes())
                .unwrap();
        }
    }
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let file = File::create(&args[1]).expect("Failed to create definition file");

    write!(&file, r#"#![allow(unused)]

//! This module contains all FFI declarations.
//!
//! These types and constants are generated at build time to mimic how they are
//! in C:
//!
//! - `EmacsInt`
//! - `EmacsUint`
//! - `EmacsDouble`
//! - `EMACS_INT_MAX`
//! - `EMACS_INT_SIZE`
//! - `EMACS_FLOAT_SIZE`
//! - `GCTYPEBITS`
//! - `USE_LSB_TAG`
//! - `BoolBF`

use libc::{{self, c_char, c_void, ptrdiff_t, c_int}};
use std::mem;

use libc::timespec;
use remacs_lib::current_timespec;

use crate::lisp::LispObject;
"#).expect("Write error!");

    generate_definitions(&file);
    write!(&file, r#"
type Lisp_Object = LispObject;
"#).expect("Write error!");
    run_bindgen(&file, &args[1]);
    generate_globals(&file);

writeln!(&file, r#"
pub const VAL_MAX: EmacsInt = (EMACS_INT_MAX >> (GCTYPEBITS - 1));
pub const VALMASK: EmacsInt = [VAL_MAX, -(1 << GCTYPEBITS)][USE_LSB_TAG as usize];
pub const PSEUDOVECTOR_FLAG: usize = 0x4000_0000_0000_0000;

// These signal an error, therefore are marked as non-returning.
extern "C" {{
    pub fn circular_list(tail: Lisp_Object) -> !;
    pub fn wrong_type_argument(predicate: Lisp_Object, value: Lisp_Object) -> !;
    // defined in eval.c, where it can actually take an arbitrary
    // number of arguments.
    // TODO: define a Rust version of this that uses Rust strings.
    pub fn error(m: *const u8, ...) -> !;
    pub fn memory_full(nbytes: libc::size_t) -> !;
    pub fn wrong_choice(choice: LispObject, wrong: LispObject) -> !;
    pub fn wrong_range(min: LispObject, max: LispObject, wrong: LispObject) -> !;
}}

#[repr(C)]
pub enum BoolVectorOp {{
    BoolVectorExclusiveOr,
    BoolVectorUnion,
    BoolVectorIntersection,
    BoolVectorSetDifference,
    BoolVectorSubsetp,
}}

// bindgen apparently misses these, for various reasons
extern "C" {{
    // these weren't declared in a header, for example
    pub static Vprocess_alist: Lisp_Object;
    pub fn update_buffer_defaults(objvar: *mut LispObject, newval: LispObject);
    pub fn concat(
        nargs: ptrdiff_t,
        args: *mut LispObject,
        target_type: Lisp_Type,
        last_special: bool,
    ) -> LispObject;
    pub fn map_keymap_item(
        fun: map_keymap_function_t,
        args: LispObject,
        key: LispObject,
        val: LispObject,
        data: *const c_void,
    );
    pub fn map_keymap_char_table_item(args: LispObject, key: LispObject, val: LispObject);
    pub static initial_obarray: LispObject;
    pub static oblookup_last_bucket_number: libc::size_t;
    pub fn scan_lists(
        from: EmacsInt,
        count: EmacsInt,
        depth: EmacsInt,
        sexpflag: bool,
    ) -> LispObject;
    pub fn read_minibuf(
        map: Lisp_Object,
        initial: Lisp_Object,
        prompt: Lisp_Object,
        expflag: bool,
        histvar: Lisp_Object,
        histpos: Lisp_Object,
        defalt: Lisp_Object,
        allow_props: bool,
        inherit_input_method: bool,
    ) -> Lisp_Object;
    pub static minibuf_prompt: LispObject;
    pub fn add_process_read_fd(fd: libc::c_int);
    #[cfg(windows)]
    pub fn file_attributes_c(filename: LispObject, id_format: LispObject) -> LispObject;
    pub fn getloadaverage(loadavg: *mut libc::c_double, nelem: libc::c_int) -> libc::c_int;
    #[cfg(unix)]
    pub fn file_attributes_c_internal(
        name: *const c_char,
        directory: LispObject,
        filename: LispObject,
        id_format: LispObject,
    ) -> LispObject;
    #[cfg(unix)]
    pub fn filemode_string(f: LispObject) -> LispObject;

    pub fn unchain_both(b: *mut Lisp_Buffer, ov: LispObject);
    pub fn emacs_get_tty_pgrp(p: *mut Lisp_Process) -> libc::pid_t;
    pub fn update_buffer_properties(start: ptrdiff_t, end: ptrdiff_t);
    pub fn set_window_hscroll(w: *mut Lisp_Window, hscroll: EMACS_INT) -> Lisp_Object;
    pub fn scroll_command(n: Lisp_Object, direction: libc::c_int);
    pub fn bool_vector_binop_driver(
        a: Lisp_Object,
        b: Lisp_Object,
        dest: Lisp_Object,
        op: BoolVectorOp,
    ) -> Lisp_Object;
}}

// Max value for the first argument of wait_reading_process_output.
pub const WAIT_READING_MAX: i64 = i64::max_value();

// In order to use `lazy_static!` with LispSubr, it must be Sync. Raw
// pointers are not Sync, but it isn't a problem to define Sync if we
// never mutate LispSubr values. If we do, we will need to create
// these objects at runtime, perhaps using forget().
//
// Based on http://stackoverflow.com/a/28116557/509706
unsafe impl Sync for Lisp_Subr {{}}
unsafe impl Sync for Aligned_Lisp_Subr {{}}
unsafe impl Sync for crate::lisp::LispSubrRef {{}}

macro_rules! export_lisp_fns {{
    ($($(#[$($meta:meta),*])* $f:ident),+) => {{
	pub fn rust_init_syms() {{
	    #[allow(unused_unsafe)] // just in case the block is empty
	    unsafe {{
		$(
		    $(#[$($meta),*])* crate::remacs_sys::defsubr(concat_idents!(S, $f).as_mut());
		)+
	    }}
	}}
    }}
}}

pub type Lisp_Buffer = buffer;
pub type Lisp_Font_Object = font;
pub type Lisp_Font_Spec = font_spec;
pub type Lisp_Frame = frame;
pub type Lisp_Glyph = glyph;
pub type Lisp_Terminal = terminal;
pub type Lisp_Window = window;
pub type Lisp_Interval = interval;

#[repr(C)]
pub struct Lisp_Vectorlike {{
    pub header: vectorlike_header,
    // shouldn't look at the contents without knowing the structure...
}}

// No C equivalent.  Generic type for a vectorlike with one or more
// LispObject slots after the header.
#[repr(C)]
pub struct Lisp_Vectorlike_With_Slots {{
    pub header: vectorlike_header,
    // actually any number of items... not sure how to express this
    pub contents: __IncompleteArrayField<Lisp_Object>,
}}

//// declare this ourselves so that the arg isn't mutable
//extern "C" {{
//    pub fn staticpro(arg1: *const Lisp_Object);
//}}
"#).expect("Write error!");
}
