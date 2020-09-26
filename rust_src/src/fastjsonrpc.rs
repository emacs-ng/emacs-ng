extern crate remacs_generated;
use remacs_generated::lisp::LispObject;
use remacs_generated::lisp::make_user_ptr;
use remacs_generated::remacs_sys::EmacsInt;
use remacs_generated::{remacs_sys::Lisp_Buffer};
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

fn stdio_client() -> JsonRpcStdio {
    let process: process::Child = Command::new("/bin/cat")
        .arg("/home/kyoncho/file.txt")
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

pub fn fastjsonrcp_connection(_input: LispObject) -> LispObject {
    let connection = Box::into_raw(Box::new(stdio_client())) as *mut c_void;
    unsafe { make_user_ptr(Some(finalize), connection) }
}

#[no_mangle]
pub extern "C" fn finalize(_data: *mut c_void) {

}

// pub fn from_pointer<T> (object: LispObject) -> T {
//     unsafe {
//         let up = object.get_untaggedptr() as *mut Lisp_User_Ptr;
//         (*up).p as *mut _ as T
//     };
// }

pub fn fastjsonrcp_get_message(input: LispObject) -> LispObject {
    // unsafe  {
    //     message1(format!("{}", "xx").as_ptr() as *const ::libc::c_char);
    //     input
    // }

    let connection: *mut JsonRpcStdio = unsafe {
        let d = (*(input.get_untaggedptr() as *mut Lisp_User_Ptr)).p;
        d as *mut JsonRpcStdio
    };

    unsafe {
        match (*connection).reader.try_recv() {
            Ok(a) => {
                println!("[main-thread rust] received from bkg thread: {:?}", a);
            }

            Err(msg) => {
                println!("[main-thread rust] received from bkg thread: {:?}", msg);
                message1(format!("{}", msg).as_ptr() as *const ::libc::c_char);
           }
        }
    }
    input
}
