use std::env;

use std::num::ParseIntError;

use super::LintMsg;
use std::io;

#[derive(Debug)]
pub enum BuildError {
    VarError(env::VarError),
    IOError(io::Error),
    Metadata(cargo_metadata::Error),
    Parse(ParseIntError),
    Lint(LintMsg),
}

impl From<env::VarError> for BuildError {
    fn from(e: env::VarError) -> Self {
        BuildError::VarError(e)
    }
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

impl From<LintMsg> for BuildError {
    fn from(e: LintMsg) -> Self {
        BuildError::Lint(e)
    }
}
