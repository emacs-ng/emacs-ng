use crate::parsing::{ArrayType, ObjectType};
use lisp::lisp::LispObject;
use lisp::list::{LispCons, LispConsCircularChecks, LispConsEndChecks};
use lisp::multibyte::LispStringRef;
use lisp::remacs_sys::Ffuncall;
use lisp_macros::lisp_fn;
use rusty_v8 as v8;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::ffi::CString;
use std::io::Result;
use std::mem::MaybeUninit;
use std::sync::Arc;

#[derive(Clone)]
struct EmacsJsOptions {
    tick_rate: f64,
    ops: Option<deno_runtime::permissions::Permissions>,
    error_handler: LispObject,
    inspect: Option<String>,
    inspect_brk: Option<String>,
    use_color: bool,
    ts_config: Option<String>,
    no_check: bool,
    no_remote: bool,
}

/// In order to smoothly interface with the Lisp VM,
/// we maintain a bit of global state, which resides in
/// a thread local singleton of this struct.
struct EmacsMainJsRuntime {
    /// The primary Tokio Runtime used for power Async I/O
    /// within the deno_worker. Unlike the deno_worker which
    /// may be destroyed and created multiple times,
    /// the tokio runtime is only created once.
    tokio_runtime: Option<tokio::runtime::Runtime>,
    /// The Primary deno worker, which contains the v8
    /// isolate. Used to execute javascript and interface
    /// with the deno runtime.
    deno_worker: Option<deno_runtime::worker::MainWorker>,
    /// If we are within the current tokio runtime. If we
    /// are within the runtime, we cannot call worker.execute
    /// or worker.execute_module due to a Deno bug. This means
    /// we need to execute an alternative code path for JavaScript
    /// See 'stacked_v8_handle'
    within_runtime: bool,
    /// In order to allow users to re-eval modules upon multiple calls
    /// to (eval-js-file), we append a unique module id and timestamp
    /// to every module evaluation.
    module_counter: u64,
    /// In order to allow for JS -> Lisp -> JS calls, we would need
    /// a valid HandleScope to use. Due to the way Deno is designed,
    /// we cannot just use the Global HandleScrope from the v8::Isolate,
    /// If a HandleScope is above us on the stack, we need to use THAT HandleScope
    /// if we are not provided one, OR we can create a new HandleScope NOT from
    /// the v8 Isolate. Due to this, when we call a JS function that may go
    /// JS -> Lisp -> JS (or even deeper, say JS -> Lisp -> JS -> Lisp -> JS),
    /// we store the pointer value of the current handle scope. By doing this,
    /// WE BREAK RUST'S BIGGEST RULE, and create two mutable references to
    /// the same variable. However, the only above us on the stack
    /// cannot be touched, by careful design. The native C++ doesn't know about
    /// lifetimes or Rust's rules, so doing this doesn't affect the underlying v8.
    /// This is inheriently fragile, but we do it in a very
    /// careful way that really only makes it a valid lifetime extension.
    /// If there is every a stray SEGFAULT or general issue with this class,
    /// it will likely be caused by mishandling of this variable.
    stacked_v8_handle: Option<*mut v8::HandleScope<'static>>,
    /// The optiosn passed to (js-initialize) to customize the
    /// JS runtime.
    options: EmacsJsOptions,
    /// Proxies are created by a global template, stored in this
    /// field
    proxy_template: Option<v8::Global<v8::ObjectTemplate>>,
    /// The deno program state for our worker. Usually not touched,
    /// it may be sometimes references to refer to certain variables
    /// not stored in EmacsJsOptions.
    program_state: Option<Arc<deno::program_state::ProgramState>>,
    within_toplevel: bool,
    tick_scheduled: bool,
}

impl Default for EmacsMainJsRuntime {
    fn default() -> Self {
        Self {
            tokio_runtime: None,
            deno_worker: None,
            within_runtime: false,
            module_counter: 0,
            stacked_v8_handle: None,
            options: EmacsJsOptions::default(),
            proxy_template: None,
            program_state: None,
            within_toplevel: false,
            tick_scheduled: false,
        }
    }
}

impl Default for EmacsJsOptions {
    fn default() -> Self {
        Self {
            tick_rate: 0.001,
            ops: None,
            error_handler: lisp::remacs_sys::Qnil,
            inspect: None,
            inspect_brk: None,
            use_color: false,
            ts_config: None,
            no_check: false,
            no_remote: false,
        }
    }
}

impl Drop for EmacsMainJsRuntime {
    fn drop(&mut self) {
        if let Some(runtime) = self.tokio_runtime.take() {
            // This is to prevent a panic if we are dropping within an async runtime.
            // The only time we drop this is during program shutdown, or thread shutdown,
            // and in either case, we don't want to block on pending tasks, or panic in
            // general.
            runtime.shutdown_background();
        }
    }
}

thread_local! {
    static MAIN: RefCell<EmacsMainJsRuntime> = RefCell::new(EmacsMainJsRuntime::default());
}

struct MainWorkerHandle {
    worker: Option<deno_runtime::worker::MainWorker>,
}

impl Drop for MainWorkerHandle {
    fn drop(&mut self) {
        EmacsMainJsRuntime::set_deno_worker(self.worker.take().unwrap());
    }
}

impl MainWorkerHandle {
    fn new(worker: deno_runtime::worker::MainWorker) -> Self {
        Self {
            worker: Some(worker),
        }
    }

    fn as_mut_ref(&mut self) -> &mut deno_runtime::worker::MainWorker {
        self.worker.as_mut().unwrap()
    }
}

impl EmacsMainJsRuntime {
    fn access<F: Sized, T: FnOnce(&mut std::cell::RefMut<'_, EmacsMainJsRuntime>) -> F>(t: T) -> F {
        let mut input: MaybeUninit<F> = MaybeUninit::<F>::uninit();
        let input_ref = &mut input;
        MAIN.with(move |main| {
            let mut x = main.borrow_mut();
            input_ref.write(t(&mut x));
        });

        unsafe { input.assume_init() }
    }

    fn set_program_state(program: Arc<deno::program_state::ProgramState>) {
        Self::access(move |main| main.program_state = Some(program));
    }

    fn get_program_state() -> Arc<deno::program_state::ProgramState> {
        Self::access(|main| main.program_state.as_ref().unwrap().clone())
    }

    fn set_proxy_template(global: v8::Global<v8::ObjectTemplate>) {
        Self::access(move |main| main.proxy_template = Some(global));
    }

    fn get_proxy_template() -> v8::Global<v8::ObjectTemplate> {
        Self::access(|main| main.proxy_template.clone().unwrap())
    }

    fn get_options() -> EmacsJsOptions {
        Self::set_default_perms_if_unset();
        Self::access(|main| main.options.clone())
    }

    fn set_options(options: EmacsJsOptions) {
        Self::access(move |main| main.options = options);
    }

    unsafe fn get_stacked_v8_handle<'a>() -> &'a mut v8::HandleScope<'a> {
        Self::access(|main| {
            std::mem::transmute::<*mut v8::HandleScope<'static>, &'a mut v8::HandleScope>(
                main.stacked_v8_handle.unwrap(),
            )
        })
    }

    unsafe fn set_stacked_v8_handle(handle_opt: Option<&mut v8::HandleScope>) {
        Self::access(|main| {
            main.stacked_v8_handle = handle_opt.map(|handle| {
                std::mem::transmute::<&mut v8::HandleScope, *mut v8::HandleScope<'static>>(handle)
            });
        });
    }

    fn get_raw_v8_handle() -> Option<*mut v8::HandleScope<'static>> {
        Self::access(|main| main.stacked_v8_handle.take())
    }

    fn set_raw_v8_handle(o: Option<*mut v8::HandleScope<'static>>) {
        Self::access(move |main| main.stacked_v8_handle = o);
    }

    fn inc_module_counter() -> u64 {
        Self::access(|main| {
            main.module_counter += 1;
            main.module_counter
        })
    }

    fn _set_toplevel(b: bool) {
        Self::access(move |main| main.within_toplevel = b);
    }

    fn enter_toplevel_module() {
        Self::_set_toplevel(true);
    }

    fn exit_toplevel_module() {
        Self::_set_toplevel(false);
    }

    fn is_within_toplevel_module() -> bool {
        Self::access(|main| main.within_toplevel)
    }

    fn enter_runtime() {
        Self::_set_within_runtime(true);
    }

    fn exit_runtime() {
        Self::_set_within_runtime(false);
    }

    fn _set_within_runtime(within_runtime: bool) {
        Self::access(move |main| main.within_runtime = within_runtime);
    }

    fn is_within_runtime() -> bool {
        Self::access(|main| main.within_runtime)
    }

    fn set_tokio_runtime(r: tokio::runtime::Runtime) {
        Self::access(move |main| main.tokio_runtime = Some(r));
    }

    fn get_tokio_handle() -> tokio::runtime::Handle {
        Self::access(|main| main.tokio_runtime.as_ref().unwrap().handle().clone())
    }

    fn is_tokio_active() -> bool {
        Self::access(|main| main.tokio_runtime.is_some())
    }

    fn destroy_worker() {
        Self::access(|main| {
            main.proxy_template = None;
            main.deno_worker = None;
        });
    }

    fn get_deno_worker() -> MainWorkerHandle {
        Self::access(|main| MainWorkerHandle::new(main.deno_worker.take().unwrap()))
    }

    fn set_deno_worker(worker: deno_runtime::worker::MainWorker) {
        Self::access(move |main| main.deno_worker = Some(worker));
    }

    fn is_main_worker_active() -> bool {
        Self::access(|main| main.deno_worker.is_some())
    }

    fn set_default_perms_if_unset() {
        Self::access(|main| {
            if main.options.ops.is_none() {
                main.options.ops = Some(deno_runtime::permissions::Permissions::allow_all())
            }
        });
    }

    fn set_tick_scheduled(b: bool) {
        Self::access(|main| main.tick_scheduled = b);
    }

    fn get_tick_scheduled() -> bool {
        Self::access(|main| main.tick_scheduled)
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
macro_rules! unique_module {
    ($format: expr) => {{
        let counter = EmacsMainJsRuntime::inc_module_counter();
        let time = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!($format, counter, time)
    }};
}

macro_rules! unique_module_import {
    ($filename: expr) => {{
        let counter = EmacsMainJsRuntime::inc_module_counter();
        let time = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("import '{}#{}{}';", $filename, counter, time)
    }};
}

// Aligned with code in prelim.js
const HASHTABLE: u32 = 0;
const ALIST: u32 = 1;
const PLIST: u32 = 2;
const ARRAY: u32 = 3;
const LIST: u32 = 4;

macro_rules! make_proxy {
    ($scope:expr, $lisp:expr) => {{
        let template = EmacsMainJsRuntime::get_proxy_template();
        let tpl = template.get($scope);
        let obj = tpl.new_instance($scope).unwrap();
        let value = v8::String::new($scope, &$lisp.to_C_unsigned().to_string()).unwrap();
        let inserted = obj.set_internal_field(0, v8::Local::<v8::Value>::try_from(value).unwrap());
        assert!(inserted);

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
    // is reflected in EmacsMainJsRuntime. If you are
    // not within the runtime, you can run worker.execute. If you are within
    // the runtime, you mem::transmute this scope and use that.
    // Think of it like passing down scope through every function call until we
    // need it, but in a round about way.
    // >>> Code touching EmacsMainJsRuntime::{get/set}_stacked_v8_handle or
    // >>> EmacsMainJsRuntime::{enter/exit}_runtime needs to be
    // >>> managed very carefully. It should not be touched without a good reason.
    let current = unsafe {
        let cur = EmacsMainJsRuntime::get_raw_v8_handle();
        EmacsMainJsRuntime::set_stacked_v8_handle(Some(scope));
        cur
    };
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
        scope = EmacsMainJsRuntime::get_stacked_v8_handle();
        EmacsMainJsRuntime::set_raw_v8_handle(current);
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
const JS_PERMS_ERROR: &str =
    "Valid options are: :allow-net nil :allow-read nil :allow-write nil :allow-run nil";
fn permissions_from_args(args: &[LispObject]) -> EmacsJsOptions {
    let mut options = EmacsJsOptions {
        ops: Some(deno_runtime::permissions::Permissions::allow_all()),
        ..Default::default()
    };

    // Safe since it was just set.
    let permissions = options.ops.as_mut().unwrap();

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
                    options.tick_rate = lisp::remacs_sys::XFLOAT_DATA(value);
                }
            },
            lisp::remacs_sys::QCjs_error_handler => {
                options.error_handler = value;
            }
            lisp::remacs_sys::QCinspect => {
                if value.is_string() {
                    options.inspect = Some(value.as_string().unwrap().to_utf8());
                } else if value == lisp::remacs_sys::Qt {
                    options.inspect = Some(DEFAULT_ADDR.to_string());
                }
            }
            lisp::remacs_sys::QCinspect_brk => {
                if value.is_string() {
                    options.inspect_brk = Some(value.as_string().unwrap().to_utf8());
                } else if value == lisp::remacs_sys::Qt {
                    options.inspect_brk = Some(DEFAULT_ADDR.to_string());
                }
            }
            lisp::remacs_sys::QCuse_color => {
                if value.is_t() {
                    options.use_color = true;
                }
            }
            lisp::remacs_sys::QCts_config => {
                let sref: LispStringRef = value.into();
                let rstring = sref.to_utf8();
                options.ts_config = Some(rstring);
            }
            lisp::remacs_sys::QCno_check => {
                if value.is_t() {
                    options.no_check = true;
                }
            }
            lisp::remacs_sys::QCno_remote => {
                if value.is_t() {
                    options.no_remote = true;
                }
            }

            _ => error!(JS_PERMS_ERROR),
        }
    }

    options
}

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
fn destroy_worker_on_promise_rejection(e: &std::io::Error) {
    let is_within_toplevel = EmacsMainJsRuntime::is_within_toplevel_module();
    if is_within_toplevel
        && e.kind() == std::io::ErrorKind::Other
        && e.to_string().starts_with("Uncaught (in promise)")
    {
        EmacsMainJsRuntime::destroy_worker();
    }
}

// I'm keeping the logic simple  for now,
// if it doesn't end with .js, we will
// treat it as ts
fn is_typescript(s: &str) -> bool {
    !s.ends_with("js")
}

/// Evaluates CODE as JavaScript on the main emacs thread.
/// If :typescript t is passed as an argument, evaluate
/// CODE as TypeScript. If a TypeScript compile error
/// is generated by CODE, error will be called
/// with the TypeScript error generated. Any runtime
/// JavaScript errors will generate a call to error.
///
/// If the evaluated JavaScript generates a top-level
/// Promise rejection, the JavaScript environment will be
/// reset and reinitalized lazily. If that happens, all
/// global state will be reset. This can be prevented by
/// implementing a top level Promise error handler.
#[lisp_fn(min = "1")]
pub fn eval_js(args: &[LispObject]) -> LispObject {
    let string_obj: LispStringRef = args[0].into();
    let ops = EmacsMainJsRuntime::get_options();
    let name = unique_module!("./$anon$lisp${}{}.ts");
    let string = string_obj.to_utf8();
    let is_typescript = args.len() == 3
        && args[1] == lisp::remacs_sys::QCtypescript
        && args[2] == lisp::remacs_sys::Qt;

    run_module(&name, Some(string), &ops, is_typescript)
}

/// Reads and evaluates FILENAME as a JavaScript module on
/// the main emacs thread. If the file does not end in '.js',
/// it will be evaluated as TypeScript. If :typescript t is
/// passed, the file will be evaluated at TypeScript.
///
/// Repeated calls to eval-js-file will re-evaluate the
/// provided module for the same file. This is not how
/// ES6 modules normally work - they are designed to be
/// immutable. However this is not inline with user's
/// expectation of this function.
///
/// If the evaluated JavaScript generates a top-level
/// Promise rejection, the JavaScript environment will be
/// reset and reinitalized lazily. If that happens, all
/// global state will be reset. This can be prevented by
/// implementing a top level Promise error handler.
#[lisp_fn(min = "1")]
pub fn eval_js_file(args: &[LispObject]) -> LispObject {
    let filename: LispStringRef = args[0].into();
    let ops = EmacsMainJsRuntime::get_options();
    let mut module = filename.to_utf8();
    let is_typescript = (args.len() == 3
        && args[1] == lisp::remacs_sys::QCtypescript
        && args[2] == lisp::remacs_sys::Qt)
        || is_typescript(&module);

    // This is a hack to allow for our behavior of
    // executing a module multiple times.
    let import = unique_module_import!(module);
    module = unique_module!("./$import${}{}.ts");
    run_module(&module, Some(import), &ops, is_typescript)
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

/// Evaluate the contents of BUFFER as JavaScript.
///
/// If the evaluated JavaScript generates a top-level
/// Promise rejection, the JavaScript environment will be
/// reset and reinitalized lazily. If that happens, all
/// global state will be reset. This can be prevented by
/// implementing a top level Promise error handler.
#[lisp_fn(min = "0", intspec = "")]
pub fn eval_js_buffer(buffer: LispObject) -> LispObject {
    let lisp_string = get_buffer_contents(buffer);
    eval_js(&[lisp_string])
}

/// Evaluate the contents of BUFFER as TypeScript.
///
/// If the evaluated JavaScript generates a top-level
/// Promise rejection, the JavaScript environment will be
/// reset and reinitalized lazily. If that happens, all
/// global state will be reset. This can be prevented by
/// implementing a top level Promise error handler.
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

/// Evaluate the contents of REGION as JavaScript.
///
/// If the evaluated JavaScript generates a top-level
/// Promise rejection, the JavaScript environment will be
/// reset and reinitalized lazily. If that happens, all
/// global state will be reset. This can be prevented by
/// implementing a top level Promise error handler.
#[lisp_fn(intspec = "r")]
pub fn eval_js_region(start: LispObject, end: LispObject) -> LispObject {
    let lisp_string = get_region(start, end);
    eval_js(&[lisp_string])
}

/// Evaluate the contents of REGION as TypeScript.
///
/// If the evaluated JavaScript generates a top-level
/// Promise rejection, the JavaScript environment will be
/// reset and reinitalized lazily. If that happens, all
/// global state will be reset. This can be prevented by
/// implementing a top level Promise error handler.
#[lisp_fn(intspec = "r")]
pub fn eval_ts_region(start: LispObject, end: LispObject) -> LispObject {
    let lisp_string = get_region(start, end);
    eval_js(&[
        lisp_string,
        lisp::remacs_sys::QCtypescript,
        lisp::remacs_sys::Qt,
    ])
}

/// Initalizes the JavaScript runtime. If this function is not
/// called prior to eval-js*, the runtime will be lazily initialized
/// js-initialize takes arguments that allow the JavaScript runtime
/// to be customized:
///
/// :allow-net nil - Prevents JS from accessing the network
/// :allow-write nil - Prevents JS from writing to the file system
/// :allow-read nil - Prevents JS from reading the file system
/// :allow-run nil - Prevents JS from executing sub-processes
/// :js-tick-rate - Defaults to 0.1. This is the interval that js
/// will evaluate if there are any resolved pending async operations
/// and execute callbacks.
/// :use-color - Will print JS error messages in color. Defaults to
/// off due to formatting issues with JS errors invoked with (error ...)
/// :inspect "IP:PORT" - Enables the JavaScript debugger, listening with
/// address IP:PORT. Will allow attachment of JavaScript debugger. If passed
/// t, will default to 127.0.0.1:9229
/// :inspect-brk "IP:PORT" - Like :inspect, but instead it will break on
/// the first line of JS evaluated to allow the user to connect to the debugger
/// prior to their code being executed.
/// :js-error-handler 'function - A function to call if a JS error occures, including
/// TypeScript compile errors. If not specified, will default to (error ...)
/// :ts-config PATH - Specifies the file path to your custom tsconfig json file
/// see https://www.typescriptlang.org/docs/handbook/tsconfig-json.html
/// :no-check t - disables TypeScript type checking. Can be used to gain performance
/// when the user does not want or require typechecking
/// :no-remote t - disables the import of remote files via import statements.
/// This option still allows network options via calls to fetch(...)
#[lisp_fn]
pub fn js_initialize(args: &[LispObject]) -> LispObject {
    let ops = permissions_from_args(args);
    EmacsMainJsRuntime::set_options(ops.clone());
    EmacsMainJsRuntime::enter_toplevel_module();
    run_module("init.js", Some("".to_string()), &ops, false)
}

/// Destroys the current JavaScript environment. The JavaScript environment will be
/// reinitalized upon the next call to eval-js*, or to js-initialize
#[lisp_fn]
pub fn js_cleanup() -> LispObject {
    EmacsMainJsRuntime::destroy_worker();
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
    inner_invokation(move |scope| js_reenter_inner(scope, args), true)
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

fn inner_invokation<F, R: Sized>(f: F, should_schedule: bool) -> R
where
    F: Fn(&mut v8::HandleScope) -> R,
{
    let result;
    if !EmacsMainJsRuntime::is_within_runtime() {
        let mut worker_handle = EmacsMainJsRuntime::get_deno_worker();
        let worker = worker_handle.as_mut_ref();
        let runtime = &mut worker.js_runtime;
        let context = runtime.global_context();
        let scope = &mut v8::HandleScope::with_context(runtime.v8_isolate(), context);
        let handle = EmacsMainJsRuntime::get_tokio_handle();
        EmacsMainJsRuntime::enter_runtime();
        result = handle.block_on(async move { f(scope) });
        EmacsMainJsRuntime::exit_runtime();

        // Only in the case that the event loop as gone to sleep,
        // we want to reinvoke it, in case the above
        // invokation has scheduled promises.
        if !EmacsMainJsRuntime::get_tick_scheduled() && should_schedule {
            schedule_tick();
        }
    } else {
        let scope: &mut v8::HandleScope = unsafe { EmacsMainJsRuntime::get_stacked_v8_handle() };
        result = f(scope);
        unsafe { EmacsMainJsRuntime::set_stacked_v8_handle(Some(scope)) };
    }

    result
}

#[lisp_fn]
pub fn js__clear(idx: LispObject) -> LispObject {
    inner_invokation(move |scope| js_clear_internal(scope, idx), false);
    lisp::remacs_sys::Qnil
}

fn into_ioerr<E: Into<Box<dyn std::error::Error + Send + Sync>>>(e: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e)
}

fn execute<T: Sized + std::future::Future<Output = Result<()>>>(fnc: T) -> Result<()> {
    if EmacsMainJsRuntime::is_within_runtime() {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Attempted to execute javascript from lisp within the javascript context. Javascript is not re-entrant, cannot call JS -> Lisp -> JS"))
    } else {
        let handle = EmacsMainJsRuntime::get_tokio_handle();
        EmacsMainJsRuntime::enter_runtime();
        let result = handle.block_on(fnc);
        EmacsMainJsRuntime::exit_runtime();
        result
    }
}

fn js_sweep_inner(scope: &mut v8::HandleScope) {
    let context = scope.get_current_context();
    let global = context.global(scope);

    let name = v8::String::new(scope, "__sweep").unwrap();
    let fnc: v8::Local<v8::Function> = global.get(scope, name.into()).unwrap().try_into().unwrap();
    let recv =
        v8::Local::<v8::Value>::try_from(v8::String::new(scope, "lisp_invoke").unwrap()).unwrap();
    let v8_args = vec![];
    fnc.call(scope, recv, v8_args.as_slice()).unwrap();
}

#[lisp_fn]
pub fn js__sweep() -> LispObject {
    if EmacsMainJsRuntime::is_main_worker_active() {
        inner_invokation(|scope| js_sweep_inner(scope), false);
    }

    lisp::remacs_sys::Qnil
}

fn tick_js() -> Result<bool> {
    let mut is_complete = false;
    let is_complete_ref = &mut is_complete;
    execute(async move {
        futures::future::poll_fn(|cx| {
            let mut worker_handle = EmacsMainJsRuntime::get_deno_worker();
            let w = worker_handle.as_mut_ref();
            let polled = w.poll_event_loop(cx);
            match polled {
                std::task::Poll::Ready(r) => {
                    *is_complete_ref = true;
                    r.map_err(|e| into_ioerr(e))?
                }
                std::task::Poll::Pending => {}
            }

            std::task::Poll::Ready(Ok(()))
        })
        .await
    })
    .map(move |_| is_complete)
}

fn init_once() -> Result<()> {
    if !EmacsMainJsRuntime::is_tokio_active() {
        let runtime = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_io()
            .enable_time()
            .max_threads(32)
            .build()?;

        EmacsMainJsRuntime::set_tokio_runtime(runtime);
    }

    Ok(())
}

fn init_worker(filepath: &str, js_options: &EmacsJsOptions) -> Result<()> {
    if EmacsMainJsRuntime::is_main_worker_active() {
        return Ok(());
    }

    let runtime = EmacsMainJsRuntime::get_tokio_handle();
    let main_module =
        deno_core::ModuleSpecifier::resolve_url_or_path(filepath).map_err(|e| into_ioerr(e))?;
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

    if !js_options.use_color {
        std::env::set_var("NO_COLOR", "1");
    }

    let flags = deno::flags::Flags {
        unstable: true, // Needed for deno in WebWorkers
        no_check: js_options.no_check,
        no_remote: js_options.no_remote,
        config_path: js_options.ts_config.clone(),
        inspect,
        inspect_brk,
        ..Default::default()
    };

    let program = deno::program_state::ProgramState::new(flags).map_err(|e| into_ioerr(e))?;
    EmacsMainJsRuntime::set_program_state(program.clone());
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
                EmacsMainJsRuntime::set_proxy_template(glob);
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
    EmacsMainJsRuntime::set_deno_worker(worker);
    Ok(())
}

fn run_module_inner(
    filepath: &str,
    additional_js: Option<String>,
    js_options: &EmacsJsOptions,
    as_typescript: bool,
) -> Result<LispObject> {
    init_once()?;
    init_worker(filepath, js_options)?;

    execute(async move {
        let mut worker_handle = EmacsMainJsRuntime::get_deno_worker();
        let w = worker_handle.as_mut_ref();
        let main_module =
            deno_core::ModuleSpecifier::resolve_url_or_path(filepath).map_err(|e| into_ioerr(e))?;

        let main_module_url = main_module.as_url().to_owned();
        if let Some(js) = additional_js {
            let program = EmacsMainJsRuntime::get_program_state();
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

    schedule_tick();
    Ok(lisp::remacs_sys::Qnil)
}

fn run_module(
    filepath: &str,
    additional_js: Option<String>,
    js_options: &EmacsJsOptions,
    as_typescript: bool,
) -> LispObject {
    EmacsMainJsRuntime::enter_toplevel_module();
    let result = run_module_inner(filepath, additional_js, js_options, as_typescript)
        .unwrap_or_else(move |e| {
            destroy_worker_on_promise_rejection(&e);
            handle_error(e, js_options.error_handler)
        });
    EmacsMainJsRuntime::exit_toplevel_module();
    result
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

fn schedule_tick() {
    EmacsMainJsRuntime::set_tick_scheduled(true);
    let js_options = EmacsMainJsRuntime::get_options();
    //(run-with-timer t 0.1 'js-tick-event-loop error-handler)
    let cstr = CString::new("run-with-timer").expect("Failed to create timer");
    let callback = CString::new("js-tick-event-loop").expect("Failed to create timer");
    unsafe {
        let fun = lisp::remacs_sys::intern_c_string(cstr.as_ptr());
        let fun_callback = lisp::remacs_sys::intern_c_string(callback.as_ptr());
        let mut args = vec![
            fun,
            lisp::remacs_sys::make_float(js_options.tick_rate),
            lisp::remacs_sys::Qnil,
            fun_callback,
            js_options.error_handler,
        ];
        lisp::remacs_sys::Ffuncall(args.len().try_into().unwrap(), args.as_mut_ptr());
    }
}

#[lisp_fn]
pub fn js_tick_event_loop(handler: LispObject) -> LispObject {
    if !EmacsMainJsRuntime::is_main_worker_active() {
        return lisp::remacs_sys::Qnil;
    }

    // If we are within the runtime, we don't want to attempt to
    // call execute, as we will error, and there really isn't anything
    // anyone can do about it. Just defer the event loop until
    // we are out of the runtime.
    if EmacsMainJsRuntime::is_within_runtime() {
        schedule_tick();
        return lisp::remacs_sys::Qnil;
    }

    let is_complete = tick_js()
        // We do NOT want to destroy the MainWorker if we error here.
        // We can still use this isolate for future promise resolutions
        // instead, just pass to the error handler.
        .unwrap_or_else(|e| {
            handle_error(e, handler);
            false
        });

    if !is_complete {
        schedule_tick();
    } else {
        EmacsMainJsRuntime::set_tick_scheduled(false);
    }

    lisp::remacs_sys::Qnil
}

// Do NOT call this function, it is just used for macro purposes to
// generate variables. The user should NOT have direct access to
// 'js-retain-map' from the scripting engine.
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
    def_lisp_sym!(QCuse_color, ":use-color");

    def_lisp_sym!(QCts_config, ":ts-config");
    def_lisp_sym!(QCno_check, ":no-check");
    def_lisp_sym!(QCno_remote, ":no-remote");
}

include!(concat!(env!("OUT_DIR"), "/javascript_exports.rs"));
