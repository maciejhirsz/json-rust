use parser::Token;
use std::error::Error;
use std::fmt;
use std::char;

#[derive(Debug)]
pub enum JsonError {
    UnexpectedToken(String),
    UnexpectedCharacter(char),
    UnexpectedEndOfJson,
    FailedUtf8Parsing,
    ArrayIndexOutOfBounds,
    WrongType(String),
    UndefinedField(String),
}

impl JsonError {
    pub fn unexpected_token(token: Token) -> Self {
        JsonError::UnexpectedToken(format!("{:?}", token))
    }

    pub fn unexpected_character(byte: u8) -> Self {
        JsonError::UnexpectedCharacter(
            char::from_u32(byte as u32).unwrap_or('?')
        )
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
            UnexpectedCharacter(ch) => write!(f, "Unexpected character: {}", ch),
            UnexpectedEndOfJson     => write!(f, "Unexpected end of JSON"),
            FailedUtf8Parsing       => write!(f, "Failed to parse UTF-8 bytes"),
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
            UnexpectedToken(_)     => "Unexpected token",
            UnexpectedCharacter(_) => "Unexpected character",
            UnexpectedEndOfJson    => "Unexpected end of JSON",
            FailedUtf8Parsing      => "Failed to read bytes as UTF-8 from JSON",
            ArrayIndexOutOfBounds  => "Array index out of bounds!",
            WrongType(_)           => "Wrong type",
            UndefinedField(_)      => "Undefined field",
        }
    }
}
