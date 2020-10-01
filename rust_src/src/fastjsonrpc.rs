extern crate remacs_generated;
use lsp_server::RequestId;
use lsp_server::Response;
use remacs_generated::lisp::LispObject;
use remacs_generated::lisp::make_user_ptr;
// use remacs_generated::lisp::*;
use remacs_generated::remacs_sys::EmacsInt;
use remacs_generated::multibyte::LispStringRef;
use remacs_generated::{remacs_sys::Lisp_Buffer};
use serde_json::Value;
use core::slice;
use std::{
    cell::RefCell,
    io::{self, BufReader, BufWriter},
    process::{self, Command, Stdio},
};

// use remacs_generated::remacs_sys::E
// use remacs_sys::make_user_ptr;
// use remacs_sys::Lisp_User_Ptr;
use remacs_generated::lisp::message1;
use remacs_generated::lisp::Lisp_User_Ptr;
// use remacs_generated::remacs_sys::

use crossbeam_channel::{bounded, Receiver, Sender};
use lsp_server::Message;
// use process::{Command, Stdio};
// use std::io;
// use std::process;
use std::{os::raw::c_void, thread};
// /// Creates an LSP connection via stdio.

fn stdio_client(program: &str, args: Vec<&str>) -> JsonRpcStdio {
    let process: process::Child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn child process");


    let (writer, writer_receiver) = bounded::<Message>(0);
    let mut inn = process.stdin;
    let writer_thread = thread::spawn(move || {
        let mut stdout_writer = BufWriter::new(inn.as_mut().unwrap());
        writer_receiver
            .into_iter()
            .try_for_each(|it| it.write(&mut stdout_writer))?;
        Ok(())
    });

    let (reader_sender, reader) = bounded::<Message>(0);
    let mut out = process.stdout;
    let reader_thread = thread::spawn(move || {
        let mut stdout_reader = BufReader::new(out.as_mut().unwrap());
        while let Some(msg) = Message::read(&mut stdout_reader)? {
            println!("[bkg thread] >>> {:?}", msg);
            match reader_sender.send(msg) {
                Ok(a) => a,
                Err(msg1) => println!("error when sending >>> {:?}", msg1)
            };
        }
        println!("XXX >>> exiting...");
        Ok(())
    });

    JsonRpcStdio {
        reader,
        writer,
        reader_thread,
        writer_thread,
    }
}


#[derive(Debug)]
struct JsonRpcStdio {
    reader: Receiver<Message>,
    writer: Sender<Message>,
    reader_thread: thread::JoinHandle<io::Result<()>>,
    writer_thread: thread::JoinHandle<io::Result<()>>,
}

impl JsonRpcStdio {
    fn finalize() {}
}

thread_local! {
    static STORAGE: RefCell<Option<JsonRpcStdio>> = RefCell::new(None);
}

fn user_pointer<T>(v: T) -> LispObject {
    let connection = Box::into_raw(Box::new(v)) as *mut c_void;
    unsafe { make_user_ptr(Some(finalize), connection)
    }
}

#[test]
fn test_name() {
    stdio_client("cat", vec!["/dev/stdin"]);
}
#[no_mangle]
pub extern "C" fn fastjsonrcp_connection(input: LispObject) -> LispObject {
    println!("Creating connection...");
    let connection = stdio_client("cat", vec!["/dev/stdin"]);
    user_pointer(connection)
}

impl<'a> From< &'a LispObject> for &'a JsonRpcStdio  {
    fn from(lisp_object:  &'a LispObject) -> &'a JsonRpcStdio {
        println!("1");
        let up = lisp_object.get_untaggedptr() as *mut Lisp_User_Ptr;
        println!("2");
        unsafe {
            let connection = (*up).p as *mut _ as *mut JsonRpcStdio;
            println!("3");
            &(*connection)
        }
    }
}

#[no_mangle]
pub extern "C" fn fastjsonrcp_send_message(connection: LispObject) {
    println!("Begin sending message...");

    let resp = Response { id: RequestId::from(10), result: Some(Value::from(10)), error: None };
    println!("Resp {:?}", resp);

    let connection: &JsonRpcStdio = (&connection).into();
    println!("Connection read... {:?}", connection);

    connection.writer.send(Message::Response(resp));
    println!("Sent message...")
}


#[no_mangle]
pub extern "C" fn finalize(_data: *mut c_void) {

}

#[no_mangle]
pub extern "C" fn fastjsonrcp_get_message(connection: &LispObject) {
    let connection: &JsonRpcStdio = connection.into();
    match connection.reader.try_recv() {
        Ok(a) => {
            println!("[main-thread rust] received from bkg thread: {:?}", a);
        }

        Err(msg) => {
            println!("[main-thread rust] received from bkg thread: {:?}", msg);
        }
    }
}
