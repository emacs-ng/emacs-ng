//! Parse the #[lisp_fn] macro.

use std::fmt::Display;

use proc_macro2::TokenStream;
use std::str::FromStr;

use darling::ast::NestedMeta;
use darling::FromMeta;

/// Arguments of the lisp_fn attribute.
#[derive(FromMeta, Default)]
struct LispFnArgsRaw {
    /// Desired Lisp name of the function.
    /// If not given, derived as the Rust name with "_" -> "-".
    #[darling(default)]
    name: Option<String>,
    /// Desired C name of the related statics (with F and S appended).
    /// If not given, same as the Rust name.
    #[darling(default)]
    c_name: Option<String>,
    /// Minimum number of required arguments.
    /// If not given, all arguments are required for normal functions,
    /// and no arguments are required for MANY functions.
    #[darling(default)]
    min: Option<i16>,
    /// The interactive specification. This may be a normal prompt
    /// string, such as `"bBuffer: "` or an elisp form as a string.
    /// If the function is not interactive, this should be None.
    #[darling(default)]
    intspec: Option<String>,
    /// Whether unevalled or not.
    #[darling(default)]
    unevalled: Option<String>,
}

impl LispFnArgsRaw {
    fn convert<D>(self, def_name: &D, def_min_args: i16) -> Result<LispFnArgs, String>
    where
        D: Display + ?Sized,
    {
        Ok(LispFnArgs {
            name: self
                .name
                .unwrap_or_else(|| def_name.to_string().replace("_", "-")),
            c_name: self.c_name.unwrap_or_else(|| def_name.to_string()),
            min: self.min.unwrap_or(def_min_args),
            intspec: self.intspec,
            unevalled: if let Some(b) = self.unevalled {
                b.parse().map_err(|_| "invalid \"unevalled\" argument")?
            } else {
                false
            },
        })
    }
}

#[derive(Debug)]
pub struct LispFnArgs {
    pub name: String,
    pub c_name: String,
    pub min: i16,
    pub intspec: Option<String>,
    pub unevalled: bool,
}

pub fn parse_lisp_fn<D>(src: &str, def_name: &D, def_min_args: i16) -> Result<LispFnArgs, String>
where
    D: Display + ?Sized,
{
    TokenStream::from_str(&src)
        .map_err(|e| e.to_string())
        .and_then(|args| NestedMeta::parse_meta_list(args.into()).map_err(|e| e.to_string()))
        .and_then(|v| LispFnArgsRaw::from_list(&v).map_err(|e| e.to_string()))
        .and_then(|v| v.convert(def_name, def_min_args))
}

#[macro_export]
macro_rules! export_lisp_fns {
    ($($(#[$($meta:meta),*])* $f:ident),+) => {
	pub fn rust_init_syms() {
	    #[allow(unused_unsafe)] // just in case the block is empty
	    unsafe {
		$(
		    $(#[$($meta),*])* emacs_sys::bindings::defsubr(
			concat_idents!(S, $f).as_ptr() as *mut emacs_sys::bindings::Aligned_Lisp_Subr
		    );
		)+
	    }
	}
    }
}
