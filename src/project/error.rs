use crate::{parser, transforms::typecheck::TypeCheckError, util::prover_process::ProverResponse};
use miette::Diagnostic;
use std::{io::Error as IOError, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("error showing equivalence: {0}")]
    EquivalenceError(#[from] crate::gamehops::equivalence::error::Error),
    #[error("proof already exists: {0}")]
    ProofExists(String),
    #[error("consistency check failed with {0}")]
    ProofCheck(String),
    #[error("apparently there is a missing composition {0}")]
    CompositionMissing(String),
    #[error("undefined game {0} referenced here: {1}")]
    UndefinedGame(String, String),
    #[error("undefined assumption {0} referenced here: {1}")]
    UndefinedAssumption(String, String),
    #[error("undefined mapping {0} referenced here: {1}")]
    UndefinedMapping(String, String),
    #[error("io error: {0}")]
    IOError(#[from] IOError),
    #[error("parse error: {0} at {1:?} / {2:?}")]
    PestParseError(
        String,
        pest::error::InputLocation,
        pest::error::LineColLocation,
    ),
    #[error("ssp parse error: {0}")]
    SspParseError(#[from] crate::parser::error::SpanError),
    #[error("toml parse error: {0}")]
    TomlParseError(#[from] toml_edit::de::Error),
    #[error("proof tree validation failed: {0}")]
    ProofTreeValidationError(String),
    #[error("package {0} defined in both {1} and {2}")]
    RedefinedPackage(String, String, String),
    #[error("composisitions {0} and {1} have mismatching const/param {2}")]
    CompositionParamMismatch(String, String, String),
    #[error("couldn't open invariant file at {invariant_path} of game hop {hop_index}, going from {left} to {right} ")]
    EquivalenceInvariantFileNotFound {
        hop_index: usize,
        left: String,
        right: String,
        invariant_path: PathBuf,
    },
    #[error("type error: {0}")]
    TypecheckError(#[from] TypeCheckError),
    #[error("error parsing utf-8: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("error in interaction with child process: {0}")]
    ChildProcessInteractionError(#[from] expectrl::Error),
    #[error("error interactiv with prover process: {0}")]
    ProcessError(#[from] crate::util::process::Error),
    #[error("error interactiv with prover process: {0}")]
    ProverProcessError(#[from] crate::util::prover_process::Error),
    #[error("unexpected prover response {0}, expected {1}")]
    UnexpectedProverResponseError(ProverResponse, ProverResponse),
    //#[error("got a formatting error")]
    //FmtError(#[from] std::fmt::Error),
    #[diagnostic(transparent)]
    #[error(transparent)]
    PackageParse(parser::package::ParsePackageError),
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
