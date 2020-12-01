use crate::lisp::LispObject;
use crate::multibyte::LispStringRef;
use crate::ng_async::{EmacsPipe, PipeDataOption, UserData};
use crate::lists::{LispCons, LispConsEndChecks, LispConsCircularChecks};
use lsp_server::Message;
use lsp_server::{RequestId, Response};
use remacs_macros::lisp_fn;
use serde_json::Value;
use std::io::{BufReader, BufWriter};
use std::process::{Command, Stdio};
use std::thread;

#[lisp_fn]
pub fn make_lsp_connection(
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

	list_args.iter_cars(LispConsEndChecks::on, LispConsCircularChecks::on)
	.for_each(|x| {
	    if let Some(string_ref) = x.as_string() {
		args_vec.push(string_ref.to_utf8());
	    } else {
		error!("make-lsp-command takes a list of string arguments");
	    }
	});
    }


    println!("Showing {:?}", args_vec);
    async_create_process(command_string, args_vec, emacs_pipe);
    proc
}

#[lisp_fn]
pub fn lsp_handler(_proc: LispObject, data: LispObject) -> bool {
    let user_data: UserData = data.into();
    let message: Message = unsafe { user_data.unpack() };
    println!("Handled message is {:?}", message);

    // Proc is our pipe process
    // any additional data (like further handlers, maps etc.
    // can be stored on proc and referenced here for dispatch
    // or further execution

    true
}

#[lisp_fn]
pub fn lsp_send_message(proc: LispObject, _msg: LispObject) -> bool {
    let mut emacs_pipe = unsafe { EmacsPipe::with_process(proc) };
    // Hardcoding message as an example
    let resp = Response {
        id: RequestId::from(10),
        result: Some(Value::from(10)),
        error: None,
    };
    let message = UserData::new(Message::Response(resp));
    // Instead of having a writer thread, we can just do the write here sync
    // OR use an async API to have a threadpool handle is behind the scenes
    emacs_pipe.message_rust_worker(message).unwrap();
    true
}

pub fn async_create_process(program: String, args: Vec<String>, pipe: EmacsPipe) {
    let process: std::process::Child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn child process");

    // @TODO better error handling......
    let mut inn = process.stdin;
    let in_pipe = pipe.clone();
    thread::spawn(move || {
        let mut stdout_writer = BufWriter::new(inn.as_mut().unwrap());
        while let Ok(msg) = in_pipe.read_pend_message::<UserData>() {
            let message: Message = unsafe { msg.unpack() };
            println!("I am writing {:?}", message);
            message.write(&mut stdout_writer);
        }
    });

    let mut out = process.stdout;
    let mut out_pipe = pipe.clone();
    thread::spawn(move || {
        let mut stdout_reader = BufReader::new(out.as_mut().unwrap());
        // @TODO better error handling
        while let Some(msg) = Message::read(&mut stdout_reader).unwrap() {
            println!("I received {:?}", msg);
            out_pipe.message_lisp(UserData::new(msg));
        }
    });
}

include!(concat!(env!("OUT_DIR"), "/parsing_exports.rs"));
