#[derive(Debug)]
pub struct JsonError(String);

impl JsonError {
    pub fn unexpected_token(token: super::parser::Token) -> Self {
        JsonError(format!("Unexpected token {:?}", token))
    }

    pub fn wrong_type(expected: &str) -> Self {
        JsonError(format!("Wrong type, expected {}", expected))
    }

    pub fn undefined(field: &str) -> Self {
        JsonError(format!("Undefined field {}", field))
    }

    pub fn custom(msg: &str) -> Self {
        JsonError(msg.into())
    }
}
