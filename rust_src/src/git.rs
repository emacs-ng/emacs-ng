use std::path::Path;

use git2::Repository;

use lisp_macros::lisp_fn;

use lisp::multibyte::LispStringRef;

#[lisp_fn]
pub fn git_init(path: LispStringRef) -> LispStringRef {
    match Repository::init(Path::new(path.to_utf8().as_str())) {
        Ok(repo) => path,
        Err(e) => {
            error!("Error initializing repository {:?}", e);
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/git_exports.rs"));
