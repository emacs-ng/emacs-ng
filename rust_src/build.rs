extern crate ng_bindgen;

use ng_bindgen::{env_var, generate_include_files, BuildError};

fn main() {
    // TODO watch relevent files to re rerun, rs files under crates?

    // generates include files for the crates from the directory "crates"
    let crates_dir: std::path::PathBuf = [&env_var("CARGO_MANIFEST_DIR"), "rust_src/crates"]
        .iter()
        .collect();

    if let Err(e) = generate_include_files(crates_dir) {
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
