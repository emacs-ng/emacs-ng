use remacs_macros::{lisp_fn, async_stream};
use lazy_static::lazy_static;
use std::{
    thread,
    slice,
};
use crate::lisp::LispObject;
use crate::remacs_sys::{
    QCname,
    QCfilter,
    build_string,
    Fmake_pipe_process,
    XPROCESS,
    SDATA,
    SBYTES,
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
    READ_FROM_EXEC_MONITOR = 4,
    EXEC_MONITOR_OUTPUT = 5,
}

pub struct EmacsPipe {
    // Represents SUBPROCESS_STDOUT, used to write from a thread or
    // subprocess to the lisp thread.
    out_fd: i32,
    // Represents SUBPROCESS_STDIN, used to read from lisp messages
    in_fd: i32,
    in_subp: i32,
    out_subp: i32,
}

const fn ptr_size() -> usize {
    core::mem::size_of::<*mut String>()
}

fn nullptr() -> usize {
    std::ptr::null() as *const i32 as usize
}


impl EmacsPipe {
    unsafe fn assert_main_thread() {
	// TODO
    }

    pub unsafe fn with_process(process: LispObject) -> EmacsPipe {
	EmacsPipe::assert_main_thread();

	let raw_proc = XPROCESS(process);
	let out = (*raw_proc).open_fd[PIPE_PROCESS::SUBPROCESS_STDOUT as usize];
	let inf = (*raw_proc).open_fd[PIPE_PROCESS::SUBPROCESS_STDIN as usize];
	let pi = (*raw_proc).open_fd[PIPE_PROCESS::READ_FROM_SUBPROCESS as usize];
	let po = (*raw_proc).open_fd[PIPE_PROCESS::WRITE_TO_SUBPROCESS as usize];

	EmacsPipe {
	    out_fd: out,
	    in_fd: inf,
	    in_subp: pi,
	    out_subp: po,
	}
    }

    pub fn with_handler(handler: LispObject) -> (EmacsPipe, LispObject) {
	EmacsPipe::create(handler)
    }

    pub fn new() -> (EmacsPipe, LispObject) {
	EmacsPipe::create(false.into())
    }

    fn create(handler: LispObject) -> (EmacsPipe, LispObject) {
	let proc = unsafe {
	    // @TODO revisit this buffer name. I have not found a way to avoid
	    // creating a buffer for this pipe. Sharing a buffer amoung pipes is fine
	    // as long as we create different fds for exchanging information.
	    let cstr = CString::new("async-msg-buffer")
		.expect("Failed to create pipe for async function");
	    let mut proc_args = vec![
		QCname, build_string(cstr.as_ptr()),
		QCfilter, handler,
	    ];

	    // This unwrap will never panic because proc_args size is small
	    // and will never overflow.
	    Fmake_pipe_process(proc_args.len().try_into().unwrap(),
			       proc_args.as_mut_ptr())
	};

	// This should be safe due to the fact that we have created the process
	// ourselves
	(unsafe { EmacsPipe::with_process(proc) }, proc)
    }

    // Called from the rust worker thread to send 'content' to the lisp
    // thread, to be processed by the users filter function
    pub fn message_lisp(&mut self, content: String) -> std::io::Result<()> {
	let mut f = unsafe { File::from_raw_fd(self.out_fd) };
	let result = write!(&mut f, "{}", content);
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

    // Called from the lisp thread, used to enqueue a message for the
    // rust worker to execute.
    pub fn message_rust_worker(&mut self, content: String) -> std::io::Result<()> {
	let raw_ptr = Box::into_raw(Box::new(content));
	let bin = raw_ptr as *mut _ as usize;
	self.internal_write(&bin.to_be_bytes())
    }

    // Used by the rust worker to receive incoming data. Messages sent from
    // calls to 'message_rust_worker' are recieved by read_pend_message
    pub fn read_pend_message(&self) -> std::io::Result<String> {
	let mut f = unsafe { File::from_raw_fd(self.in_fd) };
	let mut buffer = [0; ptr_size()];
	let size = f.read(&mut buffer)?;
	let raw_value = usize::from_be_bytes(buffer);

	if raw_value == nullptr() {
	    Err(std::io::Error::new(std::io::ErrorKind::Other, "nullptr"))
	} else {
	    Ok(unsafe { *Box::from_raw(raw_value as *mut String) })
	}
    }

    pub fn close_stream(&mut self) -> std::io::Result<()> {
	self.internal_write(&nullptr().to_be_bytes())
    }
}

pub fn rust_worker<T: 'static + Fn(String) -> String + Send>(handler: LispObject, fnc: T)
							     -> LispObject {
    let (mut pipe, proc) = EmacsPipe::with_handler(handler);
    thread::spawn(move || {
	loop {
	    if let Ok(message) = pipe.read_pend_message() {
		let result = fnc(message);
		pipe.message_lisp(result);
	    } else {
		// While I think this is all we want to do
		// it is likely a good idea to note the error
		// if its not "nullptr", which is expected in the
		// case we want to close the stream.
		break;
	    }
	}
    });

    proc
}

#[async_stream]
pub async fn my_async_fn(s: String) -> String {
    s
}

#[lisp_fn]
pub fn async_send_message(proc: LispObject, message: LispObject) -> bool {
    let mut pipe = unsafe { EmacsPipe::with_process(proc) };
    let sdata = unsafe { SDATA(message) };
    let ssize = unsafe { SBYTES(message) };
    let sslice = unsafe { slice::from_raw_parts(sdata as *const u8, ssize as usize) };
    let contents = String::from_utf8_lossy(sslice);
    pipe.message_rust_worker(contents.into_owned());

    true
}

#[lisp_fn]
pub fn async_close_stream(proc: LispObject) -> bool {
    let mut pipe = unsafe { EmacsPipe::with_process(proc) };
    pipe.close_stream();
    true
}

include!(concat!(env!("OUT_DIR"), "/ng_async_exports.rs"));
