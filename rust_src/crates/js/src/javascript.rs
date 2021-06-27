use std::cell::RefCell;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::sync::Arc;

use crate::futures::FutureExt;
use futures::Future;
use rusty_v8 as v8;

use emacs::bindings::Ffuncall;
use emacs::definitions::EmacsUint;
use emacs::lisp::LispObject;
use emacs::list::{LispCons, LispConsCircularChecks, LispConsEndChecks};
use emacs::multibyte::LispStringRef;
use lisp_macros::lisp_fn;
use lsp_json::parsing::{ArrayType, ObjectType};

pub type EmacsJsError = deno_core::error::AnyError;
pub type EmacsJsResult<T> = Result<T, EmacsJsError>;

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
    loops_per_tick: EmacsUint,
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
    /// In order to execute v8 Operations, you need a valid "Scope" Object.
    /// Certain functions, like `block_on` and `MainWorker::execute` internally
    /// handle creating that scope object for you. However,
    /// we cannot always use `block_on`, or `MainWorker::execute` due to the
    /// fact that either of those approaches will cause an intentional panic
    /// if we have already called them further up the function stack.
    /// In the case of `block_on`, we would be blocking on an already blocked
    /// runtime, and in the case of MainWorker::execute, we would be attempting
    /// to access the v8 Isolate's base Scope while there are other scopes on the
    /// scope stack.
    /// This variable is only relevent in the case that we call Lisp -> JS -> Lisp -> JS
    /// In that event, when we are in the second JS invocation, we want to use
    /// the HandleScope reference that is above us on the stack.
    /// This is safe because the JavaScript and Lisp runtimes are synchronous
    /// and we carefully manage the code to ensure that all of the invariants
    /// needed to use a pointer this way are maintained.
    /// Source: https://doc.rust-lang.org/std/primitive.pointer.html#method.as_mut
    /// We must ALWAYS fufill the following conditions:
    /// You have to ensure that either the pointer is NULL or all of the following is true:
    /// 1. The pointer must be properly aligned.
    /// 2. It must be "dereferencable" in the sense defined in the module documentation.
    /// 3. The pointer must point to an initialized instance of T.
    /// 4. You must enforce Rust's aliasing rules, since the returned lifetime 'a is arbitrarily
    /// chosen and does not necessarily reflect the actual lifetime of the data. In particular, for
    /// the duration of this lifetime, the memory the pointer points to must not get accessed (read
    /// or written) through any other pointer.
    /// This applies even if the result of this method is unused! (The part about being initialized
    /// is not yet fully decided, but until it is, the only safe approach is to ensure that they
    /// are indeed initialized.)
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
    /// If the program is within a toplevel module evaluation. If we are
    /// within a toplevel module evaluation and have an unhandled promise exception
    /// the deno runtime will be posioned, and we will need to re-initialize JS
    within_toplevel: bool,
    /// If currently have a pending tick of the JS event loop scheduled.
    /// We will only schedule one tick at a time.
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
            tick_rate: 0.25,
            ops: None,
            error_handler: emacs::globals::Qnil,
            inspect: None,
            inspect_brk: None,
            use_color: false,
            ts_config: None,
            no_check: false,
            no_remote: false,
            loops_per_tick: 1000,
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

struct RuntimeHandle {
    worker: Option<tokio::runtime::Runtime>,
}

impl Drop for RuntimeHandle {
    fn drop(&mut self) {
        EmacsMainJsRuntime::set_tokio_runtime(self.worker.take().unwrap());
    }
}

impl RuntimeHandle {
    fn new(worker: tokio::runtime::Runtime) -> Self {
        Self {
            worker: Some(worker),
        }
    }

    fn as_mut_ref(&mut self) -> &mut tokio::runtime::Runtime {
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

    fn push_stack(scope: &mut v8::HandleScope) -> Option<*mut v8::HandleScope<'static>> {
        let current = Self::_pop_handle();
        Self::_push_handle(Some(scope));
        current
    }

    fn peek_stack<'a>() -> &'a mut v8::HandleScope<'a> {
        Self::access(|main| {
            let ptr = main.stacked_v8_handle.unwrap() as *const _ as *mut v8::HandleScope<'a>;
            // Since this ptr is always derived from a valid
            // reference, this is safe
            unsafe { ptr.as_mut() }.unwrap()
        })
    }

    fn restore_stack<'a>(
        current: Option<*mut v8::HandleScope<'static>>,
    ) -> &'a mut v8::HandleScope<'a> {
        let scope = Self::peek_stack();
        Self::_push_ptr(current);
        scope
    }

    fn _push_handle(handle_opt: Option<&mut v8::HandleScope>) {
        Self::access(|main| {
            main.stacked_v8_handle =
                handle_opt.map(|handle| handle as *const _ as *mut v8::HandleScope<'static>);
        });
    }

    fn _pop_handle() -> Option<*mut v8::HandleScope<'static>> {
        Self::access(|main| main.stacked_v8_handle.take())
    }

    fn _push_ptr(o: Option<*mut v8::HandleScope<'static>>) {
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

    fn get_tokio_handle() -> RuntimeHandle {
        Self::access(|main| RuntimeHandle::new(main.tokio_runtime.take().unwrap()))
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

    fn get_loops_per_tick() -> EmacsUint {
        Self::access(|main| main.options.loops_per_tick)
    }
}

fn is_interactive() -> bool {
    unsafe { !emacs::bindings::globals.noninteractive1 }
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
            if emacs::bindings::globals.Vjs_retain_map == emacs::globals::Qnil {
                emacs::bindings::globals.Vjs_retain_map =
                    LispObject::cons($lisp, emacs::globals::Qnil);
                emacs::bindings::staticpro(&emacs::bindings::globals.Vjs_retain_map);
            } else {
                emacs::bindings::globals.Vjs_retain_map =
                    LispObject::cons($lisp, emacs::bindings::globals.Vjs_retain_map);
            }
        }

        obj
    }};
}

macro_rules! unproxy {
    ($scope:expr, $obj:expr) => {{
        let internal = $obj.get_internal_field($scope, 0).unwrap();
        let ptrstr = internal
            .to_string($scope)
            .unwrap()
            .to_rust_string_lossy($scope);
        LispObject::from_C_unsigned(ptrstr.parse::<emacs::definitions::EmacsUint>().unwrap())
    }};
}

macro_rules! make_reverse_proxy {
    ($scope:expr, $lisp:expr) => {{
        make_proxy!($scope, LispObject::cons(emacs::globals::Qjs_proxy, $lisp))
    }};
}

macro_rules! bind_global_fn {
    ($scope:expr, $global: expr, $fnc:ident) => {{
        let name = v8::String::new($scope, stringify!($fnc)).unwrap();
        let func = v8::Function::new($scope, $fnc).unwrap();
        $global.set($scope, name.into(), func.into());
    }};
}

fn throw_exception_with_error<E: std::error::Error>(scope: &mut v8::HandleScope, e: E) {
    let error_string = e.to_string();
    let error = v8::String::new(scope, &error_string).unwrap();
    let exception = v8::Exception::error(scope, error);
    scope.throw_exception(exception);
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

    let mut base_config = lsp_json::parsing::gen_ser_deser_config();

    match option {
        HASHTABLE => base_config.obj = ObjectType::Hashtable,
        ALIST => base_config.obj = ObjectType::Alist,
        PLIST => base_config.obj = ObjectType::Plist,
        ARRAY => base_config.arr = ArrayType::Array,
        LIST => base_config.arr = ArrayType::List,
        _ => { /* noop */ }
    }

    let deser_result = lsp_json::parsing::deser(&message, Some(base_config));
    match deser_result {
        Ok(result) => {
            let proxy = make_proxy!(scope, result);
            let r = v8::Local::<v8::Value>::try_from(proxy).unwrap();
            retval.set(r);
        }
        Err(e) => {
            throw_exception_with_error(scope, e);
            return;
        }
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
            emacs::globals::Qjs__clear,
            emacs::bindings::make_fixnum(len.into()),
        ];
        let list = emacs::bindings::Flist(bound.len().try_into().unwrap(), bound.as_mut_ptr());
        let mut lambda = vec![emacs::globals::Qlambda, emacs::globals::Qnil, list];
        let lambda_list =
            emacs::bindings::Flist(lambda.len().try_into().unwrap(), lambda.as_mut_ptr());
        emacs::bindings::Fmake_finalizer(lambda_list)
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

    let llen = unsafe { emacs::bindings::make_fixnum(len.into()) };

    // WHAT IS THIS?!
    // This is doing the following in native code:
    // (lambda (&REST) (js--reenter llen (make-finalizer (lambda () js--clear llen)) REST))
    // To walk through it, this is a lambda that will call js--reenter with the index of our
    // js lambda. In order to clean all this garbage up, we make a finalizer that will call
    // js--clear to null out that lambda, to 'release' it from the JS GC. JS lambdas bound
    // this way just live in a global array, and js--clear just removes them from that array.
    let finalizer = unsafe {
        let mut bound = vec![emacs::globals::Qjs__clear, llen];
        let list = emacs::bindings::Flist(bound.len().try_into().unwrap(), bound.as_mut_ptr());
        let mut fargs = vec![emacs::globals::Qand_rest, emacs::globals::Qalpha];
        let fargs_list =
            emacs::bindings::Flist(fargs.len().try_into().unwrap(), fargs.as_mut_ptr());

        let mut lambda = vec![emacs::globals::Qlambda, fargs_list, list];
        let lambda_list =
            emacs::bindings::Flist(lambda.len().try_into().unwrap(), lambda.as_mut_ptr());
        emacs::bindings::Fmake_finalizer(lambda_list)
    };

    let mut inner = vec![emacs::globals::Qjs__reenter, llen, finalizer];
    if num_args > 0 {
        inner.push(emacs::globals::Qalpha);
    }

    let result =
        unsafe { emacs::bindings::Flist(inner.len().try_into().unwrap(), inner.as_mut_ptr()) };

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
    let c_alloc = CString::new(message);
    match c_alloc {
        Ok(cstr) => {
            let result = unsafe {
                emacs::bindings::make_string_from_utf8(cstr.as_ptr(), len.try_into().unwrap())
            };
            let proxy = make_proxy!(scope, result);
            let r = v8::Local::<v8::Value>::try_from(proxy).unwrap();
            retval.set(r);
        }
        Err(e) => {
            throw_exception_with_error(scope, e);
            return;
        }
    }
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

    let result = unsafe { emacs::bindings::make_fixnum(message) };
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

    let result = unsafe { emacs::bindings::make_float(message) };
    let proxy = make_proxy!(scope, result);
    let r = v8::Local::<v8::Value>::try_from(proxy).unwrap();
    retval.set(r);
}

pub fn lisp_intern(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let lispobj = unproxy!(scope, args.get(0).to_object(scope).unwrap());
    let result = unsafe { emacs::bindings::Fintern(lispobj, emacs::globals::Qnil) };
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

            let deser_result = lsp_json::parsing::deser(&a, None);
            match deser_result {
                Ok(deser) => {
                    lisp_args.push(deser);
                }
                Err(e) => {
                    throw_exception_with_error(scope, e);
                    return;
                }
            }
        } else if arg.is_object() {
            let lispobj = unproxy!(scope, arg.to_object(scope).unwrap());
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
        emacs::bindings::Flist(lisp_args.len().try_into().unwrap(), lisp_args.as_mut_ptr())
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
        let lispobj = unproxy!(scope, args.get(0).to_object(scope).unwrap());
        if let Ok(json) = lsp_json::parsing::ser(lispobj) {
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
    let mut new_list = LispObject::cons(emacs::globals::Qnil, emacs::globals::Qnil);
    for i in 0..len {
        let arg = args.get(i);
        if arg.is_object() {
            let lispobj = unproxy!(scope, arg.to_object(scope).unwrap());
            new_list = LispObject::cons(lispobj, new_list);
        }
    }

    unsafe { emacs::bindings::globals.Vjs_retain_map = new_list };
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

pub fn is_reverse_proxy(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let mut is_reverse_proxy = false;
    if args.get(0).is_object() {
        let arg = args.get(0).to_object(scope).unwrap();
        let lisp = unproxy!(scope, arg);
        if let Some(cons) = lisp.as_cons() {
            is_reverse_proxy = cons.car().eq(emacs::globals::Qjs_proxy);
        }
    }

    let boolean = v8::Boolean::new(scope, is_reverse_proxy);
    let r = v8::Local::<v8::Value>::try_from(boolean).unwrap();
    retval.set(r);
}

pub fn make_reverse_proxy(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let idx = args
        .get(0)
        .to_number(scope)
        .unwrap()
        .integer_value(scope)
        .unwrap();

    let finalizer = unsafe {
        let list = list!(
            emacs::globals::Qjs__clear_r,
            emacs::bindings::make_fixnum(idx.into())
        );
        let lambda_list = list!(emacs::globals::Qlambda, emacs::globals::Qnil, list);
        emacs::bindings::Fmake_finalizer(lambda_list)
    };

    let num = LispObject::from_fixnum(idx);
    let rp = make_reverse_proxy!(scope, LispObject::cons(finalizer, num));
    let r = v8::Local::<v8::Value>::try_from(rp).unwrap();
    retval.set(r);
}

pub fn unreverse_proxy(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let obj = args.get(0).to_object(scope).unwrap();
    let maybe_cons = unproxy!(scope, obj);
    if let Some(cons) = maybe_cons.as_cons() {
        if let Some(inner) = cons.cdr().as_cons() {
            if let Some(value) = inner.cdr().as_fixnum() {
                let r =
                    v8::Local::<v8::Value>::try_from(v8::Number::new(scope, value as f64)).unwrap();
                retval.set(r);
            }
        }
    }
}

unsafe extern "C" fn lisp_springboard(arg1: *mut ::libc::c_void) -> LispObject {
    let mut lisp_args: Vec<LispObject> = *Box::from_raw(arg1 as *mut Vec<LispObject>);
    Ffuncall(lisp_args.len().try_into().unwrap(), lisp_args.as_mut_ptr())
}

unsafe extern "C" fn lisp_handler(
    _arg1: emacs::bindings::nonlocal_exit::Type,
    arg2: LispObject,
) -> LispObject {
    LispObject::cons(emacs::globals::Qjs_lisp_error, arg2)
}

pub fn lisp_invoke(
    mut scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let mut lisp_args = vec![];
    let len = args.length();
    let lispobj = unproxy!(scope, args.get(0).to_object(scope).unwrap());
    lisp_args.push(lispobj);

    for i in 1..len {
        let arg = args.get(i);

        if arg.is_string() {
            let a = arg.to_string(scope).unwrap().to_rust_string_lossy(scope);

            let deser_result = lsp_json::parsing::deser(&a, None);
            match deser_result {
                Ok(deser) => {
                    lisp_args.push(deser);
                }
                Err(e) => {
                    throw_exception_with_error(scope, e);
                    return;
                }
            }
        } else if arg.is_object() {
            let lispobj = unproxy!(scope, arg.to_object(scope).unwrap());
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
    let current = EmacsMainJsRuntime::push_stack(scope);
    let boxed = Box::new(lisp_args);
    let raw_ptr = Box::into_raw(boxed);
    let results = unsafe {
        emacs::bindings::internal_catch_all(
            Some(lisp_springboard),
            raw_ptr as *mut ::libc::c_void,
            Some(lisp_handler),
        )
    };

    scope = EmacsMainJsRuntime::restore_stack(current);

    if results.is_cons() {
        let cons: LispCons = results.into();
        if cons.car() == emacs::globals::Qjs_lisp_error {
            // Lisp has thrown, so we want to throw a JS exception.
            let lisp_error_string = unsafe { emacs::bindings::Ferror_message_string(cons.cdr()) };
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
        emacs::bindings::STRINGP(results)
            || emacs::bindings::FIXNUMP(results)
            || emacs::bindings::FLOATP(results)
            || results == emacs::globals::Qnil
            || results == emacs::globals::Qt
    };
    if is_primative {
        if let Ok(json) = lsp_json::parsing::ser(results) {
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
            emacs::globals::QCallow_net => {
                if value == emacs::globals::Qnil {
                    permissions.net.revoke::<&str>(None);
                }
            }
            emacs::globals::QCallow_read => {
                if value == emacs::globals::Qnil {
                    permissions.read.revoke(None);
                }
            }
            emacs::globals::QCallow_write => {
                if value == emacs::globals::Qnil {
                    permissions.write.revoke(None);
                }
            }
            emacs::globals::QCallow_run => {
                if value == emacs::globals::Qnil {
                    permissions.run.revoke(None);
                }
            }
            emacs::globals::QCjs_tick_rate => unsafe {
                if emacs::bindings::FLOATP(value) {
                    options.tick_rate = emacs::bindings::XFLOAT_DATA(value);
                }
            },
            emacs::globals::QCjs_error_handler => {
                options.error_handler = value;
            }
            emacs::globals::QCinspect => {
                if value.is_string() {
                    options.inspect = Some(value.as_string().unwrap().to_utf8());
                } else if value == emacs::globals::Qt {
                    options.inspect = Some(DEFAULT_ADDR.to_string());
                }
            }
            emacs::globals::QCinspect_brk => {
                if value.is_string() {
                    options.inspect_brk = Some(value.as_string().unwrap().to_utf8());
                } else if value == emacs::globals::Qt {
                    options.inspect_brk = Some(DEFAULT_ADDR.to_string());
                }
            }
            emacs::globals::QCuse_color => {
                if value.is_t() {
                    options.use_color = true;
                }
            }
            emacs::globals::QCts_config => {
                let sref: LispStringRef = value.into();
                let rstring = sref.to_utf8();
                options.ts_config = Some(rstring);
            }
            emacs::globals::QCno_check => {
                if value.is_t() {
                    options.no_check = true;
                }
            }
            emacs::globals::QCno_remote => {
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
fn destroy_worker_on_promise_rejection(e: &EmacsJsError) {
    let is_within_toplevel = EmacsMainJsRuntime::is_within_toplevel_module();
    if is_within_toplevel && e.to_string().starts_with("Uncaught (in promise)") {
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
///
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
    let is_typescript =
        args.len() == 3 && args[1] == emacs::globals::QCtypescript && args[2] == emacs::globals::Qt;

    run_module(&name, Some(string), &ops, is_typescript)
}

/// Evaluates JS in the global context and returns the value
/// of the latest expression with in the statement. This is
/// a wrapper around JavaScript's global `eval` function,
/// and follows the same rules.
///
/// Using global `eval` does not support ES6 import
/// statements or top-level await
#[lisp_fn(intspec = "MEval JS: ")]
pub fn eval_js_literally(js: LispStringRef) -> LispObject {
    let ops = EmacsMainJsRuntime::get_options();
    js_init_sys("init.js", &ops).unwrap_or_else(|e| {
        error!("JS Failed to initialize with error: {}", e);
    });

    let result = execute_with_current_scope(move |scope| eval_literally_inner(scope, js))
        .unwrap_or_else(|e| handle_error_inner_invokation(e));
    tick_and_schedule_if_required();
    result
}

/// Evaluate the contents of BUFFER as JavaScript
/// in the global context and returns the value
/// of the latest expression with in the statement. This is
/// a wrapper around JavaScript's global `eval` function,
/// and follows the same rules.
///
/// Using global `eval` does not support ES6 import
/// statements or top-level await
#[lisp_fn(min = "0", intspec = "")]
pub fn eval_js_buffer_literally(buffer: LispObject) -> LispObject {
    let lisp_string = get_buffer_contents(buffer);
    eval_js_literally(lisp_string.into())
}

/// Evaluate the contents of REGION as JavaScript
/// in the global context and returns the value
/// of the latest expression with in the statement. This is
/// a wrapper around JavaScript's global `eval` function,
/// and follows the same rules.
///
/// Using global `eval` does not support ES6 import
/// statements or top-level await
#[lisp_fn(intspec = "r")]
pub fn eval_js_region_literally(start: LispObject, end: LispObject) -> LispObject {
    let lisp_string = get_region(start, end);
    eval_js_literally(lisp_string.into())
}

/// Evaluate JS and print value in the echo area.
///
/// Similar to eval-expression, except for JavaScript. This
/// function does not accept TypeScript.
///
/// When called interactively, read an Emacs Lisp expression and
/// evaluate it.  Value is also consed on to front of the variable
/// ‘values’.  Optional argument INSERT-VALUE non-nil (interactively,
/// with a non ‘-’ prefix argument) means insert the result into the
/// current buffer instead of printing it in the echo area.
///
/// Normally, this function truncates long output according to the
/// value of the variables ‘eval-expression-print-length’ and
/// ‘eval-expression-print-level’.  When NO-TRUNCATE is
/// non-nil (interactively, with a prefix argument of zero), however,
/// there is no such truncation.
///
/// Runs the hook ‘eval-expression-minibuffer-setup-hook’ on entering the
/// minibuffer.
#[lisp_fn(min = "1", intspec = "MEval JS: ")]
pub fn eval_js_expression(args: &[LispObject]) -> LispObject {
    let js: LispStringRef = args[0].into();
    let result = eval_js_literally(js);
    let mut call = vec![emacs::globals::Qeval_expression, result];
    for i in 1..args.len() {
        call.push(args[i]);
    }

    unsafe { Ffuncall(call.len().try_into().unwrap(), call.as_mut_ptr()) }
}

fn eval_literally_inner(
    scope: &mut v8::HandleScope,
    js: LispStringRef,
) -> EmacsJsResult<LispObject> {
    let context = scope.get_current_context();
    let global = context.global(scope);

    let name = v8::String::new(scope, "__eval").unwrap();
    let fnc: v8::Local<v8::Function> = global.get(scope, name.into()).unwrap().try_into().unwrap();

    let eval_data = js.to_utf8();
    let arg0 =
        v8::Local::<v8::Value>::try_from(v8::String::new(scope, &eval_data).unwrap()).unwrap();
    let v8_args = vec![arg0];
    execute_function_may_throw(scope, &fnc, &v8_args)
}

/// Reads and evaluates FILENAME as a JavaScript module on
/// the main emacs thread.
///
/// If the file does not end in '.js', it will be evaluated as TypeScript.
/// If :typescript t is passed, the file will be evaluated at TypeScript.
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
        && args[1] == emacs::globals::QCtypescript
        && args[2] == emacs::globals::Qt)
        || is_typescript(&module);

    // This is a hack to allow for our behavior of
    // executing a module multiple times.
    let import = unique_module_import!(module);
    module = unique_module!("./$import${}{}.ts");
    run_module(&module, Some(import), &ops, is_typescript)
}

fn get_buffer_contents(mut buffer: LispObject) -> LispObject {
    if buffer.is_nil() {
        buffer = unsafe { emacs::bindings::Fcurrent_buffer() };
    }

    unsafe {
        let current = emacs::bindings::Fcurrent_buffer();
        emacs::bindings::Fset_buffer(buffer);
        let lstring = emacs::bindings::Fbuffer_string();
        emacs::bindings::Fset_buffer(current);
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
        emacs::globals::QCtypescript,
        emacs::globals::Qt,
    ])
}

fn get_region(start: LispObject, end: LispObject) -> LispObject {
    let saved = unsafe { emacs::bindings::save_restriction_save() };
    unsafe {
        emacs::bindings::Fnarrow_to_region(start, end);
        let lstring = emacs::bindings::Fbuffer_string();
        emacs::bindings::save_restriction_restore(saved);
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
        emacs::globals::QCtypescript,
        emacs::globals::Qt,
    ])
}

/// Initalizes the JavaScript runtime. If this function is not
/// called prior to eval-js*, the runtime will be lazily initialized
/// js-initialize takes arguments that allow the JavaScript runtime
/// to be customized:
///
/// The following flags will be set ONCE upon JS initialization.
/// In order to change these flags, you will need to call
/// 'js-cleanup', and then call 'js-initialize'.
///
/// :allow-net nil - Prevents JS from accessing the network
///
/// :allow-write nil - Prevents JS from writing to the file system
///
/// :allow-read nil - Prevents JS from reading the file system
///
/// :allow-run nil - Prevents JS from executing sub-processes
///
/// :use-color - Will print JS error messages in color. Defaults to
/// off due to formatting issues with JS errors invoked with (error ...)
///
/// :inspect "IP:PORT" - Enables the JavaScript debugger, listening with
/// address IP:PORT. Will allow attachment of JavaScript debugger. If passed
/// t, will default to 127.0.0.1:9229
///
/// :inspect-brk "IP:PORT" - Like :inspect, but instead it will break on
/// the first line of JS evaluated to allow the user to connect to the debugger
/// prior to their code being executed.
///
/// :js-error-handler 'function - A function to call if a JS error occures, including
/// TypeScript compile errors. If not specified, will default to (error ...)
///
/// :ts-config PATH - Specifies the file path to your custom tsconfig json file
/// see https://www.typescriptlang.org/docs/handbook/tsconfig-json.html
///
/// :no-check t - disables TypeScript type checking. Can be used to gain performance
/// when the user does not want or require typechecking
///
/// :no-remote t - disables the import of remote files via import statements.
/// This option still allows network options via calls to fetch(...)
///
/// The following flags will be changed upon a call to 'js-initialize',
/// even if the JS environment has already been initialized.
///
/// :js-tick-rate - Defaults to 0.25. This is the interval that js
/// will evaluate if there are any resolved pending async operations
/// and execute callbacks.
#[lisp_fn]
pub fn js_initialize(args: &[LispObject]) -> LispObject {
    let ops = permissions_from_args(args);
    EmacsMainJsRuntime::set_options(ops.clone());
    js_init_sys("init.js", &ops)
        .map(|_| emacs::globals::Qt)
        .unwrap_or_else(|e| {
            error!("JS Failed to initialize with error: {}", e);
        })
}

fn js_init_sys(filename: &str, js_options: &EmacsJsOptions) -> EmacsJsResult<()> {
    init_tokio()?;
    init_worker(filename, js_options)?;
    Ok(())
}

/// Destroys the current JavaScript environment. The JavaScript environment will be
/// reinitalized upon the next call to eval-js*, or to js-initialize
#[lisp_fn]
pub fn js_cleanup() -> LispObject {
    EmacsMainJsRuntime::destroy_worker();
    emacs::globals::Qnil
}

fn js_reenter_inner(scope: &mut v8::HandleScope, args: &[LispObject]) -> EmacsJsResult<LispObject> {
    let index = args[0];

    if !unsafe { emacs::bindings::INTEGERP(index) } {
        error!("Failed to provide proper index to js--reenter");
    }

    let value = unsafe {
        emacs::bindings::check_integer_range(
            index,
            emacs::bindings::intmax_t::MIN,
            emacs::bindings::intmax_t::MAX,
        )
    };

    let context = scope.get_current_context();
    let global = context.global(scope);

    let name = v8::String::new(scope, "__invoke").unwrap();
    let fnc: v8::Local<v8::Function> = global.get(scope, name.into()).unwrap().try_into().unwrap();

    let arg0 = v8::Local::<v8::Value>::try_from(v8::Number::new(scope, value as f64)).unwrap();
    let mut v8_args = vec![arg0];

    if args.len() > 2 {
        let cons: LispCons = args[2].into();
        cons.iter_cars(LispConsEndChecks::on, LispConsCircularChecks::on)
            .for_each(|a| {
                let is_primative = unsafe {
                    emacs::bindings::STRINGP(a)
                        || emacs::bindings::FIXNUMP(a)
                        || emacs::bindings::FLOATP(a)
                        || a == emacs::globals::Qnil
                        || a == emacs::globals::Qt
                };
                if is_primative {
                    if let Ok(json) = lsp_json::parsing::ser(a) {
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

    execute_function_may_throw(scope, &fnc, &mut v8_args)
}

fn execute_function_may_throw(
    scope: &mut v8::HandleScope,
    fnc: &v8::Local<v8::Function>,
    v8_args: &Vec<v8::Local<v8::Value>>,
) -> EmacsJsResult<LispObject> {
    let mut retval = emacs::globals::Qnil;
    // A try catch scope counts as a cope that needs to be placed
    // on the handle stack.

    let tc_scope = &mut v8::TryCatch::new(scope);
    let recv = v8::Local::<v8::Value>::try_from(v8::String::new(tc_scope, "lisp_invoke").unwrap())
        .unwrap();
    let current = EmacsMainJsRuntime::push_stack(tc_scope);
    if let Some(result) = fnc.call(tc_scope, recv, v8_args.as_slice()) {
        if result.is_string() {
            let a = result
                .to_string(tc_scope)
                .unwrap()
                .to_rust_string_lossy(tc_scope);

            retval = lsp_json::parsing::deser(&a, None)?;
        } else if result.is_object() {
            retval = unproxy!(tc_scope, result.to_object(tc_scope).unwrap());
        }
    } else {
        // From https://github.com/denoland/deno/core/runtime.js
        // Credit to deno authors
        let mut exception = tc_scope.exception().unwrap();
        let is_terminating_exception = tc_scope.is_execution_terminating();

        if is_terminating_exception {
            tc_scope.cancel_terminate_execution();

            if exception.is_null_or_undefined() {
                let message = v8::String::new(tc_scope, "execution terminated").unwrap();
                exception = v8::Exception::error(tc_scope, message);
            }
        }

        let v8_exception = deno_core::error::JsError::from_v8_exception(tc_scope, exception);
        EmacsMainJsRuntime::restore_stack(current);
        return Err(v8_exception.into());
    }

    EmacsMainJsRuntime::restore_stack(current);
    Ok(retval)
}

fn tick_and_schedule_if_required() {
    if !EmacsMainJsRuntime::is_within_runtime() && !EmacsMainJsRuntime::get_tick_scheduled() {
        js_tick_event_loop(emacs::globals::Qnil);
    }
}

fn handle_error_inner_invokation(e: EmacsJsError) -> LispObject {
    if !EmacsMainJsRuntime::is_within_runtime() {
        let js_options = EmacsMainJsRuntime::get_options();
        handle_error(e, js_options.error_handler)
    } else {
        // If we are within the runtime, we want to unwind back up to
        // the next error handler. This would imply that there is a funcall
        // above us that called back into JS. If we were to just call the error handler,
        // we would be returning the error handlers value back UP the stack, which would
        // lead to undesirable behavior.
        error!("{}", e.to_string())
    }
}

#[lisp_fn(min = "1")]
pub fn js__reenter(args: &[LispObject]) -> LispObject {
    let result = execute_with_current_scope(move |scope| js_reenter_inner(scope, args))
        .unwrap_or_else(|e| handle_error_inner_invokation(e));
    tick_and_schedule_if_required();
    result
}

fn js_clear_internal(scope: &mut v8::HandleScope, idx: LispObject) {
    js_clear_internal_impl(scope, idx, "__clear");
}

fn js_clear_r_internal(scope: &mut v8::HandleScope, idx: LispObject) {
    js_clear_internal_impl(scope, idx, "__clear_r");
}

fn js_clear_internal_impl(scope: &mut v8::HandleScope, idx: LispObject, func: &str) {
    let value = unsafe {
        emacs::bindings::check_integer_range(
            idx,
            emacs::bindings::intmax_t::MIN,
            emacs::bindings::intmax_t::MAX,
        )
    };

    let context = scope.get_current_context();
    let global = context.global(scope);

    let name = v8::String::new(scope, func).unwrap();
    let fnc: v8::Local<v8::Function> = global.get(scope, name.into()).unwrap().try_into().unwrap();
    let recv =
        v8::Local::<v8::Value>::try_from(v8::String::new(scope, "lisp_invoke").unwrap()).unwrap();
    let arg0 = v8::Local::<v8::Value>::try_from(v8::Number::new(scope, value as f64)).unwrap();
    let v8_args = vec![arg0];
    fnc.call(scope, recv, v8_args.as_slice()).unwrap();
}

fn execute_with_current_scope<F, R: Sized>(f: F) -> R
where
    F: Fn(&mut v8::HandleScope) -> R,
{
    let result;
    if !EmacsMainJsRuntime::is_within_runtime() {
        result = block_on(async move {
            let mut worker_handle = EmacsMainJsRuntime::get_deno_worker();
            let worker = worker_handle.as_mut_ref();
            let runtime = &mut worker.js_runtime;
            let context = runtime.global_context();
            let scope = &mut v8::HandleScope::with_context(runtime.v8_isolate(), context);
            let retval = f(scope);

            Ok(retval)
        })
        .unwrap(); // Safe due to the fact we set this to Ok
    } else {
        let scope = EmacsMainJsRuntime::peek_stack();
        result = f(scope);
        EmacsMainJsRuntime::restore_stack(Some(scope));
    }

    result
}

/// Internal function used for cleanup. Do not call directly.
#[lisp_fn]
pub fn js__clear(idx: LispObject) {
    execute_with_current_scope(move |scope| js_clear_internal(scope, idx));
}

/// Internal function used for cleanup. Do not call directly.
#[lisp_fn]
pub fn js__clear_r(idx: LispObject) {
    execute_with_current_scope(move |scope| js_clear_r_internal(scope, idx));
}

fn block_on<R: Sized, T: Sized + std::future::Future<Output = EmacsJsResult<R>>>(
    fnc: T,
) -> EmacsJsResult<R> {
    if EmacsMainJsRuntime::is_within_runtime() {
        Err(deno_core::error::generic_error(
            "Attempted to execute javascript from lisp within the javascript context.",
        ))
    } else {
        let mut handle = EmacsMainJsRuntime::get_tokio_handle();
        let handle_ref = handle.as_mut_ref();
        EmacsMainJsRuntime::enter_runtime();
        let result = handle_ref.block_on(fnc);
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

/// Internal function called by 'garbage-collect'. Do not call directly.
#[lisp_fn]
pub fn js__sweep() -> LispObject {
    if EmacsMainJsRuntime::is_main_worker_active() {
        execute_with_current_scope(|scope| js_sweep_inner(scope));
    }

    emacs::globals::Qnil
}

fn tick_js() -> EmacsJsResult<bool> {
    let mut is_complete = false;
    let is_complete_ref = &mut is_complete;
    block_on(async move {
        futures::future::poll_fn(|cx| {
            let mut worker_handle = EmacsMainJsRuntime::get_deno_worker();
            let w = worker_handle.as_mut_ref();
            let polled = w.poll_event_loop(cx);
            match polled {
                std::task::Poll::Ready(r) => {
                    *is_complete_ref = true;
                    r?
                }
                std::task::Poll::Pending => {}
            }

            std::task::Poll::Ready(Ok(()))
        })
        .await
    })
    .map(move |_| is_complete)
}

fn init_tokio() -> EmacsJsResult<()> {
    if !EmacsMainJsRuntime::is_tokio_active()
    // Needed in the case that the tokio runtime is being taken
    // for completing a JS operation
	&& !EmacsMainJsRuntime::is_within_runtime()
    {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .enable_time()
            .worker_threads(2)
            .max_blocking_threads(32)
            .build()?;

        EmacsMainJsRuntime::set_tokio_runtime(runtime);
    }

    Ok(())
}

pub(crate) fn v8_bind_lisp_funcs(
    worker: &mut deno_runtime::worker::MainWorker,
) -> EmacsJsResult<()> {
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
        bind_global_fn!(scope, global, lisp_invoke);
        bind_global_fn!(scope, global, is_proxy);
        bind_global_fn!(scope, global, finalize);
        bind_global_fn!(scope, global, lisp_json);
        bind_global_fn!(scope, global, lisp_intern);
        bind_global_fn!(scope, global, lisp_make_finalizer);
        bind_global_fn!(scope, global, lisp_string);
        bind_global_fn!(scope, global, lisp_fixnum);
        bind_global_fn!(scope, global, lisp_float);
        bind_global_fn!(scope, global, lisp_make_lambda);
        bind_global_fn!(scope, global, lisp_list);
        bind_global_fn!(scope, global, json_lisp);
        bind_global_fn!(scope, global, is_reverse_proxy);
        bind_global_fn!(scope, global, make_reverse_proxy);
        bind_global_fn!(scope, global, unreverse_proxy);
    }
    {
        runtime.execute("prelim.js", include_str!("prelim.js"))?
    }

    Ok(())
}

fn init_worker(filepath: &str, js_options: &EmacsJsOptions) -> EmacsJsResult<()> {
    if EmacsMainJsRuntime::is_main_worker_active() || EmacsMainJsRuntime::is_within_runtime() {
        return Ok(());
    }

    let mut handle = EmacsMainJsRuntime::get_tokio_handle();
    let runtime = handle.as_mut_ref();
    let main_module = deno_core::resolve_url_or_path(filepath)?;
    let permissions = js_options.ops.as_ref().unwrap().clone();
    let inspect = if let Some(i) = &js_options.inspect {
        Some(i.parse::<std::net::SocketAddr>()?)
    } else {
        None
    };

    let inspect_brk = if let Some(i) = &js_options.inspect_brk {
        Some(i.parse::<std::net::SocketAddr>()?)
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

    let program_fut = futures::executor::block_on(deno::program_state::ProgramState::build(flags));
    let program = program_fut?;
    EmacsMainJsRuntime::set_program_state(program.clone());
    let mut worker = deno::create_main_worker(&program, main_module.clone(), permissions);
    let result: EmacsJsResult<deno_runtime::worker::MainWorker> = runtime.block_on(async move {
        v8_bind_lisp_funcs(&mut worker)?;
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
) -> EmacsJsResult<LispObject> {
    js_init_sys(filepath, js_options)?;

    block_on(async move {
        let mut worker_handle = EmacsMainJsRuntime::get_deno_worker();
        let w = worker_handle.as_mut_ref();
        let main_module = deno_core::resolve_url_or_path(filepath)?;

        if let Some(js) = additional_js {
            let program = EmacsMainJsRuntime::get_program_state();
            // We are inserting a fake file into the file cache in order to execute
            // our module.
            let file = deno::file_fetcher::File {
                local: main_module.clone().to_file_path().unwrap(),
                maybe_types: None,
                media_type: if as_typescript {
                    deno::media_type::MediaType::TypeScript
                } else {
                    deno::media_type::MediaType::JavaScript
                },
                source: js,
                specifier: main_module.clone(),
            };

            program.file_fetcher.insert_cached(file);
        }

        w.execute_module(&main_module).await?;
        Ok(())
    })?;

    schedule_tick();
    Ok(emacs::globals::Qnil)
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

fn handle_error(e: EmacsJsError, handler: LispObject) -> LispObject {
    let err_string = e.to_string();
    if handler.is_nil() {
        error!(err_string);
    } else {
        unsafe {
            let len = err_string.len();
            let cstr = CString::new(err_string).expect("Failed to allocate CString");
            let lstring =
                emacs::bindings::make_string_from_utf8(cstr.as_ptr(), len.try_into().unwrap());
            let mut args = vec![handler, lstring];
            Ffuncall(args.len().try_into().unwrap(), args.as_mut_ptr())
        }
    }
}

/// Gets the current tick rate of JavaScript.
#[lisp_fn]
pub fn js_get_tick_rate() -> LispObject {
    let options = EmacsMainJsRuntime::get_options();
    unsafe { emacs::bindings::make_float(options.tick_rate) }
}

/// Sets F to be the current js tick rate. Every F seconds, javascript
/// will attempt to evaluate the JS event loop. It will advance the
/// event loop LOOPS_PER_TICK iterations.
#[lisp_fn(min = "1")]
pub fn js_set_tick_rate(f: LispObject, loops_per_tick: LispObject) {
    let mut options = EmacsMainJsRuntime::get_options();

    unsafe {
        emacs::bindings::CHECK_NUMBER(f);
        options.tick_rate = emacs::bindings::XFLOATINT(f);

        if loops_per_tick.is_not_nil() {
            options.loops_per_tick = loops_per_tick.as_natnum_or_error();
        }

        EmacsMainJsRuntime::set_options(options);
    }
}

fn schedule_tick() {
    // We do not want to schedule a tick if one is already scheduled.
    if EmacsMainJsRuntime::get_tick_scheduled() {
        return;
    }

    EmacsMainJsRuntime::set_tick_scheduled(true);
    let js_options = EmacsMainJsRuntime::get_options();
    let rate;
    let repeat;

    let tick_rate = unsafe { emacs::bindings::make_float(js_options.tick_rate) };
    if is_interactive() {
        rate = tick_rate;
        repeat = emacs::globals::Qnil;
    } else {
        rate = emacs::globals::Qt;
        repeat = tick_rate;
    }

    unsafe {
        let mut args = vec![
            emacs::globals::Qrun_with_timer,
            rate,
            repeat,
            emacs::globals::Qjs_tick_event_loop,
        ];
        emacs::bindings::Ffuncall(args.len().try_into().unwrap(), args.as_mut_ptr());
    }
}

fn tick_and_handle_error(handler: LispObject) -> bool {
    let error_handler = if handler.is_nil() {
        EmacsMainJsRuntime::get_options().error_handler
    } else {
        handler
    };

    tick_js()
        // We do NOT want to destroy the MainWorker if we error here.
        // We can still use this isolate for future promise resolutions
        // instead, just pass to the error handler.
        .unwrap_or_else(|e| {
            // If handler is nil, we need to manually
            // schedule tick since handle_error isn't
            // going to return.
            if handler.is_nil() {
                schedule_tick();
            }

            handle_error(e, error_handler);
            false
        })
}

/// Move the JavaScript event loop forward with arg HANDLER
///
/// HANDLER is a custom error handler that will be invoked
/// if the event loop's javascript throws. Default to invoking
/// 'error' if not specified. See 'js-initialize'.
/// This will be called via a timer. Once this function is called,
/// we will advance the loop by a certain number of iterations (default 1000).
/// This function is safe to call directly, and can even be set up to be
/// invoked regularly with custom logic.
#[lisp_fn(min = "0")]
pub fn js_tick_event_loop(handler: LispObject) -> LispObject {
    // Consume the tick for this event loop call.
    // Unless we are non-interactive
    if is_interactive() {
        EmacsMainJsRuntime::set_tick_scheduled(false);
    }

    if !EmacsMainJsRuntime::is_main_worker_active() {
        return emacs::globals::Qnil;
    }

    // If we are within the runtime, we don't want to attempt to
    // call execute, as we will error, and there really isn't anything
    // anyone can do about it. Just defer the event loop until
    // we are out of the runtime.
    if EmacsMainJsRuntime::is_within_runtime() {
        schedule_tick();
        return emacs::globals::Qnil;
    }

    let num_loops = EmacsMainJsRuntime::get_loops_per_tick();
    let mut is_complete = false;
    for _ in 0..num_loops {
        is_complete = tick_and_handle_error(handler);
        if is_complete {
            break;
        }
    }

    if !is_complete {
        schedule_tick();
    }

    emacs::globals::Qnil
}

// We overwrite certain subcommands to allow interfacing with emacs-lisp
// All other subcommands will use deno's default implementation
fn get_subcommand(flags: deno::flags::Flags) -> Pin<Box<dyn Future<Output = EmacsJsResult<()>>>> {
    match flags.clone().subcommand {
        deno::flags::DenoSubcommand::Eval {
            print,
            code,
	    ext,
        } => crate::subcommands::eval_command(flags, code, ext, print).boxed_local(),
        deno::flags::DenoSubcommand::Run { script } => {
            crate::subcommands::run_command(flags, script).boxed_local()
        }
        deno::flags::DenoSubcommand::Repl => crate::subcommands::run_repl(flags).boxed_local(),
        deno::flags::DenoSubcommand::Test {
            no_run,
            fail_fast,
            quiet,
            include,
            allow_none,
            filter,
        } => crate::subcommands::test_command(
            flags, include, no_run, fail_fast, quiet, allow_none, filter,
        )
            .boxed_local(),
	deno::flags::DenoSubcommand::Info { json, .. } => async move {
	    if is_interactive() && json && !flags.unstable {
		Err(deno_core::error::generic_error(
                    "--unstable is required for this command",
                ))
	    } else {
                deno::get_subcommand(flags).await
	    }
	}.boxed_local(),
        deno::flags::DenoSubcommand::Compile { .. } => async {
	    if is_interactive() && !flags.unstable {
                Err(deno_core::error::generic_error(
                    "--unstable is required for this command",
                ))
            } else {
                deno::get_subcommand(flags).await
            }
	}.boxed_local(),
        deno::flags::DenoSubcommand::Lint { .. } => async {
            if is_interactive() {
                Err(deno_core::error::generic_error(
                    "lint is not supported in interactive mode. Lint files with emacs as a subprocess using emacs --batch --eval '(deno \"lint\" \"--unstable\")'",
                ))
            } else {
                deno::get_subcommand(flags).await
            }
        }
        .boxed_local(),
        // (DDS) We don't want upgrade to be run from emacs
        // since it wouldnt do what the user expects
        // instead, we will just throw an error
        // @TODO it would be nice if this actually
        // upgraded emacs-ng instead
        deno::flags::DenoSubcommand::Upgrade { .. } => async {
            Err(deno_core::error::generic_error(
                "(deno upgrade) is unsupported in emacs-ng at this time.",
            ))
        }
        .boxed_local(),
        _ => deno::get_subcommand(flags),
    }
}

/// Usage: (deno CMD &REST ARGS)
///
/// Invokes a deno command using emacs-ng. This behavior mirrors as if you
/// ran a deno command from the command line, except that lisp
/// functions are available
///
/// Unlike normal JavaScript run in emacs, using this command
/// respects deno's permission model. You will need to pass
/// --allow-read, --allow-write, --allow-net or --allow-run
///
/// This command spawns a new JavaScript environment to
/// simulate how deno cli works. HOWEVER, Lisp variables
/// are shared. `deno` is a blocking call, and so interacting
/// with lisp does not need a lock.
///
/// Using this command is using emacs AS deno, and does not change
/// how deno handles Input/Output. This means that deno will write
/// to stdout and recieve input via stdin as it normally would.
/// The primary use case of this function is be run from the
/// command line, however it can be used while running emacs
///
/// The only command not fully supported is (deno "upgrade")
///
/// (deno "lint") can only be run while emacs is in batch mode via the command
/// line: emacs --batch --eval '(deno "lint" "--unstable")'
///
/// This can be combined with running emacs-ng in batch mode to fully mirror deno
/// functionality. I.e. `emacs --batch --eval '(deno "repl")'
///
/// This function is safe to execute from a lisp thread if you want to make
/// the operation non-blocking. (make-thread (lambda () (deno "fmt")))
///
/// Examples:
/// (deno "fmt") ; Will format files in the current directory as if you ran
///              ; deno fmt from the command line.
///
/// (deno "run" "--allow-read" "my-file.ts") ; Runs a typescript file named
///                                          ; my-file.ts, allowing reads
///
#[lisp_fn(min = "1")]
pub fn deno(cmd_args: &[LispObject]) {
    let mut args = vec!["deno".to_string()];
    for i in 0..cmd_args.len() {
        let stringref: LispStringRef = cmd_args[i].into();
        let string = stringref.to_utf8();
        args.push(string);
    }

    let flags = deno::flags::flags_from_vec(args.clone()).unwrap_or_else(|e| {
        error!("Error in parsing flags: {}", e);
    });
    let fut = get_subcommand(flags);
    init_tokio().unwrap_or_else(|e| {
        error!("Unable to initialize tokio runtime: {}", e);
    });

    block_on(async move { fut.await }).unwrap_or_else(|e| {
        error!("Error in deno command '{}': {}", args.join(" "), e);
    });
}

// Do NOT call this function, it is just used for macro purposes to
// generate variables. The user should NOT have direct access to
// 'js-retain-map' from the scripting engine.
#[allow(dead_code)]
fn init_syms() {
    defvar_lisp!(Vjs_retain_map, "js-retain-map", emacs::globals::Qnil);

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
    def_lisp_sym!(Qjs__clear_r, "js--clear-r");
    def_lisp_sym!(Qlambda, "lambda");
    def_lisp_sym!(Qjs__reenter, "js--reenter");
    def_lisp_sym!(QCinspect, ":inspect");
    def_lisp_sym!(QCinspect_brk, ":inspect-brk");
    def_lisp_sym!(QCuse_color, ":use-color");

    def_lisp_sym!(QCts_config, ":ts-config");
    def_lisp_sym!(QCno_check, ":no-check");
    def_lisp_sym!(QCno_remote, ":no-remote");
    def_lisp_sym!(QCloops_per_tick, ":loops-per-tick");

    def_lisp_sym!(Qrun_with_timer, "run-with-timer");
    def_lisp_sym!(Qjs_tick_event_loop, "js-tick-event-loop");
    def_lisp_sym!(Qeval_expression, "eval-expression");

    def_lisp_sym!(Qjs_proxy, "js-proxy");
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/out/javascript_exports.rs"
));
