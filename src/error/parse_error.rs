use std::fmt;
use serde::de;
use super::ValidationError;

#[derive(Debug)]
pub enum ParseError {
    /// Validation failed
    Validation(ValidationError),
    /// Deserialization failed
    Parse(String),
}

impl From<ValidationError> for ParseError {
    fn from(err: ValidationError) -> Self {
        ParseError::Validation(err)
    }
}

impl From<de::value::Error> for ParseError {
    fn from(err: de::value::Error) -> Self {
        ParseError::Parse(err.to_string())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Validation(err) => write!(f, "Validation error: {}", err),
            ParseError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Validation(err) => Some(err),
            ParseError::Parse(_) => None,
        }
    }
}