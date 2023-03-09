use std::convert::TryInto;
use std::ffi::CString;

use deno::deno_core::anyhow::anyhow;
use v8;
use deno::deno_core as deno_core;

use emacs::bindings::Ffuncall;
use emacs::lisp::LispObject;
use emacs::multibyte::LispStringRef;
use lisp_macros::lisp_fn;

pub type EmacsJsError = deno_core::error::AnyError;
pub type EmacsJsResult<T> = Result<T, EmacsJsError>;

macro_rules! inc_auto_id {
    () => {{
        let mut aid = auto_id.lock().unwrap();
        let new_auto_id = *aid + 1;
        *aid += 1;
        new_auto_id
    }};
}

macro_rules! establish_channel {
    ($chnlvar: expr, $lock: expr) => {
        {
            let mut chan = $lock.lock().unwrap();
            *chan = Some($chnlvar);
        }
    };
}

macro_rules! make_lisp_string {
    ($arg: expr) => {{
        unsafe {
            let len = $arg.len();
            let cstr = CString::new($arg).expect("Failed to allocate CString");
            emacs::bindings::make_string_from_utf8(cstr.as_ptr(), len.try_into().unwrap())
        }
    }}
}

macro_rules! lisp_yield {
    () => {
        unsafe { emacs::bindings::Fthread_yield() };
    };
}


/// Evaluates CODE as JavaScript on the main emacs thread.
///
/// DEPRECATED: Use (js-eval-string &ARGS)
#[lisp_fn(min = "1")]
pub fn eval_js(args: &[LispObject]) -> LispObject {
    js_eval_string(args[0])
}

/// Reads and evaluates FILENAME as JavaScript
#[lisp_fn(min = "1")]
pub fn js_eval_file(args: &[LispObject]) -> LispObject {
    let filename: LispStringRef = args[0].into();
    let module = filename.to_utf8();

    let id = inc_auto_id!();
    {
        let gaurd = main_to_js_lock.lock().unwrap();
        if let Some(chnl) = &*gaurd {
            chnl.send(RequestMsg { id: id, action: Action::ExecuteFile(module) }).expect("Failure to send");
        }
    }
    id.into()
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

/// Evaluate the contents of BUFFER within the active Javascript Runtime.
#[lisp_fn(min = "0", intspec = "")]
pub fn js_eval_buffer(buffer: LispObject) -> LispObject {
    let lisp_string = get_buffer_contents(buffer);
    js_eval_string(lisp_string)
}

#[lisp_fn(min = "0", intspec = "")]
pub fn js_eval_buffer_blocking(buffer: LispObject) -> LispObject {
    let result = js_eval_buffer(buffer);
    js_resolve_blocking(result)
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

/// Evaluate the contents of REGION.
#[lisp_fn(intspec = "r")]
pub fn js_eval_region(start: LispObject, end: LispObject) -> LispObject {
    let lisp_string = get_region(start, end);
    js_eval_string(lisp_string)
}

macro_rules! bind_global_fn {
    ($scope:expr, $global: expr, $fnc:ident) => {{
        let name = v8::String::new($scope, stringify!($fnc)).unwrap();
        let func = v8::Function::new($scope, $fnc).unwrap();
        $global.set($scope, name.into(), func.into());
    }};
}

pub fn send_to_lisp(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) {
    let first_arg = args.get(0);
    if first_arg.is_string() {
        let rust_string = args.get(0).to_string(scope).unwrap().to_rust_string_lossy(scope);
        println!("Sending {} to lisp...", rust_string);
        {
            let lock = js_to_lisp_worker_send_lock.lock().unwrap();
            if let Some(chnl) = &*lock {
                let id = inc_auto_id!();
                chnl.send(RequestMsg {
                    id: id,
                    action: Action::Execute(rust_string),
                }).expect("Failure to send");
                _retval.set_uint32(id as u32);
            }
        }
    }
}

// @TODO this needs proper error signaling.
pub fn recv_from_lisp(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) {
    {
        let lock = lisp_worker_to_js_recv_lock.lock().unwrap();
        if let Some(chnl) = &*lock {
            let recv_msg = chnl.try_recv();
            if let Err(_) = recv_msg {
                return;
            }

            let msg = recv_msg.unwrap();
            let result = msg.res;

            // @TODO if this is an error, throw to JS
            let response = result.map_or_else(
                    |e| e.to_string(),
                    |f| {
                        let Response::Success(success_result) = f;
                        success_result
            });


                let id_v8string = v8::String::new(scope, "id").unwrap();
                let result_v8string = v8::String::new(scope, "result").unwrap();


                let ret_object = v8::Object::new(scope);
                let ret_id = v8::Integer::new(scope, msg.id as i32);
                ret_object.set(scope, id_v8string.into(), ret_id.into());

                let content = v8::String::new(scope, &response).unwrap();
                ret_object.set(scope, result_v8string.into(), content.into());

                println!("{:?}", response);
                _retval.set(ret_object.into());
        }
    }
}

#[derive(Debug)]
enum Action {
    Execute(String),
    ExecuteFile(String),
}

#[derive(Debug)]
enum Response {
    Success(String),
}

#[derive(Debug)]
struct RequestMsg {
    pub id: usize,
    pub action: Action,
}

#[derive(Debug)]
struct ResponseMsg {
    pub id: usize,
    pub res: Result<Response, deno::AnyError>,
}

unsafe impl Send for ResponseMsg {}
unsafe impl Send for RequestMsg {}

lazy_static! {
    static ref main_to_js_lock: std::sync::Mutex<Option<std::sync::mpsc::Sender<RequestMsg>>> = {
        std::sync::Mutex::new(None)
    };
    static ref main_to_js_recv_lock: std::sync::Mutex<Option<std::sync::mpsc::Receiver<RequestMsg>>> = {
        std::sync::Mutex::new(None)
    };
    static ref js_to_main_lock: std::sync::Mutex<Option<std::sync::mpsc::Receiver<ResponseMsg>>> = {
        std::sync::Mutex::new(None)
    };
    static ref js_to_lisp_worker_send_lock: std::sync::Mutex<Option<std::sync::mpsc::Sender<RequestMsg>>> = {
        std::sync::Mutex::new(None)
    };
    static ref js_to_lisp_worker_recv_lock: std::sync::Mutex<Option<std::sync::mpsc::Receiver<RequestMsg>>> = {
        std::sync::Mutex::new(None)
    };
    static ref lisp_worker_to_js_send_lock: std::sync::Mutex<Option<std::sync::mpsc::Sender<ResponseMsg>>> = {
        std::sync::Mutex::new(None)
    };
    static ref lisp_worker_to_js_recv_lock: std::sync::Mutex<Option<std::sync::mpsc::Receiver<ResponseMsg>>> = {
        std::sync::Mutex::new(None)
    };

    static ref auto_id: std::sync::Mutex<usize> = {
        std::sync::Mutex::new(0)
    };

    static ref resolve_map: std::sync::Mutex<std::collections::HashMap<usize, ResponseMsg>> = {
        std::sync::Mutex::new(std::collections::HashMap::new())
    };
}

fn process_response(msg: ResponseMsg) -> LispObject {
    match msg.res {
        Ok(res) => {
            let Response::Success(result) = res;
            let response = lsp_json::parsing::deser(&result, None)
                .map_or_else(|_| emacs::globals::Qnil, |v| v);
            return response;
        }
        Err(e) => error!(e.to_string())
    }
}

#[lisp_fn]
pub fn js_resolve_blocking(id: LispObject) -> LispObject {
    loop {
        let result = js_resolve(id);
        if result != emacs::globals::Qjs_not_ready {
            return result;
        }

        lisp_yield!();
    }

    emacs::globals::Qnil
}

#[lisp_fn]
pub fn js_resolve(id: LispObject) -> LispObject {
    let idx = id.as_natnum_or_error() as usize;
    {
        let mut lock = resolve_map.lock().unwrap();
        if let Some(msg) = lock.remove(&idx) {
            return process_response(msg);
        }
    }

    {
        let recv_gaurd = js_to_main_lock.lock().unwrap();
        if let Some(recv_chnl) = &*recv_gaurd {
            if let Ok(msg) = recv_chnl.try_recv() {
                if msg.id == idx {
                    return process_response(msg);
                } else {
                    let mut lock = resolve_map.lock().unwrap();
                    lock.insert(msg.id, msg);
                }
            }
        }
    }

    emacs::globals::Qjs_not_ready
}

#[lisp_fn]
pub fn js_eval_string(content: LispObject) -> LispObject {
    let js_content: LispStringRef = content.into();
    let js_rust_string = js_content.to_utf8();
    {
        let gaurd = main_to_js_lock.lock().unwrap();
        if let Some(chnl) = &*gaurd {
            let id = inc_auto_id!();
            // @TODO remove unwrap
            chnl.send(RequestMsg { id: id, action: Action::Execute(js_rust_string) }).expect("Failure to send");

            return id.into();
        }
    }

    emacs::globals::Qnil
}

fn make_poll_fut() -> tokio::task::JoinHandle<Option<RequestMsg>> {
    tokio::task::spawn_blocking(|| {
        let lock = main_to_js_recv_lock.lock().unwrap();
        if let Some(chnl) = &*lock {
            chnl.recv().ok()
        } else {
            None
        }
    })
}

/// Initalizes the JavaScript runtime. This function is required
/// prior to calling any js_eval-* functions
#[lisp_fn]
pub fn js_initialize(args: &[LispObject]) -> LispObject {
    std::env::set_var("NO_COLOR", "1");

    // Main to JS (send/recv)
    let (mtjs, mtjr) = std::sync::mpsc::channel::<RequestMsg>();
    // JS to Main (send/recv)
    let (jstms, jstmr) = std::sync::mpsc::channel::<ResponseMsg>();
    // JS to Lisp Worker (send/recv)
    let (jstolws, jstolwr) = std::sync::mpsc::channel::<RequestMsg>();
    // Lisp Worker to JS (send/recv)
    let (lwtojss, lwtojsr) = std::sync::mpsc::channel::<ResponseMsg>();

    establish_channel!(mtjs, main_to_js_lock);
    establish_channel!(mtjr, main_to_js_recv_lock);
    establish_channel!(jstmr, js_to_main_lock);
    establish_channel!(jstolws, js_to_lisp_worker_send_lock);
    establish_channel!(jstolwr, js_to_lisp_worker_recv_lock);
    establish_channel!(lwtojss, lisp_worker_to_js_send_lock);
    establish_channel!(lwtojsr, lisp_worker_to_js_recv_lock);

    call!(emacs::globals::Qjs_init_lisp_thread);

    std::thread::spawn(move || {
        let _result: Result<(), deno::AnyError> = deno::deno_runtime::tokio_util::run_local(async move {
            let flags = deno::args::flags_from_vec(vec!["deno".to_owned()])?;
            let main_module = deno::deno_core::resolve_url_or_path("./$deno$repl.ts").unwrap();
            let ps = deno::proc_state::ProcState::build(flags).await?;
            let perms = deno::deno_runtime::permissions::PermissionsContainer::new(deno::deno_runtime::permissions::Permissions::from_options(&ps.options.permissions_options())?);
            let mut worker = deno::worker::create_main_worker(
                &ps,
                main_module.clone(),
                perms,
            )
            .await?;
            worker.setup_repl().await?;

            let mut main_worker = worker.into_main_worker();
            {
                let runtime = &mut main_worker.js_runtime;
                {
                    let context = runtime.global_context();
                    let scope = &mut v8::HandleScope::with_context(runtime.v8_isolate(), context);
                    let context = scope.get_current_context();
                    let global = context.global(scope);

                    bind_global_fn!(scope, global, send_to_lisp);
                    bind_global_fn!(scope, global, recv_from_lisp);
                }

                {
                    runtime.execute_script("prelim.js", include_str!("prelim.js"))?;
                }
            }

            let mut repl_session = deno::tools::repl::session::ReplSession::initialize(ps.clone(), main_worker).await?;
            let mut line_fut = make_poll_fut();
            let mut poll_worker = true;

            loop {
                tokio::select! {
                  result = &mut line_fut => {
                    let opt = result?;
                    if let Some(msg) = opt {
                        println!("JS Recv'd {:?}", msg);
                        let result = match msg.action {
                            Action::Execute(cmd) => repl_session.evaluate_line_and_get_output(&cmd).await,
                            Action::ExecuteFile(ref filepath) => {
                                let result = tokio::fs::read_to_string(filepath).await;
                                if let Ok(content) = result {
                                    repl_session.evaluate_line_and_get_output(&content).await
                                } else {
                                    deno::tools::repl::session::EvaluationOutput::Error(format!("Failed to find file {}", filepath))
                                }
                            }
                        };
                        println!("Result {}", result);

                        let msg_result = match result {
                            deno::tools::repl::session::EvaluationOutput::Value(s) => Ok(Response::Success(s.to_string())),
                            deno::tools::repl::session::EvaluationOutput::Error(s) => Err(anyhow!(s.to_string())),

                        };

                        jstms.send(ResponseMsg {
                            id: msg.id,
                            res: msg_result,
                        })?;
                    }

                    line_fut = make_poll_fut();
                    poll_worker = true;
                  },
                  _ = repl_session.run_event_loop(), if poll_worker => {
                    poll_worker = false;
                  }
                }
            }
        });

    });

    emacs::globals::Qt
}


#[lisp_fn]
pub fn js_lisp_thread(_args: &[LispObject]) -> LispObject {
    // Read from queue
    loop {
        let result = {
            let lock = js_to_lisp_worker_recv_lock.lock().unwrap();
            if let Some(chnl) = &*lock {
                chnl.try_recv().ok()
            } else {
                None
            }
        };


        if let Some(request) = result {
            println!("Recv'd on lisp {:?}", request);
            let msg = if let Action::Execute(msg) = request.action {
                msg
            } else {
                // @TODO implement
                println!("Unsupported operation -> loading lisp by file...");
                continue;
            };
            let len = msg.len();
            let cstr = CString::new(msg).expect("Failed to allocate CString");
            let lstring =
                unsafe { emacs::bindings::make_string_from_utf8(cstr.as_ptr(), len.try_into().unwrap()) };
            let mut args = vec![
                emacs::globals::Qjs_eval_lisp_string,
                lstring,
            ];
            let result = unsafe { Ffuncall(args.len().try_into().unwrap(), args.as_mut_ptr()) };
            println!("Executed with result...");
            let lisp_result = if let Some(cons) = result.as_cons() {
                let car = cons.car();
                if car == emacs::globals::Qjs_error {
                    println!("Error in lisp...");
                    cons.cdr()
                } else {
                    result
                }
            } else {
                result
            };

            println!("Performing ser...");
            let ser_result = lsp_json::parsing::ser(lisp_result);
            let res = ser_result.map(|s| Response::Success(s))
                                .map_err(|e| anyhow!(e.to_string()));
            // println!("Sending result to js {:?}", result_string);
            let lock = lisp_worker_to_js_send_lock.lock().unwrap();
            // @TODO in error case, send the err enum for res
            if let Some(chnl) = &*lock {
                chnl.send(ResponseMsg {
                    id: request.id,
                    res: res
                }).expect("Failed to send");
            }
        }

        // Ffuncall(args.len, arg2)
        // call!(emacs::globals::Qjs_eval_string, "(setq foo 3)".into()).unwrap();
        // emacs::eval_macros::eval!(`(print "hello")`);
        // std::thread::sleep(std::time::Duration::from_secs(5));
        lisp_yield!();
    }
    emacs::globals::Qnil
}

// Do NOT call this function, it is just used for macro purposes to
// generate variables. The user should NOT have direct access to
// 'js-retain-map' from the scripting engine.
#[allow(dead_code)]
fn init_syms() {
    def_lisp_sym!(Qjs_init_lisp_thread, "js-init-lisp-thread");
    def_lisp_sym!(Qjs_eval_lisp_string, "js-eval-lisp-string");
    def_lisp_sym!(Qjs_not_ready, "js-not-ready");
    def_lisp_sym!(Qjs_error, "js-error");
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/out/javascript_exports.rs"
));
