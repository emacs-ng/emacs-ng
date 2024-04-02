use codegen::with_enabled_crates;
use codegen::BuildError;
use codegen::Package;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// First we have to generate the include file for the main crate which
/// will be stored in OUT_DIR. It only contains the rust_init_syms
/// that runs the crates *_init_syms functions.
pub fn generate_include_files(packages: Vec<&Package>) -> Result<(), BuildError> {
    let out_file = PathBuf::from(env::var("OUT_DIR")?).join("c_exports.rs");
    let mut out_file = File::create(out_file)?;

    // Add main rust_init_syms function to the main c_exports file
    write!(
        out_file,
        "#[no_mangle]\npub extern \"C\" fn rust_init_syms() {{\n"
    )?;

    for package in packages {
        let crate_name = &package.name.replace('-', "_");
        // Call a crate's init_syms function in the main c_exports file
        let crate_init_syms = format!("{}::{}_init_syms();\n", crate_name, crate_name);
        write!(out_file, "{}", crate_init_syms)?
    }

    write!(out_file, "}}\n")?;
    Ok(())
}

fn main() -> Result<(), BuildError> {
    let _ = with_enabled_crates(|packages| {
        match generate_include_files(packages.clone()) {
            Err(err) => {
                eprintln!("{:?}", err);
                std::process::exit(3);
            }
            _ => {}
        };
        Ok(())
    })?;

    Ok(())
}
