#![feature(lazy_cell)]
#![recursion_limit = "256"]

use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use std::sync::LazyLock;

mod function;

#[proc_macro_attribute]
pub fn lisp_fn(attr_ts: TokenStream, fn_ts: TokenStream) -> TokenStream {
    let fn_item = syn::parse(fn_ts.clone()).unwrap();
    let function = function::parse(&fn_item).unwrap();
    let lisp_fn_args = match lisp_util::parse_lisp_fn(
        &attr_ts.to_string(),
        &function.name,
        function.fntype.def_min_args(),
    ) {
        Ok(v) => v,
        Err(e) => panic!("Invalid lisp_fn attribute: {}", e),
    };

    let mut cargs = proc_macro2::TokenStream::new();
    let mut rargs = proc_macro2::TokenStream::new();
    let mut body = proc_macro2::TokenStream::new();
    let max_args = function.args.len() as i16;
    let intspec = if let Some(intspec) = lisp_fn_args.intspec {
        let cbyte_intspec = CByteLiteral(intspec.as_str());
        quote! { (#cbyte_intspec).as_ptr() as *const libc::c_char }
    } else {
        quote! { std::ptr::null() }
    };

    match function.fntype {
        function::LispFnType::Normal(_) => {
            for ident in function.args {
                let arg = quote! { #ident: emacs_sys::lisp::LispObject, };
                cargs.extend(arg);

                let arg = quote! { (#ident).into(), };
                rargs.extend(arg);
            }
        }
        function::LispFnType::Many => {
            let args = quote! {
                nargs: libc::ptrdiff_t,
                args: *mut emacs_sys::lisp::LispObject,
            };
            cargs.extend(args);

            let b = quote! {
                let args = unsafe {
                    std::slice::from_raw_parts_mut::<emacs_sys::lisp::LispObject>(
                        args, nargs as usize)
                };
            };
            body.extend(b);

            let arg = quote! { unsafe { std::mem::transmute(args) } };
            rargs.extend(arg);
        }
    }

    let cname = lisp_fn_args.c_name;
    let sname = concat_idents("S", &cname);
    let fname = concat_idents("F", &cname);
    let srname = concat_idents("SR", &cname);
    let rname = function.name;
    let min_args = lisp_fn_args.min;
    let mut windows_header = quote! {};

    let functype = if lisp_fn_args.unevalled {
        quote! { aUNEVALLED }
    } else {
        match function.fntype {
            function::LispFnType::Normal(_) => match max_args {
                0 => quote! { a0 },
                1 => quote! { a1 },
                2 => quote! { a2 },
                3 => quote! { a3 },
                4 => quote! { a4 },
                5 => quote! { a5 },
                6 => quote! { a6 },
                7 => quote! { a7 },
                8 => quote! { a8 },
                _ => panic!("max_args too high"),
            },
            function::LispFnType::Many => quote! { aMANY },
        }
    };

    let max_args = if lisp_fn_args.unevalled {
        quote! { -1 }
    } else {
        match function.fntype {
            function::LispFnType::Normal(_) => quote! { #max_args },
            function::LispFnType::Many => quote! { emacs_sys::lisp::MANY  },
        }
    };
    let symbol_name = CByteLiteral(&lisp_fn_args.name);

    if cfg!(windows) {
        windows_header = quote! {
            | (std::mem::size_of::<emacs_sys::bindings::Lisp_Subr>()
               / std::mem::size_of::<emacs_sys::bindings::EmacsInt>()) as libc::ptrdiff_t
        };
    }

    let tokens = quote! {
        #[no_mangle]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        #[allow(clippy::transmute_ptr_to_ptr)]
        #[allow(clippy::diverging_sub_expression)]
        pub extern "C" fn #fname(#cargs) -> emacs_sys::lisp::LispObject {
            #body

            let ret = #rname(#rargs);
            #[allow(unreachable_code)]
            emacs_sys::lisp::LispObject::from(ret)
        }

    #[no_mangle]
    pub static mut #srname: std::mem::MaybeUninit<emacs_sys::bindings::Aligned_Lisp_Subr>
        = std::mem::MaybeUninit::<emacs_sys::bindings::Aligned_Lisp_Subr>::uninit();

        pub static #sname: std::sync::LazyLock<emacs_sys::lisp::LispSubrRef>
        = std::sync::LazyLock::new(|| {
            let mut subr = emacs_sys::bindings::Aligned_Lisp_Subr::default();
            unsafe {
                let mut subr_ref = subr.s.as_mut();
                subr_ref.header = emacs_sys::bindings::vectorlike_header {
                    size: ((emacs_sys::bindings::pvec_type::PVEC_SUBR as libc::ptrdiff_t)
                        << emacs_sys::bindings::More_Lisp_Bits::PSEUDOVECTOR_AREA_BITS)
                    #windows_header,
                };
                subr_ref.function = emacs_sys::bindings::Lisp_Subr__bindgen_ty_1 {
                    #functype: (Some(self::#fname))
                };
                subr_ref.min_args = #min_args;
                subr_ref.max_args = #max_args;
                subr_ref.symbol_name = (#symbol_name).as_ptr() as *const libc::c_char;
                *subr_ref.intspec.string.as_mut() = #intspec;
                subr_ref.doc = 0;

                std::ptr::copy_nonoverlapping(&subr, #srname.as_mut_ptr(), 1);
                emacs_sys::lisp::ExternalPtr::new(#srname.as_mut_ptr())
            }
        });
    };

    // we could put #fn_item into the quoted code above, but doing so
    // drops all of the line numbers on the floor and causes the
    // compiler to attribute any errors in the function to the macro
    // invocation instead.
    let tokens: TokenStream = tokens.into();
    tokens.into_iter().chain(fn_ts.into_iter()).collect()
}

#[proc_macro_attribute]
pub fn async_stream(_attr_ts: TokenStream, fn_ts: TokenStream) -> TokenStream {
    let fn_item = syn::parse(fn_ts.clone()).unwrap();
    let function = function::parse(&fn_item).unwrap();
    let name = &function.name;
    let async_name = concat_idents("call_", &name.to_string());

    let tokens = quote! {

    #[lisp_fn]
    pub fn #async_name (handler: emacs_sys::lisp::LispObject) -> emacs_sys::lisp::LispObject {
        crate::fns::rust_worker(handler, |s| {
        ::futures::executor::block_on(#name(s))
        })
    }

    };

    let result_tokens: TokenStream = tokens.into();
    result_tokens.into_iter().chain(fn_ts.into_iter()).collect()
}

struct CByteLiteral<'a>(&'a str);

impl<'a> quote::ToTokens for CByteLiteral<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"["\\]"#).unwrap());
        let s = RE.replace_all(self.0, |caps: &regex::Captures| {
            format!("\\x{:x}", u32::from(caps[0].chars().next().unwrap()))
        });
        let identifier = format!(r#"b"{}\0""#, s);
        let expr = syn::parse_str::<syn::Expr>(&identifier).unwrap();
        tokens.extend(quote! { #expr });
    }
}

fn concat_idents(lhs: &str, rhs: &str) -> syn::Ident {
    use proc_macro2::Span;
    syn::Ident::new(format!("{}{}", lhs, rhs).as_str(), Span::call_site())
}
