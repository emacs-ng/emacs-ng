use remacs_macros::lisp_fn;
use lazy_static::lazy_static;
use std::thread;
use crate::lisp::LispObject;
use crate::remacs_sys::{QCname, QCfilter, build_string, Fmake_pipe_process, XPROCESS};

use std::{
    fs::File,
    io::{Write, Read},
    os::unix::io::{FromRawFd, IntoRawFd},
    convert::TryInto,
    ffi::CString,
};

#[repr(u32)]
enum ProcessHandles {
    SUBPROCESS_STDIN = 0,
    WRITE_TO_SUBPROCESS = 1,
    READ_FROM_SUBPROCESS = 2,
    SUBPROCESS_STDOUT = 3,
    READ_FROM_EXEC_MONITOR = 4,
    EXEC_MONITOR_OUTPUT = 5,
}

// Used for a off-thread streaming calculations
// 1. NOT allowed to interact with Lisp objects or lisp runtime
//    1a. Mutation of 'input' does not directly affect lisp runtime
// 2. Will be invoked when data is written via call to send-process-string in lisp
// 3. Output data will be passed to filter function
// Usage: (setq my-stream (my-async-callback inital-args 'filter))
//        (send-process-string my-stream "input1")
//        (send-process-string my-stream "input2")
// Results will be called on 'filter
// User is allowed to specify minimum time between read() calls on
// data pipe
//#[emacs_stream]
async fn my_async_stream(input: String) -> String {
    input
}

// Used for single async function
// 1. NOT allowed to interact with Lisp objects or lisp runtime
// 2. Allowed to spawn child threads to execute additional actions
// Usage: (my-asyncfn 1 2 3 4 'mycallback)
// Can be used sync with arg :sync t
// which allows lisp runtime to wait on method completion. Result will be returned
// syncronusly
//#[emacs_async]
async fn my_async_fn(/* input args */) -> i32 {
    5
}


unsafe fn get_io_stream(proc: LispObject) -> (i32, i32) {
    let raw_proc = XPROCESS(proc);
    let out = (*raw_proc).open_fd[ProcessHandles::SUBPROCESS_STDOUT as usize];
    let inf = (*raw_proc).open_fd[ProcessHandles::SUBPROCESS_STDIN as usize];

    (inf, out)
}

struct EmacsPipe {
    out_fd: i32,
    in_fd: i32,
}

const BUFFER_SIZE: usize = 4096 * 4096 * 2;
impl EmacsPipe {
    unsafe fn assert_main_thread() {
	// TODO
    }

    unsafe fn with_process(process: LispObject) -> EmacsPipe {
	EmacsPipe::assert_main_thread();

	let raw_proc = XPROCESS(process);
	let out = (*raw_proc).open_fd[ProcessHandles::SUBPROCESS_STDOUT as usize];
	let inf = (*raw_proc).open_fd[ProcessHandles::SUBPROCESS_STDIN as usize];

	EmacsPipe {
	    out_fd: out,
	    in_fd: inf,
	}
    }

    fn write(&mut self, content: &str) -> std::io::Result<()> {
	let mut f = unsafe { File::from_raw_fd(self.out_fd) };
	let result = write!(&mut f, "{}", content);
	f.into_raw_fd();
	result
    }

    fn read(&self) -> String {
	let mut f = unsafe { File::from_raw_fd(self.in_fd) };
	// @TODO
	/* Latest working code
	let mut contents = String::new();
	    {
		let mut fi = unsafe { File::from_raw_fd(inproc) };
		let mut buffer = [0; 10]; // Let small just for this example. Final logic will
		                          // look different if we even use  this pattern
		if let Ok(size) = fi.read(&mut buffer) {
		    contents = std::str::from_utf8(&buffer[..size]).unwrap().to_string();
		}

		let _ii = fi.into_raw_fd();
	    }
	 */

	String::from("hello pipe")
    }
}

#[lisp_fn(min = "1")] // This will be num_args + 1 for handler, making sync optional
pub fn async_function(/* args ,*/ handler: LispObject, sync: LispObject) -> LispObject {

    let proc = unsafe {
	// @TODO revisit this buffer name. I have not found a way to avoid
	// creating a buffer for this pipe. Sharing a buffer amoung pipes is fine
	// as long as we create different fds for exchanging information.
	let cstr = CString::new("_internal_proc")
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

    let mut pipe = unsafe { EmacsPipe::with_process(proc) };

    if !sync.is_t() {
	thread::spawn(move || {
	    let future = async move {
		let result = my_async_fn(/* args */).await;
		pipe.write(&result.to_string());
	    };

	    futures::executor::block_on(future);
	});
    } else {
	// ffuncall handler with proc / my_async_fn() as string
    }

    proc
}

include!(concat!(env!("OUT_DIR"), "/json_rpc_exports.rs"));
