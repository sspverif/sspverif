use std::fmt::{self, Display};
use std::io::ErrorKind;

use miette::SourceSpan;

use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::statement::FilePosition;
use crate::types::Type;

#[derive(Debug, Clone)]
pub enum ErrorLocation {
    FilePosition(FilePosition),
    SourceSpan(SourceSpan),
    Unknown,
}

#[derive(Debug)]
pub struct ScopeError(pub Identifier, pub super::Type);

#[derive(Debug, Clone)]
pub struct TypeError(pub String);

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid type")
    }
}

#[derive(Debug)]
pub enum TypeCheckError {
    MisplacedStatement(ErrorLocation, String),
    TypeMismatch(
        ErrorLocation,
        String,             // message
        Option<Expression>, // found expression, if applicable
        Type,               // found
        Type,               // expected
    ),
    TypeMismatchVague(
        // This is used if multiple types would be valid and we can't just put one there
        ErrorLocation,
        String,             // message
        Option<Expression>, // found expression, if applicable
        Type,               // found
    ),
    Undefined(ErrorLocation, String, Identifier),
    ShouldBeMaybe(ErrorLocation, String, Expression, Type),
    Scope(ErrorLocation, ScopeError),
    Type(TypeError),
    TypeCheck(String),
}

impl TypeCheckError {
    pub fn new_scope_error(err: ScopeError, file_pos: &SourceSpan) -> Self {
        Self::Scope(ErrorLocation::SourceSpan(file_pos.clone()), err)
    }
}

impl From<TypeError> for TypeCheckError {
    fn from(error: TypeError) -> Self {
        TypeCheckError::Type(error)
    }
}

impl Display for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TypeCheckError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl From<TypeCheckError> for std::io::Error {
    fn from(e: TypeCheckError) -> Self {
        // TODO is this errorkind the right one in all instances?
        std::io::Error::new(ErrorKind::InvalidData, e)
    }
}
