use std::io::{BufReader, BufWriter, Result};
use std::process::{Child, Command, Stdio};
use std::thread;

use lsp_server::{Message, Notification, Request, RequestId, Response};

use ng_async::ng_async::{to_owned_userdata, EmacsPipe, PipeDataOption, UserData};

use emacs::lisp::LispObject;
use emacs::list::{LispCons, LispConsCircularChecks, LispConsEndChecks};
use emacs::multibyte::LispStringRef;
use lisp_macros::lisp_fn;

use emacs::bindings::{Fplist_get, Fplist_put, Fprocess_plist, Fset_process_plist};

use emacs::globals::QCjson_config;

use lsp_json::parsing::{
    generate_config_from_args, lisp_to_serde, serde_to_lisp, JSONConfiguration,
};

const ID: &str = "id";
const RESULT: &str = "result";
const ERROR: &str = "error";
const MESSAGE: &str = "message";
const PARAMS: &str = "params";
const METHOD: &str = "method";
const DATA: &str = "data";
const CODE: &str = "code";

// Defined by JSON RPC
const PARSE_ERROR: i32 = -32700;

/// Create a 'child process' defined by STRING 'command'
/// 'args' is a list of STRING arguments for the invoked command. Can be NIL
/// handler is the FUNCTION that will be invoked on the result data
/// returned from the process via stdout. The handler should take two
/// arguments, the pipe process and the data. Data will be returned as
/// a 'user-ptr', which should be passed to lsp-handler for further processing.
#[lisp_fn]
pub fn lsp_make_connection(
    command: LispObject,
    args: LispObject,
    handler: LispObject,
) -> LispObject {
    let command_ref: LispStringRef = command.into();
    let command_string = command_ref.to_utf8();
    let (emacs_pipe, proc) = EmacsPipe::with_handler(
        handler,
        PipeDataOption::USER_DATA,
        PipeDataOption::USER_DATA,
    );

    let mut args_vec: Vec<String> = vec![];
    if args.is_not_nil() {
        let list_args: LispCons = args.into();

        list_args
            .iter_cars(LispConsEndChecks::on, LispConsCircularChecks::on)
            .for_each(|x| {
                if let Some(string_ref) = x.as_string() {
                    args_vec.push(string_ref.to_utf8());
                } else {
                    error!("make-lsp-command takes a list of string arguments");
                }
            });
    }

    if let Err(e) = async_create_process(command_string, args_vec, emacs_pipe) {
        error!("Error creating process, reason {:?}", e);
    }

    proc
}

/// Process the result of a lsp-server invoked via make-lsp-connection,
/// and convert it to a lisp object. Data should be a USER-PTR object
/// that was provided by the lsp-servers handler.
#[lisp_fn]
pub fn lsp_handler(proc: LispObject, data: LispObject) -> LispObject {
    let user_data: UserData = to_owned_userdata(data);
    let msg: Message = unsafe { user_data.unpack() };
    let config = &get_process_json_config(proc);
    let result = match msg {
        Message::Request(re) => serde_to_lisp(
            json!({ID: re.id, METHOD: re.method, PARAMS: re.params}),
            config,
        ),
        Message::Response(r) => {
            let response = r.result.unwrap_or(serde_json::Value::Null);
            let error = r.error.map_or(serde_json::Value::Null, |e| {
                json!({
                    CODE: e.code,
                    MESSAGE: e.message,
                    DATA: e.data.unwrap_or(serde_json::Value::Null)
                })
            });
            serde_to_lisp(json!({ID: r.id, RESULT: response, ERROR: error}), config)
        }
        Message::Notification(n) => {
            serde_to_lisp(json!({METHOD: n.method, PARAMS: n.params}), config)
        }
    };

    result.unwrap_or_else(|e| {
        error!(e);
    })
}

#[lisp_fn]
pub fn lsp_async_send_request(
    proc: LispObject,
    method: LispObject,
    params: LispObject,
    id: LispObject,
) -> bool {
    let mut emacs_pipe = unsafe { EmacsPipe::with_process(proc) };
    let method_s: LispStringRef = method.into();
    let id_s: LispStringRef = id.into();
    let config = get_process_json_config(proc);
    let value = lisp_to_serde(params, &config);
    let request = Message::Request(Request::new(
        RequestId::from(id_s.to_utf8()),
        method_s.to_utf8(),
        value.unwrap(),
    ));
    if let Err(e) = emacs_pipe.message_rust_worker(UserData::new(request)) {
        error!("Failed to send request to server, reason {:?}", e);
    }
    true
}

#[lisp_fn]
pub fn lsp_async_send_notification(
    proc: LispObject,
    method: LispObject,
    params: LispObject,
) -> bool {
    let mut emacs_pipe = unsafe { EmacsPipe::with_process(proc) };
    let method_s: LispStringRef = method.into();
    let config = get_process_json_config(proc);
    let value = lisp_to_serde(params, &config);
    let request = Message::Notification(Notification::new(method_s.to_utf8(), value.unwrap()));
    if let Err(e) = emacs_pipe.message_rust_worker(UserData::new(request)) {
        error!("Failed to send notification to server, reason {:?}", e);
    }

    true
}

#[lisp_fn(min = "1")]
pub fn lsp_json_config(args: &[LispObject]) -> bool {
    let proc = args[0];
    let config = generate_config_from_args(&args[1..]);
    let user_ptr: LispObject = UserData::new(config).into();

    let mut plist = unsafe { Fprocess_plist(proc) };
    plist = unsafe { Fplist_put(plist, QCjson_config, user_ptr) };
    unsafe { Fset_process_plist(proc, plist) };

    true
}

fn get_process_json_config(proc: LispObject) -> JSONConfiguration {
    let plist = unsafe { Fprocess_plist(proc) };
    let config_obj = unsafe { Fplist_get(plist, QCjson_config) };
    if config_obj.is_nil() {
        JSONConfiguration::default()
    } else {
        let config: &JSONConfiguration = unsafe { config_obj.as_userdata_ref() };
        config.clone()
    }
}

pub fn async_create_process(program: String, args: Vec<String>, pipe: EmacsPipe) -> Result<()> {
    let process: Child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut inn = process.stdin;
    let in_pipe = pipe.clone();
    thread::spawn(move || {
        let mut stdout_writer = BufWriter::new(inn.as_mut().unwrap());
        while let Ok(msg) = in_pipe.read_pend_message::<UserData>() {
            let value: Message = unsafe { msg.unpack() };

            if let Err(_) = value.write(&mut stdout_writer) {
                break;
            }
        }
    });

    let mut out = process.stdout;
    let mut out_pipe = pipe;
    let sender = out_pipe.get_sender();
    thread::spawn(move || {
        let mut stdout_reader = BufReader::new(out.as_mut().unwrap());
        loop {
            let parsed_message = Message::read(&mut stdout_reader);
            let msg = match parsed_message {
                Ok(Some(m)) => m,
                Ok(None) => Message::Response(Response::new_err(
                    RequestId::from(0),
                    PARSE_ERROR,
                    String::from("Unable to read from stdin"),
                )),
                Err(e) => Message::Response(Response::new_err(
                    RequestId::from(0),
                    PARSE_ERROR,
                    format!("JSON Message Error: {:?}", e),
                )),
            };

            if let Err(_) = out_pipe.message_lisp(&sender, UserData::new(msg)) {
                break;
            }
        }
    });

    Ok(())
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/out/client_exports.rs"
));
