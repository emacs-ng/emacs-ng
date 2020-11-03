use remacs_macros::lisp_fn;
use lazy_static::lazy_static;
use std::thread;
use crate::lisp::LispObject;
use crate::remacs_sys::{XPROCESS};

use std::{
    fs::File,
    io::{self, Write, Read, BufReader},
    os::unix::io::{FromRawFd, IntoRawFd},
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

#[lisp_fn]
pub fn thread_path(proc: LispObject) -> bool {
    let proc_raw = unsafe { XPROCESS(proc) };
    // Write to this FD to give data to lisp.
    let outproc = unsafe { (*proc_raw).open_fd[ProcessHandles::SUBPROCESS_STDOUT as usize] };

    // Read from this FD to recieve data from lisp.
    let inproc = unsafe { (*proc_raw).open_fd[ProcessHandles::SUBPROCESS_STDIN as usize] };

    thread::spawn(move || {
	let mut f = unsafe { File::from_raw_fd(outproc) };
	write!(&mut f, "Pipe is working\n");
	let _i = f.into_raw_fd();

	loop {

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

	    if contents.len() > 0 {
		let mut ff = unsafe { File::from_raw_fd(outproc) };
		write!(&mut ff, "{}", contents);
		let _I = ff.into_raw_fd();
	    }

	}
    });

    true
}

include!(concat!(env!("OUT_DIR"), "/json_rpc_exports.rs"));
