extern crate ng_bindgen;

use ng_bindgen::{generate_crate_exports, BuildError};

fn main() {
    for varname in ["EMACS_CFLAGS", "SRC_HASH"].iter() {
        println!("cargo:rerun-if-env-changed={}", varname);
    }
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Err(e) = generate_crate_exports(&path) {
        match e {
            BuildError::IOError(msg) => {
                eprintln!("{}", msg);
                std::process::exit(3);
            }
            BuildError::Lint(msg) => {
                msg.fail(1);
            }
        }
    }
}
