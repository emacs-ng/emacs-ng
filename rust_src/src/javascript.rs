use crate::lisp::LispObject;
use crate::multibyte::LispStringRef;
use crate::remacs_sys::make_string_from_utf8;
use lazy_static::lazy_static;
use remacs_macros::lisp_fn;
use rusty_v8 as v8;
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

#[lisp_fn]
pub fn eval_js(string_obj: LispStringRef) -> LispObject {
    let isolate = MAIN.isolate();

    // Create a stack-allocated handle scope.
    let handle_scope = &mut v8::HandleScope::new(isolate);

    // Create a new context.
    let context = v8::Context::new(handle_scope);

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
