use cargo_files_core::{get_target_files, get_targets, Error};
use std::path::PathBuf;

fn main() -> Result<(), Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("Cargo.toml");
    let targets = get_targets(Some(&path))?;
    for target in targets {
        println!("target {target:?}");
        let files = get_target_files(&target)?;
        for file in files {
            println!("{}", file.display());
        }
    }

    Ok(())
}
