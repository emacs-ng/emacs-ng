use remacs_macros::{lisp_fn, async_stream};
use std::{
    thread,
    slice,
};
use crate::lisp::LispObject;
use crate::remacs_sys::{
    QCname,
    QCfilter,
    QCplist,
    QCtype,
    Qstring,
    Qcall,
    Qnil,
    build_string,
    make_multibyte_string,
    intern_c_string,
    Fmake_pipe_process,
    Fset_process_plist,
    Fplist_put,
    Fplist_get,
    Fprocess_plist,
    Ffuncall,
    XPROCESS,
    SDATA,
    SBYTES,
    Lisp_Type,
};

use std::{
    fs::File,
    io::{Write, Read},
    os::unix::io::{FromRawFd, IntoRawFd},
    convert::TryInto,
    ffi::CString,
};

#[repr(u32)]
enum PIPE_PROCESS {
    SUBPROCESS_STDIN = 0,
    WRITE_TO_SUBPROCESS = 1,
    READ_FROM_SUBPROCESS = 2,
    SUBPROCESS_STDOUT = 3,
    _READ_FROM_EXEC_MONITOR = 4,
    _EXEC_MONITOR_OUTPUT = 5,
}

pub struct EmacsPipe {
    // Represents SUBPROCESS_STDOUT, used to write from a thread or
    // subprocess to the lisp thread.
    out_fd: i32,
    // Represents SUBPROCESS_STDIN, used to read from lisp messages
    in_fd: i32,
    _in_subp: i32,
    out_subp: i32,
}

const fn ptr_size() -> usize {
    core::mem::size_of::<*mut String>()
}

fn nullptr() -> usize {
    std::ptr::null() as *const i32 as usize
}

pub enum PipeOptions {
    STRING,
    USER_DATA,
}

impl EmacsPipe {
    pub unsafe fn with_process(process: LispObject) -> EmacsPipe {
	let raw_proc = XPROCESS(process);
	let out = (*raw_proc).open_fd[PIPE_PROCESS::SUBPROCESS_STDOUT as usize];
	let inf = (*raw_proc).open_fd[PIPE_PROCESS::SUBPROCESS_STDIN as usize];
	let pi = (*raw_proc).open_fd[PIPE_PROCESS::READ_FROM_SUBPROCESS as usize];
	let po = (*raw_proc).open_fd[PIPE_PROCESS::WRITE_TO_SUBPROCESS as usize];

	EmacsPipe {
	    out_fd: out,
	    in_fd: inf,
	    _in_subp: pi,
	    out_subp: po,
	}
    }

    pub fn with_handler(handler: LispObject) -> (EmacsPipe, LispObject) {
	EmacsPipe::create(handler, PipeOptions::STRING) // @TODO don't hardcode
    }

    pub fn _new() -> (EmacsPipe, LispObject) {
	EmacsPipe::create(false.into(), PipeOptions::STRING) // @TODO don't hardcode
    }

    fn create(handler: LispObject, options: PipeOptions) -> (EmacsPipe, LispObject) {
	let proc = unsafe {
	    // @TODO revisit this buffer name. I have not found a way to avoid
	    // creating a buffer for this pipe. Sharing a buffer amoung pipes is fine
	    // as long as we create different fds for exchanging information.
	    let cstr = CString::new("async-msg-buffer")
		.expect("Failed to create pipe for async function");
	    let async_str = CString::new("async-handler")
		.expect("Failed to crate string for intern function call");
	    let mut proc_args = vec![
		QCname, build_string(cstr.as_ptr()),
		QCfilter, intern_c_string(async_str.as_ptr()),
		QCplist, Qnil,
	    ];

	    // This unwrap will never panic because proc_args size is small
	    // and will never overflow.
	    Fmake_pipe_process(proc_args.len().try_into().unwrap(),
			       proc_args.as_mut_ptr())
	};

	let plist = unsafe { Fprocess_plist(proc) };
	unsafe { Fset_process_plist(proc, Fplist_put(plist, Qcall, handler)) };
	match options {
	    PipeOptions::STRING => unsafe { Fplist_put(plist, QCtype, Qstring) },
	    PipeOptions::USER_DATA => panic!("Not Yet Supported"),
	};


	// This should be safe due to the fact that we have created the process
	// ourselves
	(unsafe { EmacsPipe::with_process(proc) }, proc)
    }

    // Called from the rust worker thread to send 'content' to the lisp
    // thread, to be processed by the users filter function
    pub fn message_lisp(&mut self, content: String) -> std::io::Result<()> {
	let mut f = unsafe { File::from_raw_fd(self.out_fd) };
	let ptr = Box::into_raw(Box::new(content));
	let bin = ptr as *mut _ as usize;
	let result = f.write(bin.to_string().as_bytes()).map(|_| ());
	f.into_raw_fd();
	result
    }

    fn internal_write(&mut self, bytes: &[u8]) -> std::io::Result<()> {
	let mut f = unsafe { File::from_raw_fd(self.out_subp) };
	let result = f.write(bytes)
	    .map(|_| ());
	f.into_raw_fd();
	result
    }

    pub fn write_ptr<T>(&mut self, ptr: *mut T) -> std::io::Result<()> {
	let bin = ptr as *mut _ as usize;
	self.internal_write(&bin.to_be_bytes())
    }

    // Called from the lisp thread, used to enqueue a message for the
    // rust worker to execute.
    pub fn message_rust_worker(&mut self, content: String) -> std::io::Result<()> {
	self.write_ptr(Box::into_raw(Box::new(content)))
    }

    pub fn read_next_ptr(&self) -> std::io::Result<usize> {
	let mut f = unsafe { File::from_raw_fd(self.in_fd) };
	let mut buffer = [0; ptr_size()];
	f.read(&mut buffer)?;
	let raw_value = usize::from_be_bytes(buffer);
	f.into_raw_fd();

	if raw_value == nullptr() {
	    Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "nullptr"))
	} else {
	    Ok(raw_value)
	}
    }

    // Used by the rust worker to receive incoming data. Messages sent from
    // calls to 'message_rust_worker' are recieved by read_pend_message
    pub fn read_pend_message(&self) -> std::io::Result<String> {
	self.read_next_ptr().map(|v| unsafe { *Box::from_raw(v as *mut String) })
    }

    pub fn close_stream(&mut self) -> std::io::Result<()> {
	self.internal_write(&nullptr().to_be_bytes())
    }
}

fn eprint_if_unexpected_error(err: std::io::Error) {
    // If we explicity set "ConnectionAborted" to close the stream
    // we don't want to log, as that was expected.
    if err.kind() != std::io::ErrorKind::ConnectionAborted {
	eprintln!("Async stream closed; Reason {:?}", err);
    }
}

pub fn rust_worker<T: 'static + Fn(String) -> String + Send>(handler: LispObject, fnc: T)
							     -> LispObject {
    let (mut pipe, proc) = EmacsPipe::with_handler(handler);
    thread::spawn(move || {
	loop {
	    match pipe.read_pend_message() {
		Ok(message) => {
		    let result = fnc(message);
		    if let Err(err) = pipe.message_lisp(result) {
			eprint_if_unexpected_error(err);
			break;
		    }
		},
		Err(err) => {
		    eprint_if_unexpected_error(err);
		    break;
		}
	    }
	}
    });

    proc
}

#[lisp_fn]
pub fn async_handler(proc: LispObject, data: LispObject) -> bool {
    let orig_handler = unsafe {
	let plist = Fprocess_plist(proc);
	Fplist_get(plist, Qcall)
    };

    // This code may seem odd. Since we are in the same process space as
    // the lisp thread, our data transfer is not the string itself, but
    // a pointer to the string. We translate the pointer to a usize, and
    // write the string representation of that pointer over the pipe.
    // This code extracts that data, and gets us the acutal Rust String
    // object, that we then translate to a lisp object.
    let sdata = unsafe { SDATA(data) };
    let ssize = unsafe { SBYTES(data) };
    let sslice = unsafe { slice::from_raw_parts(sdata as *const u8, ssize as usize) };
    let bin = String::from_utf8_lossy(sslice).parse::<usize>().unwrap();
    let content = unsafe { *Box::from_raw(bin as *mut String) };

    let nchars = content.chars().count();
    let nbytes = content.len();

    let c_content = CString::new(content).unwrap();
    // These unwraps should be 'safe', as we want to panic if we overflow
    let calculated_string = unsafe { make_multibyte_string(c_content.as_ptr(),
						nchars.try_into().unwrap(),
						nbytes.try_into().unwrap()) };

    let mut buffer = vec![orig_handler, proc, calculated_string];
    unsafe { Ffuncall(3, buffer.as_mut_ptr()) };
    true
}

#[async_stream]
pub async fn async_echo(s: String) -> String {
    s
}

#[lisp_fn]
pub fn async_send_message(proc: LispObject, message: LispObject) -> bool {
    let mut pipe = unsafe { EmacsPipe::with_process(proc) };
    let sdata = unsafe { SDATA(message) };
    let ssize = unsafe { SBYTES(message) };
    let sslice = unsafe { slice::from_raw_parts(sdata as *const u8, ssize as usize) };
    let contents = String::from_utf8_lossy(sslice);
    pipe.message_rust_worker(contents.into_owned()).is_ok()
}

#[lisp_fn]
pub fn async_close_stream(proc: LispObject) -> bool {
    let mut pipe = unsafe { EmacsPipe::with_process(proc) };
    pipe.close_stream().is_ok()
}

include!(concat!(env!("OUT_DIR"), "/ng_async_exports.rs"));
