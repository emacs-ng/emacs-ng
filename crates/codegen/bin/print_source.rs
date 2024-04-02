use cargo_metadata::Package;
use codegen::packages_source;
use codegen::with_enabled_crates;
use codegen::BuildError;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// First we have to generate the include file for the main crate which
/// will be stored in OUT_DIR. It only contains the rust_init_syms
/// that runs the crates *_init_syms functions.
pub fn generate_source_list(packages: Vec<&Package>) -> Result<(), BuildError> {
    let out_file = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("..")
        .join("..")
        .join("src")
        .join("libemacsng_source");
    let mut out_file = File::create(out_file)?;

    for file in packages_source(packages) {
        write!(out_file, "{} ", file.display())?;
    }
    Ok(())
}

// Each feature are enabled by setting CARGO_FEATURE_*=1,
// this is already the case when invoke with_enabled_crates from
// build script. For manually run this command, We have specify these
// envs
fn main() -> Result<(), BuildError> {
    let _ = with_enabled_crates(|packages| {
        match generate_source_list(packages.clone()) {
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
