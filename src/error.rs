use parser::Token;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum JsonError {
    UnexpectedToken(String),
    UnexpectedCharacter(u8),
    UnexpectedEndOfJson,
    UnableToReadStringValue,
    ArrayIndexOutOfBounds,
    WrongType(String),
    UndefinedField(String),
}

impl JsonError {
    pub fn unexpected_token(token: Token) -> Self {
        JsonError::UnexpectedToken(format!("{:?}", token))
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
            UnexpectedToken(ref s)  => write!(f, "Unexpected token: {}", s),
            UnexpectedCharacter(c)  => write!(f, "Unexpected character: {}", c),
            UnexpectedEndOfJson     => write!(f, "Unexpected end of JSON"),
            UnableToReadStringValue => write!(f, "Unable to read string value"),
            ArrayIndexOutOfBounds   => write!(f, "Array index out of bounds!"),
            WrongType(ref s)        => write!(f, "Wrong type, expected: {}", s),
            UndefinedField(ref s)   => write!(f, "Undefined field: {}", s)
        }
    }
}

impl Error for JsonError {
    fn description(&self) -> &str {
        use JsonError::*;
        match *self {
            UnexpectedToken(_)      => "Unexpected token",
            UnexpectedCharacter(_)  => "Unexpected character",
            UnexpectedEndOfJson     => "Unexpected end of JSON",
            UnableToReadStringValue => "Failed to read a string value from JSON",
            ArrayIndexOutOfBounds   => "Array index out of bounds!",
            WrongType(_)            => "Wrong type",
            UndefinedField(_)       => "Undefined field",
        }
    }
}
