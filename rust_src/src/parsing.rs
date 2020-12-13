use crate::lisp::LispObject;
use crate::lists::{LispCons, LispConsCircularChecks, LispConsEndChecks};
use crate::multibyte::LispStringRef;
use crate::ng_async::{EmacsPipe, PipeDataOption, UserData};
use lsp_server::{Message, Request, RequestId};
use remacs_macros::lisp_fn;
use serde_json::{map::Map, Value};
use std::convert::TryInto;
use std::ffi::CString;
use std::io::{BufReader, BufWriter};
use std::process::{Command, Stdio};
use std::thread;

use crate::remacs_sys::{
    check_integer_range, hash_lookup, hash_put, intmax_t, make_fixed_natnum, make_float, make_int,
    make_string_from_utf8, make_uint, make_vector, Fhash_table_p, Fmake_hash_table, QCfalse,
    QCnull, QCsize, QCtest, Qequal, Qhash_table_p, Qt, Qunbound, AREF, ASET, ASIZE, CHECK_SYMBOL,
    FLOATP, HASH_KEY, HASH_TABLE_P, HASH_TABLE_SIZE, HASH_VALUE, INTEGERP, NILP, STRINGP,
    SYMBOL_NAME, VECTORP, XFLOAT_DATA, XHASH_TABLE,
};

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

    async_create_process(command_string, args_vec, emacs_pipe);
    proc
}

#[lisp_fn]
pub fn lsp_handler(_proc: LispObject, data: LispObject) -> LispObject {
    let user_data: UserData = data.into();
    let msg: Message = unsafe { user_data.unpack() };
    match msg {
        Message::Request(_) => panic!(),
        Message::Response(r) => {
            let response = r.result.unwrap_or_else(|| serde_json::Value::Null);
            serde_to_lisp(json!({
            "id": r.id,
            "response": response,
            }))
        }
        Message::Notification(_) => panic!(),
    }
}

#[lisp_fn]
pub fn lsp_lazy_handler(_proc: LispObject, data: LispObject) -> LispObject {
    let user_data: UserData = data.into();
    let msg: Message = unsafe { user_data.unpack() };
    let payload = match msg {
        Message::Request(_) => panic!(),
        Message::Response(r) => {
            let response = r.result.unwrap_or_else(|| serde_json::Value::Null);
            json!({"id": r.id, "response": response})
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

fn lisp_to_serde(object: LispObject) -> serde_json::Value {
    if object == QCnull {
        serde_json::Value::Null
    } else if object == QCfalse {
        serde_json::Value::Bool(false)
    } else if object == Qt {
        serde_json::Value::Bool(true)
    } else if unsafe { INTEGERP(object) } {
        let value = unsafe { check_integer_range(object, intmax_t::MIN, intmax_t::MAX) };
        let num = serde_json::Number::from(value);
        serde_json::Value::Number(num)
    } else if unsafe { FLOATP(object) } {
        let float_value = unsafe { XFLOAT_DATA(object) };
        if let Some(flt) = serde_json::Number::from_f64(float_value) {
            serde_json::Value::Number(flt)
        } else {
            error!("Invalid float value {}", float_value);
        }
    } else if unsafe { STRINGP(object) } {
        let string_ref: LispStringRef = object.into();
        let utf8_string = string_ref.to_utf8();
        serde_json::Value::String(utf8_string)
    } else if unsafe { VECTORP(object) } {
        let size = unsafe { ASIZE(object) };
        let mut vector: Vec<serde_json::Value> = vec![];
        for i in 0..size {
            vector.push(lisp_to_serde(unsafe { AREF(object, i) }));
        }

        serde_json::Value::Array(vector)
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
                let insert_result = map.insert(key_utf8, lisp_to_serde(lisp_val));
                if insert_result.is_some() {
                    error!("Duplicate keys are not allowed");
                }
            }
        }

        serde_json::Value::Object(map)
    } else if unsafe { NILP(object) } {
        serde_json::Value::Object(Map::new())
    } else if object.is_cons() {
        let tail: LispCons = object.into();
        let iter = tail.iter_tails(LispConsEndChecks::on, LispConsCircularChecks::on);
        let is_plist = !tail.car().is_cons();
        let mut map = Map::new();
        let mut skip_cycle = false;
        iter.for_each(|tail| {
            if skip_cycle {
                skip_cycle = false;
                return;
            }

            let (key, value) = if is_plist {
                let key = tail.car();
                let cdr_cons: LispCons = tail.cdr().into();
                // we have looked at key, and taken value, so we want to skip the inter
                // iteration
                skip_cycle = true;
                let value = cdr_cons.car();
                (key, value)
            } else {
                let pair = tail.car();
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
                map.insert(key_utf8, lisp_to_serde(value));
            }
        });

        serde_json::Value::Object(map)
    } else {
        error!("Wrong Argument Type");
    }
}

// @TODO -> this should match 'json-serialize', meaning that we need
// a config for what the NULL and FALSE values are, and
// we need cases for how the user wants to convert
// hashmaps and arrays
// This is currently implemented to mirror the default behavior of
// json-serialize
fn serde_to_lisp(value: serde_json::Value) -> LispObject {
    match value {
        Value::Null => QCnull,
        Value::Bool(b) => {
            if b {
                Qt
            } else {
                QCfalse
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
            let result = unsafe { make_vector(len.try_into().unwrap(), Qunbound) };
            let mut i = len - 1;
            while let Some(owned) = v.pop() {
                unsafe { ASET(result, i.try_into().unwrap(), serde_to_lisp(owned)) };
                i -= 1;
            }

            result
        }
        Value::Object(mut map) => {
            let count = map.len();
            let mut args = vec![QCtest, Qequal, QCsize, unsafe {
                make_fixed_natnum(count.try_into().unwrap())
            }];

            let hashmap =
                unsafe { Fmake_hash_table(args.len().try_into().unwrap(), args.as_mut_ptr()) };

            let h = unsafe { XHASH_TABLE(hashmap) };
            let mut keys: Vec<String> = map.keys().map(|s| s.clone()).collect::<Vec<String>>();
            while let Some(k) = keys.pop() {
                if let Some(v) = map.remove(&k) {
                    let len = k.len();
                    let cstring = CString::new(k).expect("Failure to allocate CString");
                    let lisp_key =
                        unsafe { make_string_from_utf8(cstring.as_ptr(), len.try_into().unwrap()) };
                    let mut lisp_hash: LispObject = LispObject::from(0);
                    let i = unsafe { hash_lookup(h, lisp_key, &mut lisp_hash) };
                    assert!(i < 0);
                    unsafe { hash_put(h, lisp_key, serde_to_lisp(v), lisp_hash) };
                } else {
                    error!("Error in deserializing json value");
                }
            }

            hashmap
        }
    }
}

#[lisp_fn]
pub fn json_se(obj: LispObject) -> LispObject {
    let value = lisp_to_serde(obj);
    match serde_json::to_string(&value) {
        Ok(v) => {
            let len = v.len();
            let cstring = CString::new(v).expect("Failure to allocate CString");
            unsafe { make_string_from_utf8(cstring.as_ptr(), len.try_into().unwrap()) }
        }
        Err(e) => error!("Error in json serialization: {:?}", e),
    }
}

#[lisp_fn]
pub fn json_de(obj: LispObject) -> LispObject {
    let sref: LispStringRef = obj.into();

    match serde_json::from_str(&sref.to_utf8()) {
        Ok(value) => serde_to_lisp(value),
        Err(e) => error!("Error in parsing json: {:?}", e),
    }
}

#[lisp_fn]
pub fn lsp_send_request(proc: LispObject, method: LispObject, params: LispObject) -> bool {
    let mut emacs_pipe = unsafe { EmacsPipe::with_process(proc) };
    let method_s: LispStringRef = method.into();
    let value = lisp_to_serde(params);
    let request = Message::Request(Request::new(RequestId::from(0), method_s.to_utf8(), value));
    emacs_pipe
        .message_rust_worker(UserData::new(request))
        .unwrap();
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
            let value: Message = unsafe { msg.unpack() };
            value.write(&mut stdout_writer);
        }
    });

    let mut out = process.stdout;
    let mut out_pipe = pipe.clone();
    thread::spawn(move || {
        let mut stdout_reader = BufReader::new(out.as_mut().unwrap());
        // @TODO better error handling
        while let Some(msg) = Message::read(&mut stdout_reader).unwrap() {
            out_pipe.message_lisp(UserData::new(msg));
        }
    });
}

include!(concat!(env!("OUT_DIR"), "/parsing_exports.rs"));
