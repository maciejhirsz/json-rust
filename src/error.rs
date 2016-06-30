use std::error::Error;
use std::fmt;
use std::char;

#[derive(Debug, PartialEq)]
pub enum JsonError {
    UnexpectedCharacter {
        ch: char,
        line: usize,
        column: usize,
    },
    UnexpectedEndOfJson,
    FailedUtf8Parsing,
    ArrayIndexOutOfBounds,
    WrongType(String),
    UndefinedField(String),
}

impl JsonError {
    pub fn wrong_type(expected: &str) -> Self {
        JsonError::WrongType(expected.into())
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use JsonError::*;

        match *self {
            UnexpectedCharacter {
                ref ch,
                ref line,
                ref column,
            } => write!(f, "Unexpected character: {} at ({}:{})", ch, line, column),

            UnexpectedEndOfJson   => write!(f, "Unexpected end of JSON"),
            FailedUtf8Parsing     => write!(f, "Failed to parse UTF-8 bytes"),
            ArrayIndexOutOfBounds => write!(f, "Array index out of bounds!"),
            WrongType(ref s)      => write!(f, "Wrong type, expected: {}", s),
            UndefinedField(ref s) => write!(f, "Undefined field: {}", s)
        }
    }
}

impl Error for JsonError {
    fn description(&self) -> &str {
        use JsonError::*;
        match *self {
            UnexpectedCharacter { .. } => "Unexpected character",
            UnexpectedEndOfJson        => "Unexpected end of JSON",
            FailedUtf8Parsing          => "Failed to read bytes as UTF-8 from JSON",
            ArrayIndexOutOfBounds      => "Array index out of bounds!",
            WrongType(_)               => "Wrong type",
            UndefinedField(_)          => "Undefined field",
        }
    }
}
