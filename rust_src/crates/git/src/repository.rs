use std::path::Path;

use git2::Repository;

use lisp_macros::lisp_fn;

use emacs::multibyte::LispStringRef;

#[lisp_fn]
pub fn git_init(path: LispStringRef) -> LispStringRef {
    match Repository::init(Path::new(path.to_utf8().as_str())) {
        Ok(_repo) => path,
        Err(e) => {
            error!("Error initializing repository {:?}", e);
        }
    }
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/out/repository_exports.rs"
));
