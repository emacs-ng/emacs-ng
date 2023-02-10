use std::{
    convert::TryInto,
    ffi::CString,
    fs::File,
    io::{Read, Write},
    os::unix::io::{FromRawFd, IntoRawFd},
};

use std::thread;

use crossbeam::channel::{Receiver, Sender};

use emacs::bindings::{
    build_string, intern_c_string, make_string_from_utf8, make_user_ptr, plist_get, plist_put,
    Ffuncall, Fmake_pipe_process, Fprocess_plist, Fset_process_plist, Fuser_ptrp, XUSER_PTR,
};
use emacs::globals::{
    QCcoding, QCfilter, QCinchannel, QCname, QCoutchannel, QCplist, QCtype, Qcall, Qdata, Qnil,
    Qraw_text, Qreturn, Qstring, Quser_ptr, Quser_ptrp,
};
use emacs::process::LispProcessRef;
use emacs::{lisp::LispObject, multibyte::LispStringRef};
use lisp_macros::{async_stream, lisp_fn};

#[repr(u32)]
enum PIPE_PROCESS {
    SUBPROCESS_STDIN = 0,
    WRITE_TO_SUBPROCESS = 1,
    READ_FROM_SUBPROCESS = 2,
    SUBPROCESS_STDOUT = 3,
    _READ_FROM_EXEC_MONITOR = 4,
    _EXEC_MONITOR_OUTPUT = 5,
}

#[derive(Clone)]
pub struct EmacsPipe {
    // Represents SUBPROCESS_STDOUT, used to write from a thread or
    // subprocess to the lisp thread.
    out_fd: i32,
    // Represents SUBPROCESS_STDIN, used to read from lisp messages
    in_fd: i32,
    _in_subp: i32,
    out_subp: i32,
    proc: LispObject,
}

const fn ptr_size() -> usize {
    core::mem::size_of::<*mut String>()
}

fn nullptr() -> usize {
    std::ptr::null() as *const i32 as usize
}

fn is_user_ptr(o: LispObject) -> bool {
    unsafe { Fuser_ptrp(o).into() }
}

fn to_data_option(obj: LispObject) -> Option<PipeDataOption> {
    match obj {
        Qstring => Some(String::marker()),
        Quser_ptr => Some(UserData::marker()),
        _ => None,
    }
}

fn from_data_option(option: PipeDataOption) -> LispObject {
    match option {
        PipeDataOption::STRING => Qstring,
        PipeDataOption::USER_DATA => Quser_ptr,
    }
}

/// UserData is a struct used for ease-of-use for turning Rust structs
/// into Lisp_User_Ptrs. UserData does NOT implement RAII. This means
/// that in order for the underlying data to be free'd, you will need
/// to either manually call finalize, or hand ownership over to lisp
/// and let the GC take care of the finalization.
pub struct UserData {
    finalizer: Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
    data: *mut libc::c_void,
}

// UserData will be safe to send because we will take ownership of
// the underlying data from Lisp.
unsafe impl Send for UserData {}

extern "C" fn rust_finalize<T>(raw: *mut libc::c_void) {
    let _t = unsafe { *Box::from_raw(raw as *mut T) };
}

impl UserData {
    pub fn with_data_and_finalizer(
        data: *mut libc::c_void,
        finalizer: Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
    ) -> Self {
        UserData {
            finalizer: finalizer,
            data: data,
        }
    }

    pub fn new<T: Sized>(t: T) -> UserData {
        let boxed = Box::into_raw(Box::new(t));
        let finalizer = rust_finalize::<T>;
        UserData::with_data_and_finalizer(boxed as *mut libc::c_void, Some(finalizer))
    }

    pub unsafe fn unpack<T: Sized>(self) -> T {
        *Box::from_raw(self.data as *mut T)
    }

    pub unsafe fn as_ref<T>(&self) -> &T {
        &(*(self.data as *const T))
    }
}

impl From<UserData> for LispObject {
    fn from(ud: UserData) -> Self {
        unsafe { make_user_ptr(ud.finalizer, ud.data) }
    }
}

pub fn to_owned_userdata(obj: LispObject) -> UserData {
    if obj.is_user_ptr() {
        unsafe {
            let p = XUSER_PTR(obj);
            let ptr = (*p).p;
            let fin = (*p).finalizer;
            (*p).p = std::ptr::null_mut();
            (*p).finalizer = None;
            UserData::with_data_and_finalizer(ptr, fin)
        }
    } else {
        wrong_type!(Quser_ptrp, obj);
    }
}

impl Default for UserData {
    fn default() -> Self {
        UserData {
            finalizer: None,
            data: std::ptr::null_mut(),
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
        let raw_proc: LispProcessRef = process.into();
        let out = raw_proc.open_fd[PIPE_PROCESS::SUBPROCESS_STDOUT as usize];
        let inf = raw_proc.open_fd[PIPE_PROCESS::SUBPROCESS_STDIN as usize];
        let pi = raw_proc.open_fd[PIPE_PROCESS::READ_FROM_SUBPROCESS as usize];
        let po = raw_proc.open_fd[PIPE_PROCESS::WRITE_TO_SUBPROCESS as usize];

        EmacsPipe {
            out_fd: out,
            in_fd: inf,
            _in_subp: pi,
            out_subp: po,
            proc: process,
        }
    }

    pub fn with_handler(
        handler: LispObject,
        input: PipeDataOption,
        output: PipeDataOption,
    ) -> (EmacsPipe, LispObject) {
        EmacsPipe::create(handler, input, output)
    }

    fn create(
        handler: LispObject,
        input: PipeDataOption,
        output: PipeDataOption,
    ) -> (EmacsPipe, LispObject) {
        let proc = unsafe {
            // We panic here only because it will be a fairly exceptional
            // situation in which I cannot alloc these small strings on the heap
            let cstr =
                CString::new("async-msg-buffer").expect("Failed to create pipe for async function");
            let async_str = CString::new("async-handler")
                .expect("Failed to crate string for intern function call");
            let mut proc_args = vec![
                QCname,
                build_string(cstr.as_ptr()),
                QCfilter,
                intern_c_string(async_str.as_ptr()),
                QCplist,
                Qnil,
                QCcoding,
                Qraw_text,
            ];

            // This unwrap will never panic because proc_args size is small
            // and will never overflow.
            Fmake_pipe_process(proc_args.len().try_into().unwrap(), proc_args.as_mut_ptr())
        };

        let input_type = from_data_option(input);
        let output_type = from_data_option(output);
        let mut plist = unsafe { Fprocess_plist(proc) };
        plist = unsafe { plist_put(plist, Qcall, handler) };
        plist = unsafe { plist_put(plist, QCtype, input_type) };
        plist = unsafe { plist_put(plist, Qreturn, output_type) };

        let (s, r): (Sender<String>, Receiver<String>) = crossbeam::channel::unbounded();
        plist = unsafe { plist_put(plist, QCinchannel, UserData::new(s).into()) };
        plist = unsafe { plist_put(plist, QCoutchannel, UserData::new(r).into()) };

        unsafe { Fset_process_plist(proc, plist) };
        // This should be safe due to the fact that we have created the process
        // ourselves
        (unsafe { EmacsPipe::with_process(proc) }, proc)
    }

    fn send(sender: &Sender<String>, s: String) -> std::io::Result<()> {
        sender.send(s).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Error while attempting to send message: {:?}",
                    e.into_inner()
                ),
            )
        })
    }

    pub fn get_sender(&self) -> Sender<String> {
        let plist = unsafe { Fprocess_plist(self.proc) };
        let sender_obj = unsafe { plist_get(plist, QCinchannel) };
        unsafe { sender_obj.as_userdata_ref::<Sender<String>>().clone() }
    }

    fn recv(&mut self) -> std::io::Result<String> {
        let plist = unsafe { Fprocess_plist(self.proc) };
        let recv_obj = unsafe { plist_get(plist, QCoutchannel) };
        let recv: &Receiver<String> = unsafe { recv_obj.as_userdata_ref() };
        recv.recv()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }

    // Called from the rust worker thread to send 'content' to the lisp
    // thread, to be processed by the users filter function
    // We don't use internal write due to the fact that in the lisp -> rust
    // direction, we write the raw data bytes to the pipe
    // In the rust -> lisp direction, we write the pointer as as string to the
    // crossbeam channel, and write the character 'r' to the data pipe.
    // This is due to the fact that the function that handles this data can be
    // invoked by the user, and we do not want to expose code that allows them
    // to enter in memory addresses for deference. This will also eliminate
    // the issue of 'partial reads' if an address crosses the arbitrary maximum
    // read value of a lisp data pipe (which is 4096 bytes as of this commit)
    pub fn message_lisp<T: PipeData>(
        &mut self,
        sender: &Sender<String>,
        content: T,
    ) -> std::io::Result<()> {
        let mut f = unsafe { File::from_raw_fd(self.out_fd) };
        let ptr = Box::into_raw(Box::new(content));
        let bin = ptr as *mut _ as usize;
        Self::send(sender, bin.to_string())?;
        f.write("r".as_bytes())?;
        f.into_raw_fd();
        Ok(())
    }

    fn internal_write(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        let mut f = unsafe { File::from_raw_fd(self.out_subp) };
        f.write(bytes)?;
        f.into_raw_fd();
        Ok(())
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
            Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "nullptr",
            ))
        } else {
            Ok(raw_value)
        }
    }

    // Used by the rust worker to receive incoming data. Messages sent from
    // calls to 'message_rust_worker' are recieved by read_pend_message
    pub fn read_pend_message<T: PipeData>(&self) -> std::io::Result<T> {
        self.read_next_ptr()
            .map(|v| unsafe { *Box::from_raw(v as *mut T) })
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

pub fn rust_worker<
    INPUT: Send + PipeData,
    OUTPUT: Send + PipeData,
    T: 'static + Fn(INPUT) -> OUTPUT + Send,
>(
    handler: LispObject,
    fnc: T,
) -> LispObject {
    let (mut pipe, proc) = EmacsPipe::with_handler(handler, INPUT::marker(), OUTPUT::marker());
    let sender = pipe.get_sender();
    thread::spawn(move || loop {
        match pipe.read_pend_message() {
            Ok(message) => {
                let result = fnc(message);
                if let Err(err) = pipe.message_lisp(&sender, result) {
                    eprint_if_unexpected_error(err);
                    break;
                }
            }
            Err(err) => {
                eprint_if_unexpected_error(err);
                break;
            }
        }
    });

    proc
}

fn make_return_value(ptrval: usize, option: PipeDataOption) -> LispObject {
    match option {
        PipeDataOption::STRING => {
            let content = unsafe { *Box::from_raw(ptrval as *mut String) };
            let nbytes = content.len();
            let c_content = CString::new(content).unwrap();
            // These unwraps should be 'safe', as we want to panic if we overflow
            unsafe { make_string_from_utf8(c_content.as_ptr(), nbytes.try_into().unwrap()) }
        }

        PipeDataOption::USER_DATA => {
            let content = unsafe { *Box::from_raw(ptrval as *mut UserData) };
            unsafe { make_user_ptr(content.finalizer, content.data) }
        }
    }
}

/// If 'data' is not a string, we have serious problems
/// as someone is writing to this pipe without knowing
/// how the data transfer functionality works. See below
/// comment.
#[lisp_fn]
pub fn async_handler(proc: LispObject, data: LispStringRef) -> bool {
    let plist = unsafe { Fprocess_plist(proc) };
    let orig_handler = unsafe { plist_get(plist, Qcall) };

    let mut pipe = unsafe { EmacsPipe::with_process(proc) };
    // This code may seem odd. Since we are in the same process space as
    // the lisp thread, our data transfer is not the data itself, but
    // a pointer to the data. However, 'async-handler' can be called by
    // the user, and allowing the user to control a string representing
    // a pointer is dangerous. So that information is kept within
    // an object that the user does not have direct access to via the
    // any lisp function. Instead, when data is ready, we write 'r'
    // over the pipe which triggers this function to read the pointer
    // data from a crossbeam channel.
    for _ in 0..data.len_bytes() {
        if let Ok(s) = pipe.recv() {
            let bin = s.parse::<usize>().unwrap();
            let qtype = unsafe { plist_get(plist, Qreturn) };
            if let Some(quoted_type) = to_data_option(qtype) {
                let retval = make_return_value(bin, quoted_type);
                let mut buffer = vec![orig_handler, proc, retval];
                unsafe { Ffuncall(3, buffer.as_mut_ptr()) };
            } else {
                // This means that someone has mishandled the
                // process plist and removed :type. Without this,
                // we cannot safely execute data transfer.
                wrong_type!(Qdata, qtype);
            }
        } else {
            error!("Failed to read recv data from pipe");
        }
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

fn internal_send_message(
    pipe: &mut EmacsPipe,
    message: LispObject,
    option: PipeDataOption,
) -> bool {
    match option {
        PipeDataOption::STRING => {
            let string: LispStringRef = message.into();
            pipe.message_rust_worker(string.to_utf8()).is_ok()
        }
        PipeDataOption::USER_DATA => {
            if !is_user_ptr(message) {
                wrong_type!(Quser_ptrp, message);
            }

            let data_ptr = unsafe { XUSER_PTR(message) };
            let data = unsafe { *data_ptr };
            let ud = UserData::with_data_and_finalizer(data.p, data.finalizer);
            unsafe {
                (*data_ptr).p = std::ptr::null_mut();
                (*data_ptr).finalizer = None;
            };

            pipe.message_rust_worker(ud).is_ok()
        }
    }
}

#[lisp_fn]
pub fn async_send_message(proc: LispObject, message: LispObject) -> bool {
    let mut pipe = unsafe { EmacsPipe::with_process(proc) };
    let plist = unsafe { Fprocess_plist(proc) };
    let qtype = unsafe { plist_get(plist, QCtype) };
    if let Some(option) = to_data_option(qtype) {
        internal_send_message(&mut pipe, message, option)
    } else {
        // This means that someone has mishandled the
        // process plist and removed :type. Without this,
        // we cannot safely execute data transfer.
        wrong_type!(Qdata, qtype);
    }
}

#[lisp_fn]
pub fn async_close_stream(proc: LispObject) -> bool {
    let mut pipe = unsafe { EmacsPipe::with_process(proc) };
    pipe.close_stream().is_ok()
}

#[allow(dead_code)]
fn init_syms() {
    def_lisp_sym!(QCinchannel, "inchannel");
    def_lisp_sym!(QCoutchannel, "outchannel");
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/out/ng_async_exports.rs"
));
