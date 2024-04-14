//! Extract Rust docstrings from source files for Emacs' DOC file.

#![allow(clippy::cognitive_complexity)]

use libc::c_char;
use libc::c_int;
use regex::Regex;

use anyhow::format_err;
use anyhow::Context;
use anyhow::Result;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::mem;
use std::ptr;
use std::sync::LazyLock;

use lisp_util::parse_lisp_fn;

#[allow(dead_code)]
const INVALID: c_int = 0;
#[allow(dead_code)]
const LISP_OBJECT: c_int = 1;
#[allow(dead_code)]
const EMACS_INTEGER: c_int = 2;
#[allow(dead_code)]
const BOOLEAN: c_int = 3;
#[allow(dead_code)]
const SYMBOL: c_int = 4;
#[allow(dead_code)]
const FUNCTION: c_int = 5;

type AddGlobalFn = ::std::option::Option<
    unsafe extern "C" fn(c_int, *const c_char, c_int, *const c_char) -> *const (),
>;

#[macro_export]
macro_rules! printf {
    ($str:expr) => {{
        let c_str = CString::new($str).unwrap();
        let str_ptr = c_str.as_ptr() as *const ::libc::c_char;
        unsafe {
            libc::printf(str_ptr);
        }
    }};
    ($fmtstr:expr, $($arg:expr),*) => {{
        let formatted = format!($fmtstr, $($arg),*);
        let c_str = CString::new(formatted).unwrap();
        let str_ptr = c_str.as_ptr() as *const ::libc::c_char;
        unsafe {
            libc::printf(str_ptr);
        }
    }};
}

/// This function is called by lib-src/scan_file and runs with make-docfile
/// in src/Makefile.
/// We have to ensure that all necessary rust paths will be considered to
/// generate the defined globals and extract their docstrings so they can be
/// used in elisp.
#[no_mangle]
pub unsafe extern "C" fn scan_rust_file(
    filename: *const c_char,
    generate_globals: c_int,
    add_global_fp: AddGlobalFn,
) {
    match scan_rust_file1(filename, generate_globals, add_global_fp) {
        Err(e) => eprintln!("scan rust file failed with error: {e:?}"),
        _ => {}
    };
}

pub fn scan_rust_file1(
    filename: *const c_char,
    generate_globals: c_int,
    add_global_fp: AddGlobalFn,
) -> Result<()> {
    let add_global1 = add_global_fp.ok_or(format_err!("AddGlobalFn None"))?;
    let add_global = |_type: c_int, name: *const c_char, value: c_int, svalue: *const c_char| {
        unsafe { add_global1(_type, name, value, svalue) };
    };
    let filename = unsafe { CStr::from_ptr(filename) };
    let filename = filename
        .to_str()
        .with_context(|| format!("Non utf8 filename {:?}", filename))?;
    let fp = BufReader::new(
        File::open(&*filename).with_context(|| format!("Failed to open file {:?}", &*filename))?,
    );

    let mut in_docstring = false;
    let mut docstring = String::new();
    let mut docstring_usage = String::new();
    let mut attribute = String::new();

    let mut line_iter = fp.lines();

    let mut in_lisp_fn = false;

    while let Some(line) = line_iter.next() {
        let line = line?;
        let line = line.trim();

        // Collect a whole docstring
        if line.starts_with("///") {
            if !in_docstring {
                attribute.clear();
                docstring_usage.clear();
                docstring.clear();
            }
            in_docstring = true;
            if line.starts_with("/// usage: (") {
                docstring = format!("{}\n", docstring.trim_end());
                let begin = &line[11..];
                // Now find the first space after the function name. If there is a space
                // capture the rest of the usage text.
                // The function name is dropped either way.
                if let Some(mut pos) = begin.find(' ') {
                    pos += 11;
                    docstring_usage.push_str(&line[pos..]);
                }
            } else {
                docstring.push_str(line[3..].trim_start());
                docstring.push('\n');
            }
        } else {
            in_docstring = false;
        }

        if line == "#[lisp_fn(" {
            attribute = line.to_owned();
            in_lisp_fn = true;
            continue;
        } else if line.starts_with("#[lisp_fn") {
            attribute = line.to_owned();
            continue;
        }

        if in_lisp_fn {
            if line == ")]" {
                attribute += line;
                in_lisp_fn = false;
                continue;
            } else {
                attribute += line;
                continue;
            }
        }

        if line.starts_with("pub fn ") || line.starts_with("fn ") {
            if attribute.is_empty() {
                // Not a #[lisp_fn]
                continue;
            }
            let attribute = mem::replace(&mut attribute, String::new());
            let mut split = line.split('(');
            let name = split
                .next()
                .ok_or(format_err!("split.next none"))?
                .split_whitespace()
                .last()
                .ok_or(format_err!("split whitespace last None"))?;

            if name.starts_with('$') {
                // Macro; do not use it
                continue;
            }

            // Read lines until the closing paren
            let mut sig = split
                .next()
                .ok_or(format_err!("split.next none"))?
                .to_string();
            while !sig.contains(')') {
                sig.extend(line_iter.next().ok_or(format_err!("split.next none"))?);
            }
            let sig = sig
                .split(')')
                .next()
                .ok_or(format_err!("split.next none"))?;
            let has_many_args = sig.contains("&mut") || sig.contains("&[");

            // Split arg names and types
            let splitters = [':', ','];
            let args = sig.split_terminator(&splitters[..]).collect::<Vec<_>>();

            let nargs = args.len() / 2;
            let def_min_args = if has_many_args { 0 } else { nargs as i16 };
            // FIXME using regex, removing the outter [list_fn(..)]
            // or check out the way Rust turns source code into ast(tokenstream)
            let mut attribute = attribute;
            if attribute.starts_with("#[lisp_fn(") {
                attribute = attribute.replace("#[lisp_fn(", "");
            }
            if attribute.starts_with("#[lisp_fn") {
                attribute = attribute.replace("#[lisp_fn", "");
            }
            if attribute.ends_with(")]") {
                attribute = attribute.replace(")]", "");
            }
            if attribute.ends_with("]") {
                attribute = attribute.replace("]", "");
            }
            let attr_props = parse_lisp_fn(&attribute, name, def_min_args)
                .map_err(|e| format_err!("Invalid #[lisp_fn] macro ({}): {}", attribute, e))?;

            if generate_globals != 0 {
                let c_name_str = CString::new(format!("F{}", attr_props.c_name))
                    .with_context(|| format!("{:?} c_name is null", attr_props))?;
                // -1 is MANY
                // -2 is UNEVALLED
                let maxargs = if has_many_args { -1 } else { nargs as c_int };
                add_global(FUNCTION, c_name_str.as_ptr(), maxargs, ptr::null());
            } else {
                // Create usage line (fn ARG1 ...) from signature if necessary
                if docstring_usage.is_empty() {
                    for (i, chunk) in args.chunks(2).enumerate() {
                        if chunk[1].contains("&mut") || chunk[1].contains("&[") {
                            docstring_usage.push(' ');
                            docstring_usage.push_str("&rest");
                        } else if i == attr_props.min as usize {
                            docstring_usage.push(' ');
                            docstring_usage.push_str("&optional");
                        }
                        let argname = chunk[0]
                            .trim()
                            .trim_start_matches("mut ")
                            .trim()
                            .to_uppercase()
                            .replace("_", "-");
                        docstring_usage.push(' ');
                        docstring_usage.push_str(&argname);
                    }
                    docstring_usage.push(')');
                }
                // Print contents for docfile to stdout
                printf!(
                    "\x1fF{}\n{}\n(fn{}",
                    attr_props.name,
                    docstring,
                    docstring_usage
                );
            }
        } else if line.starts_with("def_lisp_sym!(") {
            static RE: LazyLock<Regex> = LazyLock::new(|| {
                Regex::new(r#"def_lisp_sym!\((.+?),\s+"(.+?)"\);"#)
                    .map_err(|e| format_err!("Failed to create regext: {:?}", e))
                    .unwrap()
            });
            let caps = RE.captures(line).ok_or(format_err!("No Regex captures"))?;
            let name = CString::new(&caps[1])?;
            let value = CString::new(&caps[2])?;
            add_global(SYMBOL, name.as_ptr(), 0, value.as_ptr());
        } else if line.starts_with("defvar_") {
            // defvar_lisp!(f_Vpost_self_insert_hook, "post-self-insert-hook", Qnil);
            // defvar_kboard!(Vlast_command_, "last-command");
            static RE: LazyLock<Regex> = LazyLock::new(|| {
                Regex::new(r#"defvar_(.+?)!\((.+?),\s+"(.+?)"(?:,\s+(.+?))?\);"#)
                    .map_err(|e| format_err!("Failed to create regext: {:?}", e))
                    .unwrap()
            });
            for caps in RE.captures_iter(line) {
                if generate_globals != 0 {
                    let kindstr = &caps[1];
                    let kind = match kindstr {
                        "lisp" => LISP_OBJECT,
                        "lisp_nopro" => LISP_OBJECT,
                        "bool" => BOOLEAN,
                        "int" => EMACS_INTEGER,
                        "per_buffer" => INVALID, // not really invalid, we just skip them here
                        "kboard" => INVALID,
                        _ => panic!("unknown macro 'defvar_{}' found; either you have a typo in '{}' or you need to update docfile.rs`",
                                    kindstr, filename),
                    };
                    if kind != INVALID {
                        let field_name = &caps[2];
                        assert!(!field_name.starts_with("f_"));
                        let field_name = CString::new(&caps[2])?;
                        add_global(kind, field_name.as_ptr(), 0, ptr::null());
                    }
                } else {
                    let lisp_name = &caps[3];
                    printf!("\x1fV{}\n{}", lisp_name, docstring)
                }
            }
        }
    }
    Ok(())
}
