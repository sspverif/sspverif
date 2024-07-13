use std::fmt::{Debug, Display};

use miette::{Diagnostic, SourceSpan};
use pest::error::ErrorVariant;
use thiserror::Error;

use crate::types::Type;

pub enum NewError {}

#[derive(Error, Diagnostic, Debug)]
#[error("syntax error: {variant}")]
#[diagnostic(code(ssbee::syntax))]
pub struct PestParseError {
    #[label("here")]
    pub at: SourceSpan,

    variant: PestErrorVariantPrinter,
}

#[derive(Debug)]
pub struct PestErrorVariantPrinter(ErrorVariant<super::Rule>);

impl Display for PestErrorVariantPrinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            ErrorVariant::ParsingError {
                positives,
                negatives,
            } => {
                writeln!(f, "(pos {positives:?}) (neg {negatives:?}) ")?;
            }
            ErrorVariant::CustomError { message } => write!(f, "{message}")?,
        }

        Ok(())
    }
}

#[derive(Error, Diagnostic, Debug)]
#[error("undefined type '{text}'")]
#[diagnostic(code(ssbee::code::undefined_type))]
pub struct UndefinedTypeError {
    #[label("this type is not defined")]
    pub at: SourceSpan,

    pub text: String,
}

#[derive(Error, Diagnostic, Debug)]
#[error("undefined identifier '{ident_name}'")]
#[diagnostic(code(ssbee::code::undefined_identifier))]
pub struct UndefinedIdentifierError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this identifier is not defined")]
    pub at: SourceSpan,

    pub ident_name: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("identifier '{ident_name}' has already been declared")]
#[diagnostic(code(ssbee::code::identifier_already_declared))]
pub struct IdentifierAlreadyDeclaredError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this identifier here")]
    pub at: SourceSpan,

    // TODO: would be nice to be also have a span for the original definition
    //       this requires keeping definition location info in the scope
    pub ident_name: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("oracle '{oracle_name}' has already been declared")]
#[diagnostic(code(ssbee::code::oracle_already_declared))]
pub struct OracleAlreadyImportedError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this oracle here")]
    pub at: SourceSpan,

    pub oracle_name: String,
}

#[derive(Error, Diagnostic, Debug)]
#[error("type mismatch: got {got:?}, expected {expected:?}")]
#[diagnostic(code(ssbee::code::type_mismatch))]
pub struct TypeMismatchError {
    #[label("this expression has the wrong type")]
    pub at: SourceSpan,

    pub expected: Type,

    pub got: Type,

    #[source_code]
    pub source_code: miette::NamedSource<String>,
}

#[derive(Error, Diagnostic, Debug)]
#[error("undefined package instance '{pkg_inst_name}'")]
#[diagnostic(code(ssbee::code::undefined_package_instance))]
pub struct UndefinedPackageInstanceError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this package instance is not defined")]
    pub at: SourceSpan,

    pub pkg_inst_name: String,
}

#[derive(Error, Diagnostic, Debug)]
#[error("undefined game instance '{game_inst_name}'")]
#[diagnostic(code(ssbee::code::undefined_game_instance))]
pub struct UndefinedGameInstanceError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this game instance is not defined")]
    pub at: SourceSpan,

    pub game_inst_name: String,
}

#[derive(Error, Diagnostic, Debug)]
#[error("undefined package '{pkg_name}'")]
#[diagnostic(code(ssbee::code::undefined_package))]
pub struct UndefinedPackageError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this package is not defined")]
    pub at: SourceSpan,

    pub pkg_name: String,
}

#[derive(Error, Diagnostic, Debug)]
#[error("undefined game '{game_name}'")]
#[diagnostic(code(ssbee::code::undefined_game))]
pub struct UndefinedGameError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this package is not defined")]
    pub at: SourceSpan,

    pub game_name: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("parameter '{param_name}' does not exist on package {pkg_name}")]
#[diagnostic(code(ssbee::code::pkg::no_such_parameter))]
pub struct NoSuchPackageParameterError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this identifier here")]
    pub at: SourceSpan,

    pub param_name: String,
    pub pkg_name: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("parameter '{param_name}' does not exist on game {game_name}")]
#[diagnostic(code(ssbee::code::game::no_such_parameter))]
pub struct NoSuchGameParameterError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this identifier here")]
    pub at: SourceSpan,

    pub param_name: String,
    pub game_name: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("type '{type_name}' does not exist ")]
#[diagnostic(code(ssbee::code::no_such_type))]
pub struct NoSuchTypeError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this type here")]
    pub at: SourceSpan,

    pub type_name: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("oracle '{oracle_name}' does not exist ")]
#[diagnostic(code(ssbee::code::no_such_oracle))]
pub struct NoSuchOracleError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this type here")]
    pub at: SourceSpan,

    pub oracle_name: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("identifiers in for loop spec don't match: {fst:?} != {snd:?}")]
#[diagnostic(code(ssbee::code::no_such_oracle))]
pub struct ForLoopIdentifersDontMatchError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this identifer...")]
    pub at_fst: SourceSpan,

    #[label("...should be the same as this")]
    pub at_snd: SourceSpan,

    pub fst: String,
    pub snd: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error(
    "package parameter '{param_name}' has been defined twice in package instance {pkg_inst_name}"
)]
#[diagnostic(code(ssbee::code::pkg_inst::duplicate_parameter_definition))]
pub struct DuplicatePackageParameterDefinitionError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this identifier here")]
    pub at: SourceSpan,

    #[label("has previously been defined here")]
    pub other: SourceSpan,

    pub param_name: String,
    pub pkg_inst_name: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("game parameter '{param_name}' has been defined twice in game instance {game_inst_name}")]
#[diagnostic(code(ssbee::code::game_inst::duplicate_parameter_definition))]
pub struct DuplicateGameParameterDefinitionError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this identifier here")]
    pub at: SourceSpan,

    #[label("has previously been defined here")]
    pub other: SourceSpan,

    pub param_name: String,
    pub game_inst_name: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("package {pkg_name} declares parameters that are not defined in package instance {pkg_inst_name}: {missing_params}")]
#[diagnostic(code(ssbee::code::pkg::missing_parameter_definition))]
pub struct MissingPackageParameterDefinitionError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this parameter definition block")]
    pub at: SourceSpan,

    pub pkg_name: String,
    pub pkg_inst_name: String,

    pub missing_params_vec: Vec<String>,
    pub missing_params: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("game {game_name} declares parameters that are not defined in game instance {game_inst_name}: {missing_params}")]
#[diagnostic(code(ssbee::code::game::missing_parameter_definition))]
pub struct MissingGameParameterDefinitionError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this parameter definition block")]
    pub at: SourceSpan,

    pub game_name: String,
    pub game_inst_name: String,

    pub missing_params_vec: Vec<String>,
    pub missing_params: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("use of undefined assumption {assumption_name}")]
#[diagnostic(code(ssbee::code::undefined_assumption))]
pub struct UndefinedAssumptionError {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this assumption here")]
    pub at: SourceSpan,

    pub assumption_name: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("the first package instance name in an assumption package mapping block header must be from the assumption, but isn't")]
#[diagnostic(code(ssbee::code::proof::reduction::assumption_mapping::no_assumption_game_instance))]
#[help("the mapping maps the package instances of the assumption game and the model game. Therefore the first needs to be an assumption game instance, while the other needs to be a model game instance. Game instance names from the assumption are {assumption_left_game_instance_name} and {assumption_right_game_instance_name}.")]
pub struct AssumptionMappingLeftGameInstanceIsNotFromAssumption {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this game instance name")]
    pub at: SourceSpan,

    pub game_instance_name: String,

    pub assumption_left_game_instance_name: String,
    pub assumption_right_game_instance_name: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("the second package instance name in an assumption package mapping block header must be from the model (i.e. not from the assumption), but isn't")]
#[diagnostic(code(ssbee::code::proof::reduction::assumption_mapping::no_assumption_game_instance))]
#[help("the mapping maps the package instances of the assumption game and the model game. Therefore the second needs to be a model game instance. Game instance names from the assumption are {model_left_game_instance_name} and {model_right_game_instance_name}.")]
pub struct AssumptionMappingRightGameInstanceIsFromAssumption {
    #[source_code]
    pub source_code: miette::NamedSource<String>,

    #[label("this game instance name")]
    pub at: SourceSpan,

    pub game_instance_name: String,

    pub model_left_game_instance_name: String,
    pub model_right_game_instance_name: String,
}
