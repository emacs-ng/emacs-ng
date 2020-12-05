use crate::lisp::LispObject;
use crate::multibyte::LispStringRef;
use crate::ng_async::{EmacsPipe, PipeDataOption, UserData};
use crate::lists::{LispCons, LispConsEndChecks, LispConsCircularChecks};
use lsp_server::Message;
use lsp_server::{RequestId, Response};
use remacs_macros::lisp_fn;
use serde_json::{Value, map::Map};
use std::io::{BufReader, BufWriter, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::ffi::CString;
use std::convert::TryInto;

use crate::remacs_sys::{
    QCfalse, QCnull, check_integer_range, intmax_t, Qt, Qnil, INTEGERP, FLOATP,
    XFLOAT_DATA, STRINGP, HASH_TABLE_P, VECTORP, ASIZE, AREF, ASET,
    XHASH_TABLE, HASH_TABLE_SIZE, NILP, HASH_KEY, Qunbound,
    HASH_VALUE, CONSP, XCAR, XCDR, SYMBOL_NAME, make_uint, make_int,
    make_float, make_string_from_utf8, make_vector, Ffuncall, QCsize, Qequal,
    make_fixed_natnum, QCtest, intern_c_string, hash_lookup, hash_put,
    CHECK_SYMBOL,
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

	list_args.iter_cars(LispConsEndChecks::on, LispConsCircularChecks::on)
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
pub fn lsp_handler(_proc: LispObject, data: LispObject) -> bool {
    let user_data: UserData = data.into();

    // Case 1: Convert mesasge to lisp object
    // Returns Lisp Object

    // Case 2: Convert lazily
    // Returns map
    // {lisp_objs: lisp_hash_table<lisp_object, lisp_object>,  user_data: ...}
    // logic -> check the lisp_obj map for the key, if its not there
    // fall back to user_data, create the key

    true
}

fn lisp_to_serde(object: LispObject) -> serde_json::Value {
    if object == QCnull {
	serde_json::Value::Null
    } else if object == QCfalse {
	serde_json::Value::Bool(false)
    } else if object == Qt {
	serde_json::Value::Bool(true)
    } else if unsafe { INTEGERP (object) } {
	let value = unsafe { check_integer_range(object,
						 intmax_t::MIN,
						 intmax_t::MAX) };
	let num = serde_json::Number::from(value);
	serde_json::Value::Number(num)
    } else if unsafe { FLOATP(object) } {
	let float_value = unsafe { XFLOAT_DATA(object) };
	if let Some(flt) = serde_json::Number::from_f64(float_value) {
	    serde_json::Value::Number(flt)
	} else {
	    error!("Invalid float value {}", float_value);
	}
    } else if unsafe { STRINGP (object) } {
	let string_ref: LispStringRef = object.into();
	let utf8_string = string_ref.to_utf8();
	serde_json::Value::String(utf8_string)
    } else if unsafe { VECTORP (object) } {
	let size = unsafe { ASIZE (object) };
	let mut vector: Vec<serde_json::Value> = vec![];
	for i in 0..size {
	    vector.push(lisp_to_serde(unsafe { AREF(object, i) }));
	}

	serde_json::Value::Array(vector)
    } else if unsafe { HASH_TABLE_P (object) } {
	let h = unsafe { XHASH_TABLE (object) };
	let size = unsafe { HASH_TABLE_SIZE(h) };
	let mut map: Map<String, serde_json::Value> = Map::new();

	for i in 0..size {
	    let key = unsafe { HASH_KEY(h, i) };
	    if key != Qunbound {
		let key_string: LispStringRef = key.into();
		let key_utf8 = key_string.to_utf8();
		let lisp_val = unsafe { HASH_VALUE(h, i) };
		let insert_result = map.insert(key_utf8, lisp_to_serde(lisp_val));
		if insert_result.is_none() {
		    error!("Duplicate keys are not allowed");
		}
	    }
	}

	serde_json::Value::Object(map)
    } else if unsafe { NILP (object) } {
	serde_json::Value::Object(Map::new())
    } else if object.is_cons() {
	// // @TODO revisit the 'cons' case
	// // I'm leaving this for now, but we need to handle "symbols"
	// let tail: LispCons = object.into();
	// let mut iter = tail.iter_cars(LispConsEndChecks::on, LispConsCircularChecks::on);
	// iter.map(|car| lisp_to_serde(car)).collect()

	let tail: LispCons = object.into();
	let mut iter = tail.iter_tails(LispConsEndChecks::on, LispConsCircularChecks::on);
	let is_plist = !tail.car().is_cons();
	let mut map = Map::new();
	let mut skip_cycle = false;
	iter.for_each(|tail| {
	    if skip_cycle {
		skip_cycle = false;
		return;
	    }

	    let pair = if is_plist {
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

	    let key = pair.0;
	    let value = pair.1;

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
fn serde_to_lisp(value: serde_json::Value) -> LispObject {
    match value {
	Value::Null => QCnull,
	Value::Bool(b) => if b { Qt } else { QCfalse },
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
	},
	Value::String(s) => {
	    let len = s.len();
	    let c_content = CString::new(s).expect("Failed to convert to C string");
	    unsafe { make_string_from_utf8(c_content.as_ptr(), len.try_into().unwrap()) }
	},
	Value::Array(mut v) => {
	    let len = v.len();
	    let result = unsafe { make_vector(len.try_into().unwrap(), Qunbound) };
	    let mut i = len - 1;
	    while let Some(owned) = v.pop() {
		unsafe { ASET(result, i.try_into().unwrap(), serde_to_lisp(owned)) };
		i -= 1;
	    }

	    result
	},
	Value::Object(mut map) => {
	    let count = map.len();
	    let c_string = CString::new("make-hash-table").expect("Failed to allocate hash string");
	    let mut args = vec![unsafe { intern_c_string(c_string.as_ptr()) },
				QCtest,
				Qequal,
				QCsize,
				unsafe { make_fixed_natnum(count.try_into().unwrap()) }];

	    let hashmap = unsafe { Ffuncall(args.len().try_into().unwrap(),
					    args.as_mut_ptr()) };

	    let h = unsafe { XHASH_TABLE(hashmap) };
	    let mut keys: Vec<String> = map.keys().map(|s| s.clone()).collect::<Vec<String>>();
	    while let Some(k) = keys.pop() {
		if let Some(v) = map.remove(&k) {
		    let len = k.len();
		    let cstring = CString::new(k).expect("Failure to allocate CString");
		    let lisp_key = unsafe { make_string_from_utf8(cstring.as_ptr(),
								  len.try_into().unwrap()) };
		    let mut lisp_hash: LispObject = LispObject::from(0);
		    let i = unsafe { hash_lookup(h, lisp_key, &mut lisp_hash) };
		    debug_assert!(i < 0); // This should be an eassert
		    unsafe { hash_put(h, lisp_key, serde_to_lisp(v), lisp_hash) };
		} else {
		    error!("Error in deserializing json value");
		}
	    }

	    hashmap
	},
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
	},
	Err(e) => error!("Error in json serialization: {:?}", e)
    }
}

#[lisp_fn]
pub fn json_de(obj: LispObject) -> LispObject {
    let sref: LispStringRef = obj.into();

    match serde_json::from_str(&sref.to_utf8()) {
	Ok(value) => serde_to_lisp(value),
	Err(e) => error!("Error in parsing json: {:?}", e)
    }
}


#[lisp_fn]
pub fn lsp_send_message(proc: LispObject, msg: LispObject) -> bool {
    let mut emacs_pipe = unsafe { EmacsPipe::with_process(proc) };
    let value = lisp_to_serde(msg);
    emacs_pipe.message_rust_worker(UserData::new(value)).unwrap();
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
	    let value: serde_json::Value = unsafe { msg.unpack() };
	    let message = serde_json::to_string(&value).unwrap();
            write!(&mut stdout_writer, "{}", message);
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
