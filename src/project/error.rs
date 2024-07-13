use crate::parser;
use miette::Diagnostic;
use std::io::Error as IOError;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("error showing equivalence: {0}")]
    EquivalenceError(#[from] crate::gamehops::equivalence::error::Error),
    #[error("consistency check failed with {0}")]
    ProofCheck(String),
    #[error("io error: {0}")]
    IOError(#[from] IOError),
    #[error("package {0} defined in both {1} and {2}")]
    RedefinedPackage(String, String, String),
    #[error("error parsing utf-8: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("error in interaction with child process: {0}")]
    ChildProcessInteractionError(#[from] expectrl::Error),
    #[error("error interactiv with prover process: {0}")]
    ProcessError(#[from] crate::util::process::Error),
    #[error("error interactiv with prover process: {0}")]
    ProverProcessError(#[from] crate::util::prover_process::Error),
    //#[error("got a formatting error")]
    //FmtError(#[from] std::fmt::Error),

    // confirmed needed errors are below:
    #[error("syntax error: {0} at {1:?} / {2:?}")]
    PestParseError(
        String,
        pest::error::InputLocation,
        pest::error::LineColLocation,
    ),

    #[diagnostic(transparent)]
    #[error(transparent)]
    ParsePackage(#[from] parser::package::ParsePackageError),
    #[diagnostic(transparent)]
    #[error(transparent)]
    ParseGame(#[from] parser::composition::ParseGameError),
    #[diagnostic(transparent)]
    #[error(transparent)]
    ParseProof(#[from] parser::proof::ParseProofError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl<R: pest::RuleType> From<pest::error::Error<R>> for Error {
    fn from(e: pest::error::Error<R>) -> Error {
        Error::PestParseError(format!("{:?}", e.variant), e.location, e.line_col)
    }
}

impl<'a, R: pest::RuleType> From<(&'a str, pest::error::Error<R>)> for Error {
    fn from(e: (&'a str, pest::error::Error<R>)) -> Error {
        let (filename, e) = e;
        Error::PestParseError(
            format!("{:?} in {filename}", e.variant),
            e.location,
            e.line_col,
        )
    }
}
