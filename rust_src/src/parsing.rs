use crate::ng_async::{EmacsPipe, PipeDataOption, UserData, to_owned_userdata};
use lisp::lisp::LispObject;
use lisp::lists::{LispCons, LispConsCircularChecks, LispConsEndChecks};
use lisp::multibyte::LispStringRef;
use lisp_macros::lisp_fn;
use lsp_server::{Message, Request, RequestId, Response};
use serde_json::{map::Map, Value};
use std::convert::TryInto;
use std::ffi::CString;
use std::io::{BufReader, BufWriter, Error, Result};
use std::process::{Child, Command, Stdio};
use std::thread;

use lisp::remacs_sys::{
    check_integer_range, hash_lookup, hash_put, intmax_t, make_fixed_natnum, make_float, make_int,
    make_string_from_utf8, make_uint, make_vector, Fcons, Fintern, Flist, Fmake_hash_table,
    Fnreverse, Fplist_get, Fplist_put, Fprocess_plist, Fset_process_plist, QCarray_type, QCfalse,
    QCfalse_object, QCjson_config, QCnull, QCnull_object, QCobject_type, QCsize, QCtest, Qalist,
    Qarray, Qequal, Qhash_table, Qlist, Qnil, Qplist, Qplistp, Qt, Qunbound, AREF, ASET, ASIZE,
    CHECK_SYMBOL, FLOATP, HASH_KEY, HASH_TABLE_P, HASH_TABLE_SIZE, HASH_VALUE, INTEGERP, NILP,
    STRINGP, SYMBOL_NAME, VECTORP, XFLOAT_DATA, XHASH_TABLE,
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

#[derive(Clone)]
pub(crate) enum ObjectType {
    Hashtable,
    Alist,
    Plist,
}

#[derive(Clone)]
pub(crate) enum ArrayType {
    Array,
    List,
}

#[derive(Clone)]
pub(crate) struct JSONConfiguration {
    pub(crate) obj: ObjectType,
    pub(crate) arr: ArrayType,
    pub(crate) null_obj: LispObject,
    pub(crate) false_obj: LispObject,
}

impl Default for JSONConfiguration {
    fn default() -> Self {
        JSONConfiguration {
            obj: ObjectType::Hashtable,
            arr: ArrayType::Array,
            null_obj: QCnull,
            false_obj: QCfalse,
        }
    }
}

/// Create a 'child process' defined by STRING 'command'
/// 'args' is a list of STRING arguments for the invoked command. Can be NIL
/// handler is the FUNCTION that will be invoked on the result data
/// returned from the process via stdout. The handler should take two
/// arguments, the pipe process and the data. Data will be returned as
/// a 'user-ptr', which should be passed to lsp-handler for further processing.
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
    match msg {
        Message::Request(re) => serde_to_lisp(
            json!({ID: re.id, METHOD: re.method, PARAMS: re.params}),
            config,
        ),
        Message::Response(r) => {
            let response = r.result.unwrap_or_else(|| serde_json::Value::Null);
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
    }
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

/*

#[lisp_fn]
pub fn lsp_lazy_handler(_proc: LispObject, data: LispObject) -> LispObject {
    let user_data: UserData = data.into();
    let msg: Message = unsafe { user_data.unpack() };
    let payload = match msg {
        Message::Request(_) => panic!(),
        Message::Response(r) => {
            let response = r.result.unwrap_or_else(|| serde_json::Value::Null);
            json!({ID: r.id, RESULT: response})
        }
        Message::Notification(_) => panic!(),
    };

    let mut args = vec![QCtest, Qequal];

    let hashmap = unsafe { Fmake_hash_table(args.len().try_into().unwrap(), args.as_mut_ptr()) };
    let payload_data = UserData::new(payload);

    LispObject::cons(hashmap, payload_data)
}

#[lisp_fn]
pub fn get_json_cached_data(map: LispCons, key: LispObject) -> LispObject {
    let hashmap = map.car();
    let string_val: LispStringRef = key.into();
    let is_hashtable: bool = unsafe { Fhash_table_p(hashmap) }.into();
    if !is_hashtable {
        wrong_type!(Qhash_table_p, hashmap);
    }

    let h = unsafe { XHASH_TABLE(hashmap) };
    let mut lisp_hash: LispObject = LispObject::from(0);
    let i = unsafe { hash_lookup(h, key, &mut lisp_hash) };
    if i < 0 {
        let u: UserData = map.cdr().into();
        let mut m: serde_json::Value = unsafe { u.unpack() };
        let utf8 = string_val.to_utf8();
        let taken = m["response"][utf8].take();
        let result = serde_to_lisp(taken);
        unsafe { hash_put(h, key, result, lisp_hash) };
        result
    } else {
        unsafe { HASH_VALUE(h, i) }
    }
}

*/

// @TODO: Instead of an option, have this return a result
// and do not invoke 'error' directly. Have serde_to_lisp match.
fn lisp_to_serde(object: LispObject, config: &JSONConfiguration) -> Option<serde_json::Value> {
    if object == config.null_obj {
        Some(serde_json::Value::Null)
    } else if object == config.false_obj {
        Some(serde_json::Value::Bool(false))
    } else if object == Qt {
        Some(serde_json::Value::Bool(true))
    } else if unsafe { INTEGERP(object) } {
        let value = unsafe { check_integer_range(object, intmax_t::MIN, intmax_t::MAX) };
        let num = serde_json::Number::from(value);
        Some(serde_json::Value::Number(num))
    } else if unsafe { FLOATP(object) } {
        let float_value = unsafe { XFLOAT_DATA(object) };
        if let Some(flt) = serde_json::Number::from_f64(float_value) {
            Some(serde_json::Value::Number(flt))
        } else {
            error!("Invalid float value {}", float_value);
        }
    } else if unsafe { STRINGP(object) } {
        let string_ref: LispStringRef = object.into();
        let utf8_string = string_ref.to_utf8();
        Some(serde_json::Value::String(utf8_string))
    } else if unsafe { VECTORP(object) } {
        let size = unsafe { ASIZE(object) };
        let mut vector: Vec<serde_json::Value> = vec![];
        for i in 0..size {
            vector.push(lisp_to_serde(unsafe { AREF(object, i) }, config)?);
        }

        Some(serde_json::Value::Array(vector))
    } else if unsafe { HASH_TABLE_P(object) } {
        let h = unsafe { XHASH_TABLE(object) };
        let size = unsafe { HASH_TABLE_SIZE(h) };
        let mut map: Map<String, serde_json::Value> = Map::new();

        for i in 0..size {
            let key = unsafe { HASH_KEY(h, i) };
            if key != Qunbound {
                let key_string: LispStringRef = key.into();
                let key_utf8 = key_string.to_utf8();
                let lisp_val = unsafe { HASH_VALUE(h, i) };
                let insert_result = map.insert(key_utf8, lisp_to_serde(lisp_val, config)?);
                if insert_result.is_some() {
                    error!("Duplicate keys are not allowed");
                }
            }
        }

        Some(serde_json::Value::Object(map))
    } else if unsafe { NILP(object) } {
        Some(serde_json::Value::Object(Map::new()))
    } else if object.is_cons() {
        let tail: LispCons = object.into();
        let iter = tail.iter_tails(LispConsEndChecks::on, LispConsCircularChecks::on);
        let is_plist = !tail.car().is_cons();
        let mut map = Map::new();
        let mut skip_cycle = false;
        let mut return_none = false;
        iter.for_each(|tail| {
            if skip_cycle {
                skip_cycle = false;
                return;
            }

            let (key, value) = if is_plist {
                let key = tail.car();
                if !tail.cdr().is_cons() {
                    return_none = true;
                    return;
                }

                let cdr_cons: LispCons = tail.cdr().into();
                // we have looked at key, and taken value, so we want to skip the inter
                // iteration
                skip_cycle = true;
                let value = cdr_cons.car();
                (key, value)
            } else {
                let pair = tail.car();
                if !pair.is_cons() {
                    return_none = true;
                    return;
                }

                let pair_value: LispCons = pair.into();
                (pair_value.car(), pair_value.cdr())
            };

            unsafe { CHECK_SYMBOL(key) };
            let key_symbol = unsafe { SYMBOL_NAME(key) };
            let key_string: LispStringRef = key_symbol.into();
            let mut key_utf8 = key_string.to_utf8();
            if is_plist && key_utf8.as_bytes()[0] == b':' && key_utf8.len() > 1 {
                key_utf8.remove(0);
            }

            // We only will add to the map if a value is not present
            // at that key
            if !map.contains_key(&key_utf8) {
                if let Some(insert_value) = lisp_to_serde(value, config) {
                    map.insert(key_utf8, insert_value);
                } else {
                    return_none = true;
                }
            }
        });

        if return_none {
            None
        } else {
            Some(serde_json::Value::Object(map))
        }
    } else {
        None
    }
}

fn serde_to_lisp(value: serde_json::Value, config: &JSONConfiguration) -> LispObject {
    match value {
        Value::Null => config.null_obj,
        Value::Bool(b) => {
            if b {
                Qt
            } else {
                config.false_obj
            }
        }
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                unsafe { make_uint(u) }
            } else if let Some(i) = n.as_i64() {
                unsafe { make_int(i) }
            } else if let Some(f) = n.as_f64() {
                unsafe { make_float(f) }
            } else {
                error!("Unable to parse Number {:?}", n);
            }
        }
        Value::String(s) => {
            let len = s.len();
            let c_content = CString::new(s).expect("Failed to convert to C string");
            unsafe { make_string_from_utf8(c_content.as_ptr(), len.try_into().unwrap()) }
        }
        Value::Array(mut v) => {
            let len = v.len();
            match config.arr {
                ArrayType::Array => {
                    let result = unsafe { make_vector(len.try_into().unwrap(), Qunbound) };
                    let len64: i64 = len.try_into().unwrap();
                    let mut i = len64 - 1;
                    while let Some(owned) = v.pop() {
                        unsafe {
                            ASET(result, i.try_into().unwrap(), serde_to_lisp(owned, config))
                        };
                        i -= 1;
                    }

                    result
                }
                ArrayType::List => {
                    let mut result = Qnil;
                    for i in (0..len).rev() {
                        result = unsafe { Fcons(serde_to_lisp(v[i].take(), config), result) };
                    }

                    result
                }
            }
        }
        Value::Object(mut map) => {
            let count = map.len();
            match config.obj {
                ObjectType::Hashtable => {
                    let mut args = vec![QCtest, Qequal, QCsize, unsafe {
                        make_fixed_natnum(count.try_into().unwrap())
                    }];

                    let hashmap = unsafe {
                        Fmake_hash_table(args.len().try_into().unwrap(), args.as_mut_ptr())
                    };

                    let h = unsafe { XHASH_TABLE(hashmap) };
                    let mut keys: Vec<String> =
                        map.keys().map(|s| s.clone()).rev().collect::<Vec<String>>();
                    while let Some(k) = keys.pop() {
                        if let Some(v) = map.remove(&k) {
                            let len = k.len();
                            let cstring = CString::new(k).expect("Failure to allocate CString");
                            let lisp_key = unsafe {
                                make_string_from_utf8(cstring.as_ptr(), len.try_into().unwrap())
                            };
                            let mut lisp_hash: LispObject = LispObject::from(0);
                            let i = unsafe { hash_lookup(h, lisp_key, &mut lisp_hash) };
                            assert!(i < 0);
                            unsafe { hash_put(h, lisp_key, serde_to_lisp(v, config), lisp_hash) };
                        } else {
                            error!("Error in deserializing json value");
                        }
                    }

                    hashmap
                }
                ObjectType::Alist => {
                    let mut result = Qnil;
                    let mut keys: Vec<String> =
                        map.keys().map(|s| s.clone()).rev().collect::<Vec<String>>();
                    while let Some(k) = keys.pop() {
                        if let Some(v) = map.remove(&k) {
                            let len = k.len();
                            let cstring = CString::new(k).expect("Failure to allocate CString");
                            let lisp_key = unsafe {
                                Fintern(
                                    make_string_from_utf8(
                                        cstring.as_ptr(),
                                        len.try_into().unwrap(),
                                    ),
                                    Qnil,
                                )
                            };
                            result =
                                unsafe { Fcons(Fcons(lisp_key, serde_to_lisp(v, config)), result) };
                        }
                    }

                    unsafe { Fnreverse(result) }
                }
                ObjectType::Plist => {
                    // @TODO this likely can be optimized by appending the
                    // : when we clone the string, followed by only looking for
                    // a slice via map.remove
                    let mut result = Qnil;
                    let mut keys: Vec<String> =
                        map.keys().map(|s| s.clone()).rev().collect::<Vec<String>>();
                    while let Some(k) = keys.pop() {
                        if let Some(v) = map.remove(&k) {
                            let mut colon_key = String::from(":");
                            colon_key.push_str(&k);
                            let len = colon_key.len();
                            let cstring =
                                CString::new(colon_key).expect("Failure to allocate CString");
                            let lisp_key = unsafe {
                                Fintern(
                                    make_string_from_utf8(
                                        cstring.as_ptr(),
                                        len.try_into().unwrap(),
                                    ),
                                    Qnil,
                                )
                            };
                            result = unsafe { Fcons(lisp_key, result) };
                            result = unsafe { Fcons(serde_to_lisp(v, config), result) };
                        }
                    }

                    unsafe { Fnreverse(result) }
                }
            }
        }
    }
}

// This function is written so that if len args == 0, it will return
// JSONConfiguration::default(). If you edit this function, ensure
// that you aware of that functionality.
fn generate_config_from_args(args: &[LispObject]) -> JSONConfiguration {
    let mut config = JSONConfiguration::default();

    if args.len() % 2 != 0 {
        wrong_type!(Qplistp, unsafe {
            Flist(
                args.len().try_into().unwrap(),
                args.as_ptr() as *mut LispObject,
            )
        });
    }

    for i in 0..args.len() {
        if i % 2 != 0 {
            continue;
        }

        let key = args[i];
        let value = args[i + 1];
        match key {
            QCobject_type => {
                config.obj = match value {
                    Qhash_table => ObjectType::Hashtable,
                    Qalist => ObjectType::Alist,
                    Qplist => ObjectType::Plist,
                    _ => error!(":object-type must be 'hash-table, 'alist, 'plist"),
                };
            }
            QCarray_type => {
                config.arr = match value {
                    Qarray => ArrayType::Array,
                    Qlist => ArrayType::List,
                    _ => error!(":array-type must be 'array, 'list"),
                };
            }
            QCnull_object => {
                config.null_obj = value;
            }
            QCfalse_object => {
                config.false_obj = value;
            }
            _ => {
                error!("Wrong type: must be :object-type, :array-type, :null-object, :false-object")
            }
        }
    }

    config
}

#[lisp_fn(min = "1")]
pub fn json_se(args: &[LispObject]) -> LispObject {
    let config = generate_config_from_args(&args[1..]);
    let value = lisp_to_serde(args[0], &config)
        .ok_or(Error::new(
            std::io::ErrorKind::Other,
            "Failure to parse json object due to construction",
        ))
        .map_err(|e| error!("Error in json serialization: {:?}", e))
        .unwrap(); // Safe because we mapped error.
    match serde_json::to_string(&value) {
        Ok(v) => {
            let len = v.len();
            let cstring = CString::new(v).expect("Failure to allocate CString");
            unsafe { make_string_from_utf8(cstring.as_ptr(), len.try_into().unwrap()) }
        }
        Err(e) => error!("Error in json serialization: {:?}", e),
    }
}

#[lisp_fn(min = "1")]
pub fn json_de(args: &[LispObject]) -> LispObject {
    let config = generate_config_from_args(&args[1..]);
    let sref: LispStringRef = args[0].into();

    match serde_json::from_str(&sref.to_utf8()) {
        Ok(value) => serde_to_lisp(value, &config),
        Err(e) => error!("Error in parsing json: {:?}", e),
    }
}

pub(crate) fn gen_ser_deser_config() -> JSONConfiguration {
    JSONConfiguration {
        null_obj: Qnil,
        ..Default::default()
    }
}

pub(crate) fn deser(
    string: &str,
    config: Option<JSONConfiguration>,
) -> std::result::Result<LispObject, serde_json::Error> {
    let val = serde_json::from_str(string)?;
    let config = config.unwrap_or_else(|| gen_ser_deser_config());
    Ok(serde_to_lisp(val, &config))
}

pub(crate) fn ser(o: LispObject) -> Result<String> {
    let config = gen_ser_deser_config();
    let value = lisp_to_serde(o, &config)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "bad"))?;
    serde_json::to_string(&value)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e)))
}

#[lisp_fn]
pub fn lsp_async_send_request(proc: LispObject, method: LispObject, params: LispObject) -> bool {
    let mut emacs_pipe = unsafe { EmacsPipe::with_process(proc) };
    let method_s: LispStringRef = method.into();
    let config = get_process_json_config(proc);
    let value = lisp_to_serde(params, &config);
    let request = Message::Request(Request::new(RequestId::from(0), method_s.to_utf8(), value));
    if let Err(e) = emacs_pipe.message_rust_worker(UserData::new(request)) {
        error!("Failed to send request to server, reason {:?}", e);
    }

    true
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
    let mut out_pipe = pipe.clone();
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

// In order to have rust generate symbols at compile time,
// I need a line of code starting with "def_lisp_sym"
// This function does not actually run any code, it should
// not be called at runtime. Doing so would actually be harmless
// as 'def_lisp_sym' generates no runtime code.
#[allow(dead_code)]
fn init_syms() {
    def_lisp_sym!(QCnull, ":null");
    def_lisp_sym!(QCfalse, ":false");
    def_lisp_sym!(QCobject_type, ":object-type");
    def_lisp_sym!(QCarray_type, ":array-type");
    def_lisp_sym!(QCnull_object, ":null-object");
    def_lisp_sym!(QCfalse_object, ":false-object");
    def_lisp_sym!(QCjson_config, ":json-config");
    def_lisp_sym!(Qalist, "alist");
    def_lisp_sym!(Qplist, "plist");
    def_lisp_sym!(Qarray, "array");
}

include!(concat!(env!("OUT_DIR"), "/parsing_exports.rs"));
