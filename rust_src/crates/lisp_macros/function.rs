use syn;

use quote::quote;
use syn::Abi;
use syn::FnArg;
use syn::Ident;
use syn::Item;
use syn::ItemFn;
use syn::Pat;
use syn::PatIdent;
use syn::PatType;
use syn::Receiver;
use syn::Signature;
use syn::Type;
use syn::TypePath;
use syn::TypeReference;
use syn::TypeSlice;

type Result<T> = ::std::result::Result<T, &'static str>;

pub enum LispFnType {
    /// A normal function with given max. number of arguments
    Normal(i16),
    /// A function taking an arbitrary amount of arguments as a slice
    Many,
}

impl LispFnType {
    pub fn def_min_args(&self) -> i16 {
        match *self {
            LispFnType::Normal(n) => n,
            LispFnType::Many => 0,
        }
    }
}

pub struct Function {
    /// The function name
    pub name: Ident,

    /// The argument type
    pub fntype: LispFnType,

    /// The function header
    pub args: Vec<Ident>,
}

pub fn parse(item: &Item) -> Result<Function> {
    match item {
        Item::Fn(ItemFn {
            sig,
            // ref decl,
            ..
        }) => {
            if sig.unsafety.is_some() {
                return Err("lisp functions cannot be `unsafe`");
            }

            if sig.constness.is_some() {
                return Err("lisp functions cannot be `const`");
            }

            if !is_rust_abi(&sig.abi) {
                return Err("lisp functions can only use \"Rust\" ABI");
            }

            let args = sig
                .inputs
                .iter()
                .map(get_fn_arg_ident_ty)
                .collect::<Result<_>>()?;

            Ok(Function {
                name: sig.ident.clone(),
                fntype: parse_function_type(&sig)?,
                args: args,
            })
        }
        _ => Err("`lisp_fn` attribute can only be used on functions"),
    }
}

fn is_rust_abi(abi: &Option<Abi>) -> bool {
    match *abi {
        Some(Abi { name: Some(_), .. }) => false,
        Some(Abi { name: None, .. }) => true,
        None => true,
    }
}

fn get_fn_arg_ident_ty(fn_arg: &FnArg) -> Result<Ident> {
    match fn_arg {
        FnArg::Typed(PatType { pat, .. }) => match pat.as_ref() {
            Pat::Ident(PatIdent { ref ident, .. }) => Ok(ident.clone()),
            _ => Err("invalid function argument"),
        },
        _ => Err("invalid function argument"),
    }
}

fn parse_function_type(fndecl: &Signature) -> Result<LispFnType> {
    let nargs = fndecl.inputs.len() as i16;
    for fnarg in &fndecl.inputs {
        match fnarg {
            FnArg::Typed(PatType { ty, .. }) | FnArg::Receiver(Receiver { ty, .. }) => {
                match parse_arg_type(ty.as_ref()) {
                    ArgType::LispObject => {}
                    ArgType::LispObjectSlice => {
                        if fndecl.inputs.len() != 1 {
                            return Err("`[LispObject]` cannot be mixed in with other types");
                        }
                        return Ok(LispFnType::Many);
                    }
                    ArgType::Other => {}
                }
            }
        }
    }
    Ok(LispFnType::Normal(nargs))
}

enum ArgType {
    LispObject,
    LispObjectSlice,
    Other,
}

fn parse_arg_type(fn_arg: &Type) -> ArgType {
    if is_lisp_object(fn_arg) {
        ArgType::LispObject
    } else {
        match *fn_arg {
            Type::Reference(TypeReference {
                elem: ref ty,
                ref lifetime,
                ..
            }) => {
                if lifetime.is_some() {
                    ArgType::Other
                } else {
                    match **ty {
                        Type::Slice(TypeSlice { elem: ref ty, .. }) => {
                            if is_lisp_object(&**ty) {
                                ArgType::LispObjectSlice
                            } else {
                                ArgType::Other
                            }
                        }
                        _ => ArgType::Other,
                    }
                }
            }
            _ => ArgType::Other,
        }
    }
}

fn is_lisp_object(ty: &Type) -> bool {
    match *ty {
        Type::Path(TypePath {
            qself: None,
            ref path,
        }) => {
            let str_path = format!("{}", quote!(#path));
            str_path == "LispObject"
                || str_path == "lisp :: LispObject"
                || str_path == ":: lisp :: LispObject"
        }
        _ => false,
    }
}
