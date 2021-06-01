// Include the main c_exports file that holds the main rust_init_syms.
// This function calls the other crates init_syms functions which contain
// the generated bindings.
#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/c_exports.rs"));
