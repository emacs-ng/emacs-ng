use crate::parsing::{ArrayType, ObjectType};
use lazy_static::lazy_static;
use lisp::lisp::LispObject;
use lisp::lists::{LispCons, LispConsCircularChecks, LispConsEndChecks};
use lisp::multibyte::LispStringRef;
use lisp::remacs_sys::Ffuncall;
use lisp_macros::lisp_fn;
use rusty_v8 as v8;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::ffi::CString;
use std::io::Result;
use std::sync::Arc;
use std::sync::Mutex;

struct EmacsJsRuntime {
    r: Option<tokio::runtime::Runtime>,
    w: Option<deno_runtime::worker::MainWorker>,
}

static mut MAIN_RUNTIME: EmacsJsRuntime = EmacsJsRuntime { r: None, w: None };
static mut WITHIN_RUNTIME: bool = false;

impl EmacsJsRuntime {
    fn runtime() -> &'static mut EmacsJsRuntime {
        unsafe { &mut MAIN_RUNTIME }
    }

    fn handle() -> tokio::runtime::Handle {
        let runtime = Self::runtime();
        let roption: &Option<tokio::runtime::Runtime> = &runtime.r;
        roption.as_ref().unwrap().handle().clone()
    }

    fn worker() -> &'static mut deno_runtime::worker::MainWorker {
        let runtime = Self::runtime();
        runtime.w.as_mut().unwrap()
    }

    fn destroy_worker() {
        let runtime = Self::runtime();
        if runtime.w.is_some() {
            runtime.w.take();
        }
    }

    fn set_worker(w: deno_runtime::worker::MainWorker) {
        Self::runtime().w = Some(w);
    }

    fn main_worker_active() -> bool {
        Self::runtime().w.is_some()
    }

    fn set_runtime(r: tokio::runtime::Runtime) {
        Self::runtime().r = Some(r);
    }
}

// (DDS) This exists for breaking Deno's cache busting
// across invokations of emacs-ng, along with a hack
// to allow modules to be evaluated multiple times.
// By default, deno cache's modules results in a cache
// dir, ($HOME/.cache/deno/...) by default. If we didn't append
// time, it would re-evaluate $$import#1$$ over and over,
// and sometimes those cached files are different, which can
// lead to bad states.
// Sub-modules can be cached, which in general is fine, we just
// want our users high level code to be re-executed.
static mut COUNTER: u64 = 0;
macro_rules! unique_module {
    ($format: expr) => {{
        unsafe {
            COUNTER += 1;
            let time = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            format!($format, COUNTER, time)
        }
    }};
}

macro_rules! unique_module_import {
    ($filename: expr) => {{
        unsafe {
            COUNTER += 1;
            let time = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            format!("import '{}#{}{}';", $filename, COUNTER, time)
        }
    }};
}

lazy_static! {
    static ref FILE_CACHE: Mutex<HashMap<String, String>> = {
        {
            Mutex::new(HashMap::new())
        }
    };
}

// Aligned with code in prelim.js
const HASHTABLE: u32 = 0;
const ALIST: u32 = 1;
const PLIST: u32 = 2;
const ARRAY: u32 = 3;
const LIST: u32 = 4;

macro_rules! make_proxy {
    ($scope:expr, $lisp:expr) => {{
        let obj = if let Some(template) = unsafe { g.clone() } {
            let tpl = template.get($scope);
            let obj = tpl.new_instance($scope).unwrap();
            let value = v8::String::new($scope, &$lisp.to_C_unsigned().to_string()).unwrap();
            let inserted =
                obj.set_internal_field(0, v8::Local::<v8::Value>::try_from(value).unwrap());
            assert!(inserted);
            obj
        } else {
            panic!("Proxy Template None, unable  to build proxies");
        };

        unsafe {
            if lisp::remacs_sys::globals.Vjs_retain_map == lisp::remacs_sys::Qnil {
                lisp::remacs_sys::globals.Vjs_retain_map =
                    LispObject::cons($lisp, lisp::remacs_sys::Qnil);
                lisp::remacs_sys::staticpro(&lisp::remacs_sys::globals.Vjs_retain_map);
            } else {
                lisp::remacs_sys::globals.Vjs_retain_map =
                    LispObject::cons($lisp, lisp::remacs_sys::globals.Vjs_retain_map);
            }
        }

        obj
    }};
}

pub fn json_lisp(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let message = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);
    let option = args
        .get(1)
        .to_uint32(scope)
        .unwrap()
        .uint32_value(scope)
        .unwrap();

    let mut base_config = crate::parsing::gen_ser_deser_config();

    match option {
        HASHTABLE => base_config.obj = ObjectType::Hashtable,
        ALIST => base_config.obj = ObjectType::Alist,
        PLIST => base_config.obj = ObjectType::Plist,
        ARRAY => base_config.arr = ArrayType::Array,
        LIST => base_config.arr = ArrayType::List,
        _ => { /* noop */ }
    }

    if let Ok(result) = crate::parsing::deser(&message, Some(base_config)) {
        let proxy = make_proxy!(scope, result);
        let r = v8::Local::<v8::Value>::try_from(proxy).unwrap();
        retval.set(r);
    }
}

pub fn lisp_make_finalizer(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let len = args
        .get(0)
        .to_uint32(scope)
        .unwrap()
        .uint32_value(scope)
        .unwrap();

    let result = unsafe {
        let mut bound = vec![
            lisp::remacs_sys::Qjs__clear,
            lisp::remacs_sys::make_fixnum(len.into()),
        ];
        let list = lisp::remacs_sys::Flist(bound.len().try_into().unwrap(), bound.as_mut_ptr());
        let mut lambda = vec![lisp::remacs_sys::Qlambda, lisp::remacs_sys::Qnil, list];
        let lambda_list =
            lisp::remacs_sys::Flist(lambda.len().try_into().unwrap(), lambda.as_mut_ptr());
        lisp::remacs_sys::Fmake_finalizer(lambda_list)
    };

    let proxy = make_proxy!(scope, result);
    let r = v8::Local::<v8::Value>::try_from(proxy).unwrap();
    retval.set(r);
}

pub fn lisp_make_lambda(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let len = args
        .get(0)
        .to_uint32(scope)
        .unwrap()
        .uint32_value(scope)
        .unwrap();

    let num_args = args
        .get(1)
        .to_uint32(scope)
        .unwrap()
        .uint32_value(scope)
        .unwrap();

    let llen = unsafe { lisp::remacs_sys::make_fixnum(len.into()) };

    let finalizer = unsafe {
        let mut bound = vec![lisp::remacs_sys::Qjs__clear, llen];
        let list = lisp::remacs_sys::Flist(bound.len().try_into().unwrap(), bound.as_mut_ptr());
        let mut fargs = vec![lisp::remacs_sys::Qand_rest, lisp::remacs_sys::Qalpha];
        let fargs_list =
            lisp::remacs_sys::Flist(fargs.len().try_into().unwrap(), fargs.as_mut_ptr());

        let mut lambda = vec![lisp::remacs_sys::Qlambda, fargs_list, list];
        let lambda_list =
            lisp::remacs_sys::Flist(lambda.len().try_into().unwrap(), lambda.as_mut_ptr());
        lisp::remacs_sys::Fmake_finalizer(lambda_list)
    };

    let mut inner = vec![lisp::remacs_sys::Qjs__reenter, llen, finalizer];
    if num_args > 0 {
        inner.push(lisp::remacs_sys::Qalpha);
    }

    let result =
        unsafe { lisp::remacs_sys::Flist(inner.len().try_into().unwrap(), inner.as_mut_ptr()) };

    let proxy = make_proxy!(scope, result);
    let r = v8::Local::<v8::Value>::try_from(proxy).unwrap();
    retval.set(r);
}

pub fn lisp_string(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let message = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    let len = message.len();
    let cstr = CString::new(message).expect("Failed to allocate CString");
    let result =
        unsafe { lisp::remacs_sys::make_string_from_utf8(cstr.as_ptr(), len.try_into().unwrap()) };
    let proxy = make_proxy!(scope, result);
    let r = v8::Local::<v8::Value>::try_from(proxy).unwrap();
    retval.set(r);
}

pub fn lisp_fixnum(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let message = args
        .get(0)
        .to_number(scope)
        .unwrap()
        .integer_value(scope)
        .unwrap();

    let result = unsafe { lisp::remacs_sys::make_fixnum(message) };
    let proxy = make_proxy!(scope, result);
    let r = v8::Local::<v8::Value>::try_from(proxy).unwrap();
    retval.set(r);
}

pub fn lisp_float(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let message = args
        .get(0)
        .to_number(scope)
        .unwrap()
        .number_value(scope)
        .unwrap();

    let result = unsafe { lisp::remacs_sys::make_float(message) };
    let proxy = make_proxy!(scope, result);
    let r = v8::Local::<v8::Value>::try_from(proxy).unwrap();
    retval.set(r);
}

pub fn lisp_intern(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let a = args.get(0).to_object(scope).unwrap();
    assert!(a.internal_field_count() > 0);
    let internal = a.get_internal_field(scope, 0).unwrap();
    let ptrstr = internal
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);
    let lispobj =
        LispObject::from_C_unsigned(ptrstr.parse::<lisp::remacs_sys::EmacsUint>().unwrap());

    let result = unsafe { lisp::remacs_sys::Fintern(lispobj, lisp::remacs_sys::Qnil) };

    let proxy = make_proxy!(scope, result);
    let r = v8::Local::<v8::Value>::try_from(proxy).unwrap();
    retval.set(r);
}

pub fn lisp_list(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let len = args.length();
    let mut lisp_args = vec![];
    for i in 0..len {
        let arg = args.get(i);

        if arg.is_string() {
            let a = arg.to_string(scope).unwrap().to_rust_string_lossy(scope);

            if let Ok(deser) = crate::parsing::deser(&a, None) {
                lisp_args.push(deser);
            }
        } else if arg.is_object() {
            let a = arg.to_object(scope).unwrap();
            assert!(a.internal_field_count() > 0);
            let internal = a.get_internal_field(scope, 0).unwrap();
            let ptrstr = internal
                .to_string(scope)
                .unwrap()
                .to_rust_string_lossy(scope);
            let lispobj =
                LispObject::from_C_unsigned(ptrstr.parse::<lisp::remacs_sys::EmacsUint>().unwrap());
            lisp_args.push(lispobj);
        } else {
            let error = v8::String::new(scope, "Invalid arguments passed to lisp_invoke. Valid options are String, Function, or Proxy Object").unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
            // We do not want to execute any additional JS operations now
            // that we have thrown an exception. Instead we return.
            return;
        }
    }

    let result = unsafe {
        lisp::remacs_sys::Flist(lisp_args.len().try_into().unwrap(), lisp_args.as_mut_ptr())
    };
    let proxy = make_proxy!(scope, result);
    let r = v8::Local::<v8::Value>::try_from(proxy).unwrap();
    retval.set(r);
}

pub fn lisp_json(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let mut parsed = false;
    if args.get(0).is_object() {
        let a = args.get(0).to_object(scope).unwrap();
        assert!(a.internal_field_count() > 0);
        let internal = a.get_internal_field(scope, 0).unwrap();
        let ptrstr = internal
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope);
        let lispobj =
            LispObject::from_C_unsigned(ptrstr.parse::<lisp::remacs_sys::EmacsUint>().unwrap());

        if let Ok(json) = crate::parsing::ser(lispobj) {
            parsed = true;
            let r =
                v8::Local::<v8::Value>::try_from(v8::String::new(scope, &json).unwrap()).unwrap();
            retval.set(r);
        }
    }

    if !parsed {
        let r = v8::Local::<v8::Value>::try_from(
            v8::String::new(scope, "{\"nativeProxy\": true}").unwrap(),
        )
        .unwrap();
        retval.set(r);
    }
}

pub fn finalize(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _retval: v8::ReturnValue,
) {
    let len = args.length();
    let mut new_list = LispObject::cons(lisp::remacs_sys::Qnil, lisp::remacs_sys::Qnil);
    for i in 0..len {
        let arg = args.get(i);
        if arg.is_object() {
            let a = arg.to_object(scope).unwrap();
            assert!(a.internal_field_count() > 0);
            let internal = a.get_internal_field(scope, 0).unwrap();
            let ptrstr = internal
                .to_string(scope)
                .unwrap()
                .to_rust_string_lossy(scope);
            let lispobj =
                LispObject::from_C_unsigned(ptrstr.parse::<lisp::remacs_sys::EmacsUint>().unwrap());
            new_list = LispObject::cons(lispobj, new_list);
        }
    }

    unsafe { lisp::remacs_sys::globals.Vjs_retain_map = new_list };
}

pub fn is_proxy(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let mut is_proxy = false;
    if args.get(0).is_object() {
        let arg = args.get(0).to_object(scope).unwrap();
        if arg.internal_field_count() > 0 {
            is_proxy = true;
        }
    }
    let boolean = v8::Boolean::new(scope, is_proxy);
    let r = v8::Local::<v8::Value>::try_from(boolean).unwrap();
    retval.set(r);
}

unsafe extern "C" fn lisp_springboard(arg1: *mut ::libc::c_void) -> LispObject {
    let mut lisp_args: Vec<LispObject> = *Box::from_raw(arg1 as *mut Vec<LispObject>);
    Ffuncall(lisp_args.len().try_into().unwrap(), lisp_args.as_mut_ptr())
}

unsafe extern "C" fn lisp_handler(
    _arg1: lisp::remacs_sys::nonlocal_exit::Type,
    arg2: LispObject,
) -> LispObject {
    LispObject::cons(lisp::remacs_sys::Qjs_lisp_error, arg2)
}

static mut raw_handle: *mut v8::HandleScope = std::ptr::null_mut();
pub fn lisp_callback(
    mut scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let mut lisp_args = vec![];
    let len = args.length();

    let a = args.get(0).to_object(scope).unwrap();
    assert!(a.internal_field_count() > 0);
    let internal = a.get_internal_field(scope, 0).unwrap();
    let ptrstr = internal
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);
    let lispobj =
        LispObject::from_C_unsigned(ptrstr.parse::<lisp::remacs_sys::EmacsUint>().unwrap());
    lisp_args.push(lispobj);

    for i in 1..len {
        let arg = args.get(i);

        if arg.is_string() {
            let a = arg.to_string(scope).unwrap().to_rust_string_lossy(scope);

            if let Ok(deser) = crate::parsing::deser(&a, None) {
                lisp_args.push(deser);
            }
        } else if arg.is_object() {
            let a = arg.to_object(scope).unwrap();
            assert!(a.internal_field_count() > 0);
            let internal = a.get_internal_field(scope, 0).unwrap();
            let ptrstr = internal
                .to_string(scope)
                .unwrap()
                .to_rust_string_lossy(scope);
            let lispobj =
                LispObject::from_C_unsigned(ptrstr.parse::<lisp::remacs_sys::EmacsUint>().unwrap());
            lisp_args.push(lispobj);
        } else {
            let error = v8::String::new(scope, "Invalid arguments passed to lisp_invoke. Valid options are String, Function, or Proxy Object").unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
            // We do not want to execute any additional JS operations now
            // that we have thrown an exception. Instead we return.
            return;
        }
    }

    // 'Why are you using mem transmute to store this scope globally?'
    // If you attempt to re-enter JS from a lisp function
    // i.e. js -> lisp -> js, if you were to use "worker.execute",
    // Deno will attempt to create a new handlescope using the
    // thread isolate and global context. Due to how Deno
    // constructed the HandleScope class, attempting to create
    // a handle scope from an isolate when there is already a handle
    // on the stack will cause the process to panic.
    // The 'right way' to do  this is to use the handle that is currently
    // on the stack, which is scope. Since JS and Lisp are on the same thread,
    // scope will be alive in the situation we go from js -> lisp -> js. This
    // is reflected in the stateful variable WITHIN_RUNTIME. If you are
    // not within the runtime, you can run worker.execute. If you are within
    // the runtime, you mem::transmute this scope and use that.
    // Think of it like passing down scope through every function call until we
    // need it, but in a round about way.
    // >>> Code touching 'raw_handle' and 'WITHIN_RUNTIME' needs to be
    // >>> managed very carefully. It should not be touched without a good reason.
    unsafe { raw_handle = std::mem::transmute(scope) };
    let boxed = Box::new(lisp_args);
    let raw_ptr = Box::into_raw(boxed);
    let results = unsafe {
        lisp::remacs_sys::internal_catch_all(
            Some(lisp_springboard),
            raw_ptr as *mut ::libc::c_void,
            Some(lisp_handler),
        )
    };

    unsafe {
        scope = std::mem::transmute(raw_handle);
        raw_handle = std::ptr::null_mut();
    }

    if results.is_cons() {
        let cons: LispCons = results.into();
        if cons.car() == lisp::remacs_sys::Qjs_lisp_error {
            // Lisp has thrown, so we want to throw a JS exception.
            let lisp_error_string = unsafe { lisp::remacs_sys::Ferror_message_string(cons.cdr()) };
            let lisp_ref: LispStringRef = lisp_error_string.into();
            let err = lisp_ref.to_utf8();
            let error = v8::String::new(scope, &err).unwrap();
            let exception = v8::Exception::error(scope, error);
            scope.throw_exception(exception);
            // We do not want to execute any additional JS operations now
            // that we have thrown an exception. Instead we return.
            return;
        }
    }

    // LOGIC, attempt to se, with a version of se that returns an error,
    // if this can't se, it is a proxy, and we will treat it as such.
    let is_primative = unsafe {
        lisp::remacs_sys::STRINGP(results)
            || lisp::remacs_sys::FIXNUMP(results)
            || lisp::remacs_sys::FLOATP(results)
            || results == lisp::remacs_sys::Qnil
            || results == lisp::remacs_sys::Qt
    };
    if is_primative {
        if let Ok(json) = crate::parsing::ser(results) {
            let r =
                v8::Local::<v8::Value>::try_from(v8::String::new(scope, &json).unwrap()).unwrap();
            retval.set(r);
        }
    } else {
        let obj = make_proxy!(scope, results);
        let r = v8::Local::<v8::Value>::try_from(obj).unwrap();
        retval.set(r);
    }
}

const DEFAULT_ADDR: &str = "127.0.0.1:9229";

struct EmacsJsOptions {
    tick_rate: f64,
    ops: Option<deno_runtime::permissions::Permissions>,
    error_handler: LispObject,
    inspect: Option<String>,
    inspect_brk: Option<String>,
}

static mut OPTS: EmacsJsOptions = EmacsJsOptions {
    tick_rate: 0.1,
    ops: None,
    error_handler: lisp::remacs_sys::Qnil,
    inspect: None,
    inspect_brk: None,
};

fn set_default_opts_if_unset() {
    unsafe {
        if OPTS.ops.is_none() {
            OPTS.ops = Some(deno_runtime::permissions::Permissions::allow_all());
        }
    }
}

const JS_PERMS_ERROR: &str =
    "Valid options are: :allow-net nil :allow-read nil :allow-write nil :allow-run nil";
fn permissions_from_args(args: &[LispObject]) -> EmacsJsOptions {
    let mut permissions = deno_runtime::permissions::Permissions::allow_all();
    let mut tick_rate = 0.1;
    let mut error_handler = lisp::remacs_sys::Qnil;
    let mut inspect = None;
    let mut inspect_brk = None;

    if args.len() % 2 != 0 {
        error!(JS_PERMS_ERROR);
    }

    for i in 0..args.len() {
        if i % 2 != 0 {
            continue;
        }

        let key = args[i];
        let value = args[i + 1];

        match key {
            lisp::remacs_sys::QCallow_net => {
                if value == lisp::remacs_sys::Qnil {
                    permissions.net.global_state =
                        deno_runtime::permissions::PermissionState::Denied;
                }
            }
            lisp::remacs_sys::QCallow_read => {
                if value == lisp::remacs_sys::Qnil {
                    permissions.read.global_state =
                        deno_runtime::permissions::PermissionState::Denied;
                }
            }
            lisp::remacs_sys::QCallow_write => {
                if value == lisp::remacs_sys::Qnil {
                    permissions.write.global_state =
                        deno_runtime::permissions::PermissionState::Denied;
                }
            }
            lisp::remacs_sys::QCallow_run => {
                if value == lisp::remacs_sys::Qnil {
                    permissions.run = deno_runtime::permissions::PermissionState::Denied;
                }
            }
            lisp::remacs_sys::QCjs_tick_rate => unsafe {
                if lisp::remacs_sys::FLOATP(value) {
                    tick_rate = lisp::remacs_sys::XFLOAT_DATA(value);
                }
            },
            lisp::remacs_sys::QCjs_error_handler => {
                error_handler = value;
            }
            lisp::remacs_sys::QCinspect => {
                if value.is_string() {
                    inspect = Some(value.as_string().unwrap().to_utf8());
                } else if value == lisp::remacs_sys::Qt {
                    inspect = Some(DEFAULT_ADDR.to_string());
                }
            }
            lisp::remacs_sys::QCinspect_brk => {
                if value.is_string() {
                    inspect_brk = Some(value.as_string().unwrap().to_utf8());
                } else if value == lisp::remacs_sys::Qt {
                    inspect_brk = Some(DEFAULT_ADDR.to_string());
                }
            }

            _ => error!(JS_PERMS_ERROR),
        }
    }

    EmacsJsOptions {
        tick_rate,
        ops: Some(permissions),
        error_handler,
        inspect,
        inspect_brk,
    }
}

// I'm keeping the logic simple  for now,
// if it doesn't end with .js, we will
// treat it as ts
fn is_typescript(s: &str) -> bool {
    !s.ends_with("js")
}

#[lisp_fn(min = "1")]
pub fn eval_js(args: &[LispObject]) -> LispObject {
    let string_obj: LispStringRef = args[0].into();
    let ops = unsafe { &OPTS };
    let name = unique_module!("./$anon$lisp${}{}.ts");
    let string = string_obj.to_utf8();
    let is_typescript = args.len() == 3
        && args[1] == lisp::remacs_sys::QCtypescript
        && args[2] == lisp::remacs_sys::Qt;
    let result = run_module(&name, Some(string), ops, is_typescript).unwrap_or_else(move |e| {
        // See comment in eval-js-file for why we call destroy_worker
        EmacsJsRuntime::destroy_worker();
        handle_error(e, ops.error_handler)
    });
    result
}

#[lisp_fn(min = "1")]
pub fn eval_js_file(args: &[LispObject]) -> LispObject {
    let filename: LispStringRef = args[0].into();
    let ops = unsafe { &OPTS };
    let mut module = filename.to_utf8();
    let is_typescript = (args.len() == 3
        && args[1] == lisp::remacs_sys::QCtypescript
        && args[2] == lisp::remacs_sys::Qt)
        || is_typescript(&module);

    // This is a hack to allow for our behavior of
    // executing a module multiple times.
    // @TODO (DDS) we should revisit if we actually want to
    // do this.
    let import = unique_module_import!(module);
    module = unique_module!("./$import${}{}.ts");

    let result = run_module(&module, Some(import), ops, is_typescript).unwrap_or_else(move |e| {
        // If a toplevel module rejects in the Deno
        // framework, it will .unwrap() a bad result
        // in the next call to poll(). This is due to
        // assumptions that deno is making about module
        // re-entry after a failure. Even though
        // v8 can handle reusing an isolate after a
        // module fails to load, Deno cannot at this time.
        // So instead, we destroy the main worker,
        // and re-create the isolate. The user impact
        // should be minimal since their module never
        // loaded anyway.
        EmacsJsRuntime::destroy_worker();
        handle_error(e, ops.error_handler)
    });
    result
}

fn get_buffer_contents(mut buffer: LispObject) -> LispObject {
    if buffer.is_nil() {
        buffer = unsafe { lisp::remacs_sys::Fcurrent_buffer() };
    }

    unsafe {
        let current = lisp::remacs_sys::Fcurrent_buffer();
        lisp::remacs_sys::Fset_buffer(buffer);
        let lstring = lisp::remacs_sys::Fbuffer_string();
        lisp::remacs_sys::Fset_buffer(current);
        lstring
    }
}

#[lisp_fn(min = "0", intspec = "")]
pub fn eval_js_buffer(buffer: LispObject) -> LispObject {
    let lisp_string = get_buffer_contents(buffer);
    eval_js(&[lisp_string])
}

#[lisp_fn(min = "0", intspec = "")]
pub fn eval_ts_buffer(buffer: LispObject) -> LispObject {
    let lisp_string = get_buffer_contents(buffer);
    eval_js(&[
        lisp_string,
        lisp::remacs_sys::QCtypescript,
        lisp::remacs_sys::Qt,
    ])
}

fn get_region(start: LispObject, end: LispObject) -> LispObject {
    let saved = unsafe { lisp::remacs_sys::save_restriction_save() };
    unsafe {
        lisp::remacs_sys::Fnarrow_to_region(start, end);
        let lstring = lisp::remacs_sys::Fbuffer_string();
        lisp::remacs_sys::save_restriction_restore(saved);
        lstring
    }
}

#[lisp_fn(intspec = "r")]
pub fn eval_js_region(start: LispObject, end: LispObject) -> LispObject {
    let lisp_string = get_region(start, end);
    eval_js(&[lisp_string])
}

#[lisp_fn(intspec = "r")]
pub fn eval_ts_region(start: LispObject, end: LispObject) -> LispObject {
    let lisp_string = get_region(start, end);
    eval_js(&[
        lisp_string,
        lisp::remacs_sys::QCtypescript,
        lisp::remacs_sys::Qt,
    ])
}

#[lisp_fn]
pub fn js_initialize(args: &[LispObject]) -> LispObject {
    let ops = permissions_from_args(args);
    unsafe {
        OPTS = ops;
    }
    lisp::remacs_sys::Qnil
}

fn js_reenter_inner(scope: &mut v8::HandleScope, args: &[LispObject]) -> LispObject {
    let index = args[0];

    if !unsafe { lisp::remacs_sys::INTEGERP(index) } {
        error!("Failed to provide proper index to js--reenter");
    }

    let value = unsafe {
        lisp::remacs_sys::check_integer_range(
            index,
            lisp::remacs_sys::intmax_t::MIN,
            lisp::remacs_sys::intmax_t::MAX,
        )
    };

    let context = scope.get_current_context();
    let global = context.global(scope);

    let name = v8::String::new(scope, "__invoke").unwrap();
    let fnc: v8::Local<v8::Function> = global.get(scope, name.into()).unwrap().try_into().unwrap();

    let recv =
        v8::Local::<v8::Value>::try_from(v8::String::new(scope, "lisp_invoke").unwrap()).unwrap();
    let arg0 = v8::Local::<v8::Value>::try_from(v8::Number::new(scope, value as f64)).unwrap();
    let mut v8_args = vec![arg0];

    if args.len() > 2 {
        let cons: LispCons = args[2].into();
        cons.iter_cars(LispConsEndChecks::on, LispConsCircularChecks::on)
            .for_each(|a| {
                let is_primative = unsafe {
                    lisp::remacs_sys::STRINGP(a)
                        || lisp::remacs_sys::FIXNUMP(a)
                        || lisp::remacs_sys::FLOATP(a)
                        || a == lisp::remacs_sys::Qnil
                        || a == lisp::remacs_sys::Qt
                };
                if is_primative {
                    if let Ok(json) = crate::parsing::ser(a) {
                        v8_args.push(
                            v8::Local::<v8::Value>::try_from(
                                v8::String::new(scope, &json).unwrap(),
                            )
                            .unwrap(),
                        );
                    }
                } else {
                    let obj = make_proxy!(scope, a);
                    v8_args.push(v8::Local::<v8::Value>::try_from(obj).unwrap());
                }
            });
    }

    let mut retval = lisp::remacs_sys::Qnil;
    if let Some(result) = fnc.call(scope, recv, v8_args.as_slice()) {
        if result.is_string() {
            let a = result.to_string(scope).unwrap().to_rust_string_lossy(scope);

            if let Ok(deser) = crate::parsing::deser(&a, None) {
                retval = deser;
            }
        } else if result.is_object() {
            let a = result.to_object(scope).unwrap();
            assert!(a.internal_field_count() > 0);
            let internal = a.get_internal_field(scope, 0).unwrap();
            let ptrstr = internal
                .to_string(scope)
                .unwrap()
                .to_rust_string_lossy(scope);
            let lispobj =
                LispObject::from_C_unsigned(ptrstr.parse::<lisp::remacs_sys::EmacsUint>().unwrap());
            retval = lispobj;
        }
    }

    retval
}

#[lisp_fn(min = "1")]
pub fn js__reenter(args: &[LispObject]) -> LispObject {
    let retval;
    if !unsafe { WITHIN_RUNTIME } {
        let worker = EmacsJsRuntime::worker();
        let runtime = &mut worker.js_runtime;
        let context = runtime.global_context();
        let scope = &mut v8::HandleScope::with_context(runtime.v8_isolate(), context);
        let stacked = unsafe { raw_handle }; // Should be std::ptr::null_mut()
        unsafe {
            raw_handle = scope as *mut v8::HandleScope;
        };
        unsafe { WITHIN_RUNTIME = true };
        retval = js_reenter_inner(scope, args);
        unsafe { WITHIN_RUNTIME = false };
        unsafe { raw_handle = stacked };
    } else {
        let scope: &mut v8::HandleScope = unsafe { std::mem::transmute(raw_handle) };
        retval = js_reenter_inner(scope, args);
        unsafe { raw_handle = std::mem::transmute(scope) };
    }

    retval
}

fn js_clear_internal(scope: &mut v8::HandleScope, idx: LispObject) {
    let value = unsafe {
        lisp::remacs_sys::check_integer_range(
            idx,
            lisp::remacs_sys::intmax_t::MIN,
            lisp::remacs_sys::intmax_t::MAX,
        )
    };

    let context = scope.get_current_context();
    let global = context.global(scope);

    let name = v8::String::new(scope, "__clear").unwrap();
    let fnc: v8::Local<v8::Function> = global.get(scope, name.into()).unwrap().try_into().unwrap();
    let recv =
        v8::Local::<v8::Value>::try_from(v8::String::new(scope, "lisp_invoke").unwrap()).unwrap();
    let arg0 = v8::Local::<v8::Value>::try_from(v8::Number::new(scope, value as f64)).unwrap();
    let v8_args = vec![arg0];
    fnc.call(scope, recv, v8_args.as_slice()).unwrap();
}

#[lisp_fn]
pub fn js__clear(idx: LispObject) -> LispObject {
    if !unsafe { WITHIN_RUNTIME } {
        let worker = EmacsJsRuntime::worker();
        let runtime = &mut worker.js_runtime;
        let context = runtime.global_context();
        let scope = &mut v8::HandleScope::with_context(runtime.v8_isolate(), context);
        js_clear_internal(scope, idx);
    } else {
        let scope: &mut v8::HandleScope = unsafe { std::mem::transmute(raw_handle) };
        js_clear_internal(scope, idx);
        unsafe { raw_handle = std::mem::transmute(scope) };
    }

    lisp::remacs_sys::Qnil
}

fn into_ioerr<E: Into<Box<dyn std::error::Error + Send + Sync>>>(e: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e)
}

fn execute<T: Sized + std::future::Future<Output = Result<()>>>(fnc: T) -> Result<()> {
    if unsafe { WITHIN_RUNTIME } {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Attempted to execute javascript from lisp within the javascript context. Javascript is not re-entrant, cannot call JS -> Lisp -> JS"))
    } else {
        let handle = EmacsJsRuntime::handle();
        unsafe { WITHIN_RUNTIME = true };
        let result = handle.block_on(fnc);
        unsafe { WITHIN_RUNTIME = false };
        result
    }
}

fn tick_js() -> Result<()> {
    execute(async move {
        futures::future::poll_fn(|cx| {
            let w = EmacsJsRuntime::worker();
            let polled = w.poll_event_loop(cx);
            match polled {
                std::task::Poll::Ready(r) => r.map_err(|e| into_ioerr(e))?,
                std::task::Poll::Pending => {}
            }

            std::task::Poll::Ready(Ok(()))
        })
        .await
    })
}
static mut FAILED_TO_INIT: bool = false;
static ONCE: std::sync::Once = std::sync::Once::new();
fn init_once(js_options: &EmacsJsOptions) -> Result<()> {
    if unsafe { FAILED_TO_INIT } {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Javascript environment failed to initalize, cannot run JS code",
        ));
    }

    let mut once_result: Result<()> = Ok(());
    ONCE.call_once(|| {
        let indirect = || -> Result<()> {
            let runtime = tokio::runtime::Builder::new()
                .threaded_scheduler()
                .enable_io()
                .enable_time()
                .max_threads(32)
                .build()?;

            EmacsJsRuntime::set_runtime(runtime);

            //(run-with-timer t 1 'js-tick-event-loop error-handler)
            let cstr = CString::new("run-with-timer").expect("Failed to create timer");
            let callback = CString::new("js-tick-event-loop").expect("Failed to create timer");
            unsafe {
                let fun = lisp::remacs_sys::intern_c_string(cstr.as_ptr());
                let fun_callback = lisp::remacs_sys::intern_c_string(callback.as_ptr());
                let mut args = vec![
                    fun,
                    lisp::remacs_sys::Qt,
                    lisp::remacs_sys::make_float(js_options.tick_rate),
                    fun_callback,
                    js_options.error_handler,
                ];
                lisp::remacs_sys::Ffuncall(args.len().try_into().unwrap(), args.as_mut_ptr());
            }

            Ok(())
        };
        once_result = indirect();
    });

    if once_result.is_err() {
        unsafe { FAILED_TO_INIT = true };
    }

    once_result
}

static mut g: Option<v8::Global<v8::ObjectTemplate>> = None;
static mut program_state: Option<Arc<deno::program_state::ProgramState>> = None;
fn init_worker(filepath: &str, js_options: &EmacsJsOptions) -> Result<()> {
    if EmacsJsRuntime::main_worker_active() {
        return Ok(());
    }

    let runtime = EmacsJsRuntime::handle();
    let main_module =
        deno_core::ModuleSpecifier::resolve_url_or_path(filepath).map_err(|e| into_ioerr(e))?;
    set_default_opts_if_unset();
    let permissions = js_options.ops.as_ref().unwrap().clone();
    let inspect = if let Some(i) = &js_options.inspect {
        Some(
            i.parse::<std::net::SocketAddr>()
                .map_err(|e| into_ioerr(e))?,
        )
    } else {
        None
    };

    let inspect_brk = if let Some(i) = &js_options.inspect_brk {
        Some(
            i.parse::<std::net::SocketAddr>()
                .map_err(|e| into_ioerr(e))?,
        )
    } else {
        None
    };

    let flags = deno::flags::Flags {
        unstable: true,
        inspect,
        inspect_brk,
        ..Default::default()
    };

    let program = unsafe {
        let p = deno::program_state::ProgramState::new(flags).map_err(|e| into_ioerr(e))?;
        program_state = Some(p);
        program_state.as_ref().clone().unwrap()
    };
    let mut worker = deno::create_main_worker(&program, main_module.clone(), permissions);
    let result: Result<deno_runtime::worker::MainWorker> = runtime.block_on(async move {
        let runtime = &mut worker.js_runtime;
        {
            let context = runtime.global_context();
            let scope = &mut v8::HandleScope::with_context(runtime.v8_isolate(), context);
            let context = scope.get_current_context();
            let global = context.global(scope);
            {
                let name = v8::String::new(scope, "proxyProto").unwrap();
                let template = v8::ObjectTemplate::new(scope);
                template.set_internal_field_count(1);
                let glob = v8::Global::new(scope, template);
                unsafe { g = Some(glob) };
                let obj = v8::Object::new(scope);
                global.set(scope, name.into(), obj.into());
            }
            {
                let name = v8::String::new(scope, "lisp_invoke").unwrap();
                let func = v8::Function::new(scope, lisp_callback).unwrap();
                global.set(scope, name.into(), func.into());
            }
            {
                let name = v8::String::new(scope, "is_proxy").unwrap();
                let func = v8::Function::new(scope, is_proxy).unwrap();
                global.set(scope, name.into(), func.into());
            }
            {
                let name = v8::String::new(scope, "finalize").unwrap();
                let func = v8::Function::new(scope, finalize).unwrap();
                global.set(scope, name.into(), func.into());
            }
            {
                let name = v8::String::new(scope, "lisp_json").unwrap();
                let func = v8::Function::new(scope, lisp_json).unwrap();
                global.set(scope, name.into(), func.into());
            }
            {
                let name = v8::String::new(scope, "lisp_intern").unwrap();
                let func = v8::Function::new(scope, lisp_intern).unwrap();
                global.set(scope, name.into(), func.into());
            }
            {
                let name = v8::String::new(scope, "lisp_make_finalizer").unwrap();
                let func = v8::Function::new(scope, lisp_make_finalizer).unwrap();
                global.set(scope, name.into(), func.into());
            }
            {
                let name = v8::String::new(scope, "lisp_string").unwrap();
                let func = v8::Function::new(scope, lisp_string).unwrap();
                global.set(scope, name.into(), func.into());
            }
            {
                let name = v8::String::new(scope, "lisp_fixnum").unwrap();
                let func = v8::Function::new(scope, lisp_fixnum).unwrap();
                global.set(scope, name.into(), func.into());
            }
            {
                let name = v8::String::new(scope, "lisp_float").unwrap();
                let func = v8::Function::new(scope, lisp_float).unwrap();
                global.set(scope, name.into(), func.into());
            }
            {
                let name = v8::String::new(scope, "lisp_make_lambda").unwrap();
                let func = v8::Function::new(scope, lisp_make_lambda).unwrap();
                global.set(scope, name.into(), func.into());
            }
            {
                let name = v8::String::new(scope, "lisp_list").unwrap();
                let func = v8::Function::new(scope, lisp_list).unwrap();
                global.set(scope, name.into(), func.into());
            }
            {
                let name = v8::String::new(scope, "json_lisp").unwrap();
                let func = v8::Function::new(scope, json_lisp).unwrap();
                global.set(scope, name.into(), func.into());
            }
        }
        {
            runtime
                .execute("prelim.js", include_str!("prelim.js"))
                .map_err(|e| into_ioerr(e))?
        }

        Ok(worker)
    });

    let worker = result?;
    EmacsJsRuntime::set_worker(worker);
    Ok(())
}

fn run_module(
    filepath: &str,
    additional_js: Option<String>,
    js_options: &EmacsJsOptions,
    as_typescript: bool,
) -> Result<LispObject> {
    init_once(js_options)?;
    init_worker(filepath, js_options)?;

    execute(async move {
        let w = EmacsJsRuntime::worker();
        let main_module =
            deno_core::ModuleSpecifier::resolve_url_or_path(filepath).map_err(|e| into_ioerr(e))?;

        let main_module_url = main_module.as_url().to_owned();
        if let Some(js) = additional_js {
            let program = unsafe { program_state.as_ref().clone().unwrap() };
            // We are inserting a fake file into the file cache in order to execute
            // our module.
            let file = deno::file_fetcher::File {
                local: main_module_url.to_file_path().unwrap(),
                maybe_types: None,
                media_type: if as_typescript {
                    deno::media_type::MediaType::TypeScript
                } else {
                    deno::media_type::MediaType::JavaScript
                },
                source: js,
                specifier: deno_core::ModuleSpecifier::from(main_module_url),
            };

            program.file_fetcher.insert_cached(file);
        }

        w.execute_module(&main_module)
            .await
            .map_err(|e| into_ioerr(e))?;
        Ok(())
    })?;

    Ok(lisp::remacs_sys::Qnil)
}

fn handle_error(e: std::io::Error, handler: LispObject) -> LispObject {
    let err_string = e.to_string();
    if handler.is_nil() {
        error!(err_string);
    } else {
        unsafe {
            let len = err_string.len();
            let cstr = CString::new(err_string).expect("Failed to allocate CString");
            let lstring =
                lisp::remacs_sys::make_string_from_utf8(cstr.as_ptr(), len.try_into().unwrap());
            let mut args = vec![handler, lstring];
            Ffuncall(args.len().try_into().unwrap(), args.as_mut_ptr())
        }
    }
}

fn tick() -> Result<()> {
    init_worker("tick.js", unsafe { &OPTS })?;
    tick_js()
}

#[lisp_fn]
pub fn js_tick_event_loop(handler: LispObject) -> LispObject {
    // If we are within the runtime, we don't want to attempt to
    // call execute, as we will error, and there really isn't anything
    // anyone can do about it. Just defer the event loop until
    // we are out of the runtime.
    if unsafe { WITHIN_RUNTIME } {
        return lisp::remacs_sys::Qnil;
    }

    tick()
        .map(|_| lisp::remacs_sys::Qnil)
        // We do NOT want to destroy the MainWorker if we error here.
        // We can still use this isolate for future promise resolutions
        // instead, just pass to the error handler.
        .unwrap_or_else(|e| handle_error(e, handler))
}

// @TODO we actually should call this, since it performs runtime actions.
// for now, we are manually calling 'staticpro'
#[allow(dead_code)]
fn init_syms() {
    defvar_lisp!(Vjs_retain_map, "js-retain-map", lisp::remacs_sys::Qnil);

    def_lisp_sym!(Qjs_lisp_error, "js-lisp-error");
    def_lisp_sym!(QCallow_net, ":allow-net");
    def_lisp_sym!(QCallow_read, ":allow-read");
    def_lisp_sym!(QCallow_write, ":allow-write");
    def_lisp_sym!(QCallow_run, ":allow-run");
    def_lisp_sym!(QCjs_tick_rate, ":js-tick-rate");
    def_lisp_sym!(Qjs_error, "js-error");
    def_lisp_sym!(QCjs_error_handler, ":js-error-handler");
    def_lisp_sym!(QCtypescript, ":typescript");

    def_lisp_sym!(Qjs__clear, "js--clear");
    def_lisp_sym!(Qlambda, "lambda");
    def_lisp_sym!(Qjs__reenter, "js--reenter");
    def_lisp_sym!(QCinspect, ":inspect");
    def_lisp_sym!(QCinspect_brk, ":inspect-brk");
}

include!(concat!(env!("OUT_DIR"), "/javascript_exports.rs"));
