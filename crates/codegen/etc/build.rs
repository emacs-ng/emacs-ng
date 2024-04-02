extern crate codegen;

use codegen::generate_crate_exports;
use codegen::BuildError;

fn main() -> Result<(), BuildError> {
    generate_crate_exports()?;
    Ok(())
}
