use std::collections::HashMap;

pub mod filesystem;
pub mod project;

use std::io::Error as IOError;

use thiserror::Error;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CompositionSpec {
    composition: String,
    params: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct ProofFile {
    params: Vec<String>,
    left: CompositionSpec,
    right: CompositionSpec,
}

#[derive(Debug, Error)]
pub enum Error {
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
		#[error("io error: {0}")]
		IOError(#[from] IOError),
		#[error("parse error: {0} at {1:?} / {2:?}")]
		PestParseError(String, pest::error::InputLocation, pest::error::LineColLocation),
		#[error("toml parse error: {0}")]
		TomlParseError(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl<R: pest::RuleType> From<pest::error::Error<R>> for Error {
	fn from(e: pest::error::Error<R>) -> Error {
		Error::PestParseError(
			format!("{:?}", e.variant),
			e.location,
			e.line_col,
		)
	}
}
