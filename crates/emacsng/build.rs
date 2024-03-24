extern crate codegen;

use codegen::env_var;
use codegen::generate_include_files;
use codegen::BuildError;

fn main() {
    // TODO watch relevent files to re rerun, rs files under crates?
    println!("cargo:rerun-if-changed=build.rs");

    // generates include files for the crates from the directory "crates"
    let crates_dir: std::path::PathBuf = [&env_var("CARGO_MANIFEST_DIR"), ".."].iter().collect();

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
