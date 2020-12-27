use crate::lisp::LispObject;
use crate::lists::LispCons;
use crate::multibyte::LispStringRef;
use crate::remacs_sys::{intern_c_string, Ffuncall};
use remacs_macros::lisp_fn;
use rusty_v8 as v8;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::ffi::CString;
use std::io::Result;

struct EmacsJsRuntime {
    r: Option<tokio::runtime::Runtime>,
    w: Option<deno_runtime::worker::MainWorker>,
}

static mut MAIN_RUNTIME: std::mem::MaybeUninit<EmacsJsRuntime> =
    std::mem::MaybeUninit::<EmacsJsRuntime>::uninit();
static mut WITHIN_RUNTIME: bool = false;

impl EmacsJsRuntime {
    fn set_main(r: tokio::runtime::Runtime, w: deno_runtime::worker::MainWorker) {
        let main = EmacsJsRuntime {
            r: Some(r),
            w: Some(w),
        };
        unsafe { MAIN_RUNTIME.write(main) };
    }

    fn runtime() -> &'static mut EmacsJsRuntime {
        unsafe { &mut *MAIN_RUNTIME.as_mut_ptr() }
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
            LispObject::from_C_unsigned(ptrstr.parse::<crate::remacs_sys::EmacsUint>().unwrap());

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
    let mut new_list = LispObject::cons(crate::remacs_sys::Qnil, crate::remacs_sys::Qnil);
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
            let lispobj = LispObject::from_C_unsigned(
                ptrstr.parse::<crate::remacs_sys::EmacsUint>().unwrap(),
            );
            new_list = LispObject::cons(lispobj, new_list);
        }
    }

    unsafe { crate::remacs_sys::globals.Vjs_retain_map = new_list };
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
    _arg1: crate::remacs_sys::nonlocal_exit::Type,
    arg2: LispObject,
) -> LispObject {
    LispObject::cons(crate::remacs_sys::Qjs_lisp_error, arg2)
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
        let arg = args.get(i);

        if arg.is_string() {
            let a = arg.to_string(scope).unwrap().to_rust_string_lossy(scope);

            if let Ok(deser) = crate::parsing::deser(&a) {
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
            let lispobj = LispObject::from_C_unsigned(
                ptrstr.parse::<crate::remacs_sys::EmacsUint>().unwrap(),
            );
            lisp_args.push(lispobj);
        } else {
            panic!("Wrong Argument");
        }
    }

    let boxed = Box::new(lisp_args);
    let raw_ptr = Box::into_raw(boxed);
    let results = unsafe {
        crate::remacs_sys::internal_catch_all(
            Some(lisp_springboard),
            raw_ptr as *mut ::libc::c_void,
            Some(lisp_handler),
        )
    };

    if results.is_cons() {
        let cons: LispCons = results.into();
        if cons.car() == crate::remacs_sys::Qjs_lisp_error {
            // Lisp has thrown, so we want to throw a JS exception.
            let lisp_error_string = unsafe { crate::remacs_sys::Ferror_message_string(cons.cdr()) };
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
        crate::remacs_sys::STRINGP(results)
            || crate::remacs_sys::FIXNUMP(results)
            || crate::remacs_sys::FLOATP(results)
            || results == crate::remacs_sys::Qnil
            || results == crate::remacs_sys::Qt
    };
    if is_primative {
        if let Ok(json) = crate::parsing::ser(results) {
            let r =
                v8::Local::<v8::Value>::try_from(v8::String::new(scope, &json).unwrap()).unwrap();
            retval.set(r);
        }
    } else {
        let template = v8::ObjectTemplate::new(scope);
        template.set_internal_field_count(1);
        let obj = template.new_instance(scope).unwrap();
        let value = v8::String::new(scope, &results.to_C_unsigned().to_string()).unwrap();
        let inserted = obj.set_internal_field(0, v8::Local::<v8::Value>::try_from(value).unwrap());
        assert!(inserted);

        unsafe {
            if crate::remacs_sys::globals.Vjs_retain_map == crate::remacs_sys::Qnil {
                crate::remacs_sys::globals.Vjs_retain_map =
                    LispObject::cons(results, crate::remacs_sys::Qnil);
                crate::remacs_sys::staticpro(&crate::remacs_sys::globals.Vjs_retain_map);
            } else {
                crate::remacs_sys::globals.Vjs_retain_map =
                    LispObject::cons(results, crate::remacs_sys::globals.Vjs_retain_map);
            }
        }

        let r = v8::Local::<v8::Value>::try_from(obj).unwrap();
        retval.set(r);
    }
}

struct EmacsJsOptions {
    tick_rate: f64,
    ops: deno_runtime::permissions::PermissionsOptions,
    error_handler: LispObject,
}

const JS_PERMS_ERROR: &str =
    "Valid options are: :allow-net nil :allow-read nil :allow-write nil :allow-run nil";
fn permissions_from_args(args: &[LispObject]) -> EmacsJsOptions {
    let mut allow_net = true;
    let mut allow_read = true;
    let mut allow_write = true;
    let mut allow_run = true;
    let mut tick_rate = 0.1;
    let mut error_handler = crate::remacs_sys::Qnil;

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
            crate::remacs_sys::QCallow_net => {
                if value == crate::remacs_sys::Qnil {
                    allow_net = false
                }
            }
            crate::remacs_sys::QCallow_read => {
                if value == crate::remacs_sys::Qnil {
                    allow_read = false
                }
            }
            crate::remacs_sys::QCallow_write => {
                if value == crate::remacs_sys::Qnil {
                    allow_write = false
                }
            }
            crate::remacs_sys::QCallow_run => {
                if value == crate::remacs_sys::Qnil {
                    allow_run = false
                }
            }
            crate::remacs_sys::QCjs_tick_rate => unsafe {
                if crate::remacs_sys::FLOATP(value) {
                    tick_rate = crate::remacs_sys::XFLOAT_DATA(value);
                }
            },
            crate::remacs_sys::QCjs_error_handler => {
                error_handler = value;
            }
            _ => error!(JS_PERMS_ERROR),
        }
    }

    let ops = deno_runtime::permissions::PermissionsOptions {
        allow_net,
        allow_read,
        allow_write,
        allow_run,
        ..Default::default()
    };

    EmacsJsOptions {
        tick_rate,
        ops,
        error_handler,
    }
}

#[lisp_fn(min = "1")]
pub fn eval_js(args: &[LispObject]) -> LispObject {
    let string_obj: LispStringRef = args[0].into();
    let ops = permissions_from_args(&args[1..args.len()]);
    run_module("anon-lisp.js", Some(string_obj.to_utf8()), &ops)
        .unwrap_or_else(move |e| handle_error(e, ops.error_handler))
}

#[lisp_fn(min = "1")]
pub fn eval_js_file(args: &[LispObject]) -> LispObject {
    let filename: LispStringRef = args[0].into();
    let ops = permissions_from_args(&args[1..args.len()]);
    run_module(&filename.to_utf8(), None, &ops)
        .unwrap_or_else(move |e| handle_error(e, ops.error_handler))
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
fn init_once(filepath: &str, js_options: &EmacsJsOptions) -> Result<()> {
    if unsafe { FAILED_TO_INIT } {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Javascript environment failed to initalize, cannot run JS code",
        ));
    }

    let mut once_result: Result<()> = Ok(());
    ONCE.call_once(|| {
        let indirect = || -> Result<()> {
            let mut runtime = tokio::runtime::Builder::new()
                .threaded_scheduler()
                .enable_io()
                .enable_time()
                .max_threads(32)
                .build()?;

            let main_module = deno_core::ModuleSpecifier::resolve_url_or_path(filepath)
                .map_err(|e| into_ioerr(e))?;
            let permissions = deno_runtime::permissions::Permissions::from_options(&js_options.ops);

            // @TODO I'm leaving this line commented out, but we should add this to
            // the init API. Flags listed at https://deno.land/manual/contributing/development_tools
            // v8::V8::set_flags_from_string("--trace-gc --gc-global --gc-interval 1 --heap-profiler-trace-objects");
            let options = deno_runtime::worker::WorkerOptions {
                apply_source_maps: false,
                user_agent: "x".to_string(),
                args: vec![],
                debug_flag: false,
                unstable: false,
                ca_filepath: None,
                seed: None,
                js_error_create_fn: None,
                create_web_worker_cb: std::sync::Arc::new(|_| unreachable!()),
                attach_inspector: false,
                maybe_inspector_server: None,
                should_break_on_first_statement: false,
                module_loader: std::rc::Rc::new(deno_core::FsModuleLoader),
                runtime_version: "x".to_string(),
                ts_version: "x".to_string(),
                no_color: true,
                get_error_class_fn: None,
            };

            let mut worker = deno_runtime::worker::MainWorker::from_options(
                main_module.clone(),
                permissions,
                &options,
            );
            let result: Result<deno_runtime::worker::MainWorker> = runtime.block_on(async move {
                worker.bootstrap(&options);
                let runtime = &mut worker.js_runtime;
                {
                    let context = runtime.global_context();
                    let scope = &mut v8::HandleScope::with_context(runtime.v8_isolate(), context);
                    let context = scope.get_current_context();
                    let global = context.global(scope);
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
                }
                {
                    runtime
                        .execute("prelim.js", include_str!("prelim.js"))
                        .map_err(|e| into_ioerr(e))?
                }

                Ok(worker)
            });

            let worker = result?;
            EmacsJsRuntime::set_main(runtime, worker);
            //(run-with-timer t 1 'js-tick-event-loop error-handler)

            let cstr = CString::new("run-with-timer").expect("Failed to create timer");
            let callback = CString::new("js-tick-event-loop").expect("Failed to create timer");
            unsafe {
                let fun = crate::remacs_sys::intern_c_string(cstr.as_ptr());
                let fun_callback = crate::remacs_sys::intern_c_string(callback.as_ptr());
                let mut args = vec![
                    fun,
                    crate::remacs_sys::Qt,
                    crate::remacs_sys::make_float(js_options.tick_rate),
                    fun_callback,
                    js_options.error_handler,
                ];
                crate::remacs_sys::Ffuncall(args.len().try_into().unwrap(), args.as_mut_ptr());
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

fn run_module(
    filepath: &str,
    additional_js: Option<String>,
    js_options: &EmacsJsOptions,
) -> Result<LispObject> {
    init_once(filepath, js_options)?;

    execute(async move {
        let w = EmacsJsRuntime::worker();
        if let Some(js) = additional_js {
            w.execute(&js).map_err(|e| into_ioerr(e))?;
        } else {
            let main_module = deno_core::ModuleSpecifier::resolve_url_or_path(filepath)
                .map_err(|e| into_ioerr(e))?;
            w.execute_module(&main_module)
                .await
                .map_err(|e| into_ioerr(e))?;
        }

        Ok(())
    })?;

    Ok(crate::remacs_sys::Qnil)
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
                crate::remacs_sys::make_string_from_utf8(cstr.as_ptr(), len.try_into().unwrap());
            let mut args = vec![handler, lstring];
            Ffuncall(args.len().try_into().unwrap(), args.as_mut_ptr())
        }
    }
}

#[lisp_fn]
pub fn js_tick_event_loop(handler: LispObject) -> LispObject {
    tick_js()
        .map(|_| crate::remacs_sys::Qnil)
        .unwrap_or_else(|e| handle_error(e, handler))
}

// @TODO we actually should call this, since it performs runtime actions.
// for now, we are manually calling 'staticpro'
#[allow(dead_code)]
fn init_syms() {
    defvar_lisp!(Vjs_retain_map, "js-retain-map", crate::remacs_sys::Qnil);

    def_lisp_sym!(Qjs_lisp_error, "js-lisp-error");
    def_lisp_sym!(QCallow_net, ":allow-net");
    def_lisp_sym!(QCallow_read, ":allow-read");
    def_lisp_sym!(QCallow_write, ":allow-write");
    def_lisp_sym!(QCallow_run, ":allow-run");
    def_lisp_sym!(QCjs_tick_rate, ":js-tick-rate");
    def_lisp_sym!(Qjs_error, "js-error");
    def_lisp_sym!(QCjs_error_handler, ":js-error-handler");
}

include!(concat!(env!("OUT_DIR"), "/javascript_exports.rs"));
