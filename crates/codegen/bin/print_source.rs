use cargo_metadata::Package;
use codegen::packages_source;
use codegen::with_enabled_crates_all;
use codegen::BuildError;
use std::io::Write;
use std::path::PathBuf;

/// First we have to generate the include file for the main crate which
/// will be stored in OUT_DIR. It only contains the rust_init_syms
/// that runs the crates *_init_syms functions.
pub fn generate_source_list(packages: Vec<&Package>) -> Result<(), BuildError> {
    let abs_top_srcdir = std::env::var("ABS_TOP_SRCDIR")
        .map(|dir| PathBuf::from(dir))
        .ok();
    for file in packages_source(packages) {
        let file = match abs_top_srcdir {
            Some(ref base) => {
                let path = file
                    .as_path()
                    .strip_prefix(base)
                    .map_err(|e| {
                        anyhow::format_err!("error: {e:?}, file: {file:?}, base: {base:?}")
                    })
                    .unwrap()
                    .to_path_buf();
                PathBuf::from("..").join(path)
            }
            None => file,
        };
        write!(std::io::stdout(), "{} ", file.display())?;
    }
    Ok(())
}

// Each feature are enabled by setting CARGO_FEATURE_*=1,
// this is already the case when invoke with_enabled_crates from
// build script. For manually run this command, We have specify these
// envs
fn main() -> Result<(), BuildError> {
    for arg in std::env::args().skip(1) {
        let key = format!("CARGO_FEATURE_{}", arg.replace("-", "_").to_uppercase());
        std::env::set_var(key, "1");
    }
    let _ = with_enabled_crates_all(|packages| {
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
