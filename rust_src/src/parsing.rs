use crate::lisp::LispObject;
use crate::ng_async::{EmacsPipe, UserData};
use remacs_macros::{async_stream, lisp_fn};
use serde_json::{Result, Value};

pub struct JsonMessage {
    pipe: EmacsPipe,
    json: Value,
}

#[async_stream]
pub async fn async_parse_json(s: UserData) -> UserData {
    let mut unpacked: JsonMessage = unsafe { s.unpack() };
    // The key for some added performance to do perform the
    // object -> json string encoding offthread. This allows
    // us to gain back cycles.
    unpacked
        .pipe
        .write_external_process(&unpacked.json.to_string());
    UserData::default()
}

// proc is our LSP server process lisp object
// pipe is our async thread pipe lisp object
// message is a lisp object that will be json-encoded
#[lisp_fn]
pub fn async_message_json(pipe: LispObject, proc: LispObject, _message: LispObject) -> LispObject {
    let transfer_pipe = unsafe { EmacsPipe::with_process(proc) };
    let jsonm = JsonMessage {
        pipe: transfer_pipe,
        json: json!(null), // lisp_to_json(message)
    };

    let data = UserData::new(jsonm);
    crate::ng_async::Fasync_send_message(pipe, data.into())
}

// Option 1: Write functions that look something like whats defined below
// Option 2: Compile emacs-ng with json enabled, and use the
// emacs functions defined in json.c, pass json_t* around instead of
// serde_json::Value objects

// fn lisp_to_json_non_primative(o: LispObject) -> serde_json::Value {
//     if VECTORP(o) {
// 	let size = ASIZE(o);
// 	for i in 0..size {
// 	    let json = lisp_to_json(AREF(o, i));
// 	    // append json
// 	}

// 	json!(null)
//     } else if HASH_TABLE_P(o) {
// 	json!(null)
//     } else if NILP (o) {
// 	json!(null)
//     } else if CONSP(o) {
// 	json!(null)
//     } else {
// 	wrong_type!(Qjson_value_p, o);
//     }
// }

// fn lisp_to_json(o: LispObject) -> serde_json::Value {
//     match o {
// 	QCnull => { json!(null) },
// 	QCfalse => { json!(false) },
// 	Qt => { json!(true) },
// 	_ => {
// 	    if INTEGERP(o) {
// 		json!(XINT(o))
// 	    } else if FLOATP(o) {
// 		json!(XFLOAT_DATA(o))
// 	    } else if STRINGP(o) {
// 		json!("TODO")
// 	    } else {
// 		lisp_to_json_non_primative(o)
// 	    }
// 	}
//     }
// }

include!(concat!(env!("OUT_DIR"), "/parsing_exports.rs"));
