use parser::Token;
use std::error::Error;
use std::fmt;
use std::char;

#[derive(Debug, PartialEq)]
pub enum JsonError {
    UnexpectedToken {
        token: String,
        line: usize,
        column: usize,
    },
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
    pub fn unexpected_token(token: Token,) -> Self {
        JsonError::UnexpectedToken {
            token: token.to_string(),
            line: 0,
            column: 0,
        }
    }

    pub fn wrong_type(expected: &str) -> Self {
        JsonError::WrongType(expected.into())
    }

    pub fn undefined(field: &str) -> Self {
        JsonError::UndefinedField(field.into())
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use JsonError::*;

        match *self {
            UnexpectedToken {
                ref token,
                ref line,
                ref column,
            } => write!(f, "Unexpected token: {} at ({}:{})", token, line, column),

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
            UnexpectedToken { .. }     => "Unexpected token",
            UnexpectedCharacter { .. } => "Unexpected character",
            UnexpectedEndOfJson        => "Unexpected end of JSON",
            FailedUtf8Parsing          => "Failed to read bytes as UTF-8 from JSON",
            ArrayIndexOutOfBounds      => "Array index out of bounds!",
            WrongType(_)               => "Wrong type",
            UndefinedField(_)          => "Undefined field",
        }
    }
}
