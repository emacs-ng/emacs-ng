#[cfg(debug_assertions)]
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
#[macro_use]
extern crate emacs;

use emacs::bindings::{main1, terminate_due_to_signal, will_dump_p};

// Include the main c_exports file that holds the main rust_init_syms.
// This function calls the other crates init_syms functions which contain
// the generated bindings.
#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));

#[no_mangle]
#[allow(unused_doc_comments)]
pub extern "C" fn main(argc: ::libc::c_int, argv: *mut *mut ::libc::c_char) -> ::libc::c_int {
    unsafe {
        if will_dump_p() {
            return main1(argc, argv);
        }
    }

    // install global collector configured based on RUST_LOG env var.
    #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("EMACSNG_LOG"))
        .init();

    log::trace!("Emacs NG");

    // tokio::spawn(async move {
    unsafe { main1(argc, argv) };
    // });

    // emacs abort
    unsafe { terminate_due_to_signal(libc::SIGABRT, 40) };
    0
}
