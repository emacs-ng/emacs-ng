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
    QCcoding,
    Qstring,
    Quser_ptr,
    Qcall,
    Qnil,
    Qraw_text,
    build_string,
    make_multibyte_string,
    make_user_ptr,
    intern_c_string,
    Fmake_pipe_process,
    Fset_process_plist,
    Fplist_put,
    Fplist_get,
    Fprocess_plist,
    Ffuncall,
    XPROCESS,
    XUSER_PTR,
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

impl LispObject {
    fn to_data_option(self) -> Option<PipeDataOption> {
	match self {
	    Qstring => Some(String::marker()),
	    Quser_ptr => Some(UserData::marker()),
	    _ => None
	}
    }

    fn from_data_option(option: PipeDataOption) -> LispObject {
	match option {
	    PipeDataOption::STRING => Qstring,
	    PipeDataOption::USER_DATA => Quser_ptr,
	}
    }
}

pub struct UserData {
    finalizer: Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
    data: *mut libc::c_void,
}

// UserData will be safe to send because we will take ownership of
// the underlying data from Lisp.
unsafe impl Send for UserData { }

impl UserData {
    fn with_data(finalizer: Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
		 data: *mut libc::c_void) -> UserData {
	UserData {
	    finalizer: finalizer,
	    data: data
	}
    }
}

// This enum defines the types that we will
// send through our data pipe.
// If you add a type to this enum, it should
// implement the trait 'PipeData'. This enum
// is a product of Rust's generic system
// combined with our usage pattern.
pub enum PipeDataOption {
    STRING,
    USER_DATA,
}

pub trait PipeData {
    fn marker() -> PipeDataOption;
}

impl PipeData for String {
    fn marker() -> PipeDataOption {
	PipeDataOption::STRING
    }
}

impl PipeData for UserData {
    fn marker() -> PipeDataOption {
	PipeDataOption::USER_DATA
    }
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

    pub fn with_handler(handler: LispObject, option: PipeDataOption) -> (EmacsPipe, LispObject) {
	EmacsPipe::create(handler, option)
    }

    fn create(handler: LispObject, option: PipeDataOption) -> (EmacsPipe, LispObject) {
	let proc = unsafe {
	    // We panic here only because it will be a fairly exceptional
	    // situation in which I cannot alloc these small strings on the heap
	    let cstr = CString::new("async-msg-buffer")
		.expect("Failed to create pipe for async function");
	    let async_str = CString::new("async-handler")
		.expect("Failed to crate string for intern function call");
	    let mut proc_args = vec![
		QCname, build_string(cstr.as_ptr()),
		QCfilter, intern_c_string(async_str.as_ptr()),
		QCplist, Qnil,
		QCcoding, Qraw_text,
	    ];

	    // This unwrap will never panic because proc_args size is small
	    // and will never overflow.
	    Fmake_pipe_process(proc_args.len().try_into().unwrap(),
			       proc_args.as_mut_ptr())
	};

	let qtype = LispObject::from_data_option(option);
	let mut plist = unsafe { Fprocess_plist(proc) };
	plist = unsafe { Fplist_put(plist, Qcall, handler) };
	plist = unsafe { Fplist_put(plist, QCtype, qtype) };
	unsafe { Fset_process_plist(proc, plist) };
	// This should be safe due to the fact that we have created the process
	// ourselves
	(unsafe { EmacsPipe::with_process(proc) }, proc)
    }

    // Called from the rust worker thread to send 'content' to the lisp
    // thread, to be processed by the users filter function
    // We don't use internal write due to the fact that in the lisp -> rust
    // direction, we write the raw data bytes to the pipe
    // In the rust -> lisp direction, we write the pointer as as string. This is
    // due to the fact that in the rust -> lisp direction, the data output will be
    // encoded as a string prior to being given to our handler.
    // An example pointer of 0xffff00ff as raw bytes will contain
    // a NULL TERMINATOR segment prior to pointer completion.
    pub fn message_lisp<T: PipeData>(&mut self, content: T) -> std::io::Result<()> {
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

    pub fn write_ptr<T: PipeData>(&mut self, ptr: *mut T) -> std::io::Result<()> {
	let bin = ptr as *mut _ as usize;
	self.internal_write(&bin.to_be_bytes())
    }

    // Called from the lisp thread, used to enqueue a message for the
    // rust worker to execute.
    pub fn message_rust_worker<T: PipeData>(&mut self, content: T) -> std::io::Result<()> {
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
    pub fn read_pend_message<T: PipeData>(&self) -> std::io::Result<T> {
	self.read_next_ptr().map(|v| unsafe { *Box::from_raw(v as *mut T) })
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

pub fn rust_worker<S: Send + PipeData,
		   T: 'static + Fn(S) -> S + Send>
    (handler: LispObject, fnc: T)
     -> LispObject {
    let (mut pipe, proc) = EmacsPipe::with_handler(handler, S::marker());
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

fn make_return_value(ptrval: usize, option: PipeDataOption) -> LispObject {
    match option {
	PipeDataOption::STRING => {
	    let content = unsafe { *Box::from_raw(ptrval as *mut String) };
	    let nchars = content.chars().count();
	    let nbytes = content.len();
	    let c_content = CString::new(content).unwrap();
	    // These unwraps should be 'safe', as we want to panic if we overflow
	    unsafe { make_multibyte_string(c_content.as_ptr(),
					   nchars.try_into().unwrap(),
					   nbytes.try_into().unwrap()) }
	},

	PipeDataOption::USER_DATA => {
	    let content = unsafe { *Box::from_raw(ptrval as *mut UserData) };
	    unsafe { make_user_ptr(content.finalizer, content.data) }
	}
    }
}

#[lisp_fn]
pub fn async_handler(proc: LispObject, data: LispObject) -> bool {
    let plist = unsafe { Fprocess_plist(proc) };
    let orig_handler = unsafe { Fplist_get(plist, Qcall) };

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

    let qtype = unsafe { Fplist_get(plist, QCtype) };
    if let Some(quoted_type) = qtype.to_data_option() {
	let retval = make_return_value(bin, quoted_type);
	let mut buffer = vec![orig_handler, proc, retval];
	unsafe { Ffuncall(3, buffer.as_mut_ptr()) };
    } else {
	// @TODO signal an error here, don't panic.
    }

    true
}

#[async_stream]
pub async fn async_echo(s: String) -> String {
    s
}

#[async_stream]
pub async fn async_data_echo(e: UserData) -> UserData {
    e
}

fn internal_send_message(pipe: &mut EmacsPipe, message: LispObject, option: PipeDataOption) -> bool {
    match option {
	PipeDataOption::STRING => {
	    let sdata = unsafe { SDATA(message) };
	    let ssize = unsafe { SBYTES(message) };
	    let sslice = unsafe { slice::from_raw_parts(sdata as *const u8, ssize as usize) };
	    let contents = String::from_utf8_lossy(sslice);
	    pipe.message_rust_worker(contents.into_owned()).is_ok()
	},
	PipeDataOption::USER_DATA => {
	    let data_ptr = unsafe { XUSER_PTR(message) };
	    let data = unsafe { *data_ptr };
	    let ud = UserData::with_data(data.finalizer, data.p);
	    unsafe {
		(*data_ptr).p = std::ptr::null_mut();
		(*data_ptr).finalizer = None;
	    };

	    pipe.message_rust_worker(ud).is_ok()
	},

    }
}

#[lisp_fn]
pub fn async_send_message(proc: LispObject, message: LispObject) -> bool {
    let mut pipe = unsafe { EmacsPipe::with_process(proc) };
    let plist = unsafe { Fprocess_plist(proc) };
    let qtype = unsafe { Fplist_get(plist, QCtype) };
    if let Some(option) = qtype.to_data_option() {
	internal_send_message(&mut pipe, message, option)
    } else {
	// @TODO signal error
	false
    }
}

#[lisp_fn]
pub fn async_close_stream(proc: LispObject) -> bool {
    let mut pipe = unsafe { EmacsPipe::with_process(proc) };
    pipe.close_stream().is_ok()
}

include!(concat!(env!("OUT_DIR"), "/ng_async_exports.rs"));
