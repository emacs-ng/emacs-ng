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

pub fn lisp_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let mut lisp_args = vec![];
    let len = args.length();

    let message = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope)
        .replace("_", "-");
    let cstr = CString::new(message).expect("Failure of CString");
    let interned = unsafe { intern_c_string(cstr.as_ptr()) };
    lisp_args.push(interned);

    for i in 1..len {
        let arg = args
            .get(i)
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope);

        if let Ok(deser) = crate::parsing::deser(&arg, true) {
            lisp_args.push(deser);
        } else {
        }
    }

    let results = unsafe { Ffuncall(lisp_args.len().try_into().unwrap(), lisp_args.as_mut_ptr()) };
    // LOGIC, attempt to se, with a version of se that returns an error,
    // if this can't se, it is a proxy, and we will treat it as such.
    if let Ok(json) = crate::parsing::ser(results) {
        let r = v8::Local::<v8::Value>::try_from(v8::String::new(scope, &json).unwrap()).unwrap();
        retval.set(r);
    } else {
        // @TODO, FIXME
        // This is NOT how to implement proxies! Use the proper v8 API
        // for setting a real proxy.
        let obj = v8::Object::new(scope);
        let key = v8::String::new(scope, "__proxy__").unwrap();
        let value = v8::String::new(scope, &results.to_C_unsigned().to_string()).unwrap();
        obj.set(
            scope,
            v8::Local::<v8::Value>::try_from(key).unwrap(),
            v8::Local::<v8::Value>::try_from(value).unwrap(),
        );
        let json_result =
            v8::json::stringify(scope, v8::Local::<v8::Value>::try_from(obj).unwrap()).unwrap();
        let r = v8::Local::<v8::Value>::try_from(json_result).unwrap();
        retval.set(r);
    }
}

#[lisp_fn]
pub fn eval_js(string_obj: LispStringRef) -> LispObject {
    js_eval(string_obj.to_utf8())
}

#[lisp_fn]
pub fn eval_js_file(filename: LispStringRef) -> LispObject {
    let string = std::fs::read_to_string(filename.to_utf8()).unwrap();
    println!("{}", string);
    js_eval(string)
}

fn js_eval(string: String) -> LispObject {
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

    {
        let prelim_code = v8::String::new(
            scope,
            "var lisp = new Proxy({}, {
get: function(o, k) {
   return function() {
       const modargs = [k.replaceAll('-', '_')];
       for (let i = 0; i < arguments.length; ++i) {
           modargs.push(JSON.stringify(arguments[i]));
       }
       return JSON.parse(lisp_invoke.apply(this, modargs));
   }

}});
",
        )
        .unwrap();
        let prelim_script = v8::Script::compile(scope, prelim_code, None).unwrap();
        prelim_script.run(scope).unwrap();
    }

    // Create a string containing the JavaScript source code.
    let code = v8::String::new(scope, &string).unwrap();

    // Compile the source code.
    let script = v8::Script::compile(scope, code, None).unwrap();
    // Run the script to get the result.
    let result = script.run(scope).unwrap();
    let json_result = v8::json::stringify(scope, result).unwrap();

    // @TODO if result is undefined, don't stringify it, instead return
    // something that will ser into nil.

    // Convert the result to a string and print it.
    let result_string = json_result.to_rust_string_lossy(scope);
    crate::parsing::deser(&result_string, false).unwrap()
}

include!(concat!(env!("OUT_DIR"), "/javascript_exports.rs"));
