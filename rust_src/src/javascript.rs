use crate::lisp::LispObject;
use crate::multibyte::LispStringRef;
use crate::remacs_sys::{intern_c_string, make_string_from_utf8, Ffuncall};
use lazy_static::lazy_static;
use remacs_macros::lisp_fn;
use rusty_v8 as v8;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::ffi::CString;

static mut MAIN_ISOLATE: std::mem::MaybeUninit<v8::OwnedIsolate> =
    std::mem::MaybeUninit::<v8::OwnedIsolate>::uninit();

struct EmacsIsolate;

lazy_static! {
    static ref MAIN: EmacsIsolate = {
        {
            let platform = v8::new_default_platform().unwrap();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
            let isolate = v8::Isolate::new(v8::CreateParams::default());
            unsafe { MAIN_ISOLATE.write(isolate) };
            EmacsIsolate {}
        }
    };
}

impl EmacsIsolate {
    fn isolate(&self) -> &'static mut v8::Isolate {
        unsafe { &mut *MAIN_ISOLATE.as_mut_ptr() }
    }
}

/*
var lisp = new Proxy({}, {
  get: function(target, key) {
    return function() {
      _lisp_funcall.apply(key, arguments);
    }
  }
});

in Rust
fn _lisp_funcall(key: String, args: Vec<_>) {
    let func = intern!(key);
    let result = ffuncall(key, args);
    return result;
}

 */

pub fn lisp_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let message = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    let lisp_args = args
        .get(1)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    let cstr = CString::new(message).expect("Failure of CString");
    let interned = unsafe { intern_c_string(cstr.as_ptr()) };
    let mut args = vec![interned];
    let results = unsafe { Ffuncall(args.len().try_into().unwrap(), args.as_mut_ptr()) };

    //    let json = crate::parsing::json_se(&[results]);
    let string_o: LispStringRef = results.into();
    let string = string_o.to_utf8();
    let r = v8::Local::<v8::Value>::try_from(v8::String::new(scope, &string).unwrap()).unwrap();
    retval.set(r);
}

#[lisp_fn]
pub fn eval_js(string_obj: LispStringRef) -> LispObject {
    let isolate = MAIN.isolate();

    // Create a stack-allocated handle scope.
    let handle_scope = &mut v8::HandleScope::new(isolate);

    let global = v8::ObjectTemplate::new(handle_scope);
    global.set(
        v8::String::new(handle_scope, "lisp_invoke").unwrap().into(),
        v8::FunctionTemplate::new(handle_scope, lisp_callback).into(),
    );

    let context = v8::Context::new_from_template(handle_scope, global);

    // Enter the context for compiling and running the hello world script.
    let scope = &mut v8::ContextScope::new(handle_scope, context);

    // Create a string containing the JavaScript source code.
    let string = string_obj.to_utf8();
    let code = v8::String::new(scope, &string).unwrap();

    // Compile the source code.
    let script = v8::Script::compile(scope, code, None).unwrap();
    // Run the script to get the result.
    let result = script.run(scope).unwrap();

    // Convert the result to a string and print it.
    let result = result.to_string(scope).unwrap();
    let result_string = result.to_rust_string_lossy(scope);
    let len = result_string.len();
    let c_content = CString::new(result_string).expect("Failed to allocate");
    unsafe { make_string_from_utf8(c_content.as_ptr(), len.try_into().unwrap()) }
}

include!(concat!(env!("OUT_DIR"), "/javascript_exports.rs"));
