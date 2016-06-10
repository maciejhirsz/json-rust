pub struct JsonError(String);

impl JsonError {
    pub fn unexpected_token(token: super::parser::Token) -> Self {
        JsonError(format!("Unexpected token {:?}", token))
    }
}
