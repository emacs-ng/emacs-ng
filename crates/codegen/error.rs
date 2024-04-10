use std::env;

use std::num::ParseIntError;

use super::LintMsg;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Var error {var:?}: {error:?}")]
    VarError { var: String, error: env::VarError },
    #[error("{0}")]
    IOError(io::Error),
    #[error("{0}")]
    Metadata(cargo_metadata::Error),
    #[error("{0}")]
    CargoFiles(cargo_files_core::Error),
    #[error("{0}")]
    Parse(ParseIntError),
    #[error("{0:?}")]
    Lint(LintMsg),
}

impl From<io::Error> for BuildError {
    fn from(e: io::Error) -> Self {
        BuildError::IOError(e)
    }
}
impl From<cargo_metadata::Error> for BuildError {
    fn from(e: cargo_metadata::Error) -> Self {
        BuildError::Metadata(e)
    }
}
impl From<ParseIntError> for BuildError {
    fn from(e: ParseIntError) -> Self {
        BuildError::Parse(e)
    }
}

impl From<cargo_files_core::Error> for BuildError {
    fn from(e: cargo_files_core::Error) -> Self {
        BuildError::CargoFiles(e)
    }
}

impl From<LintMsg> for BuildError {
    fn from(e: LintMsg) -> Self {
        BuildError::Lint(e)
    }
}
