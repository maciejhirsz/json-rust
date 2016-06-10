use std::str::Chars;
use std::iter::{ Peekable, Iterator };
use std::collections::BTreeMap;
use { JsonValue, JsonError, JsonResult };

macro_rules! expect {
    ($tokenizer:ident, $p:pat) => (
        match $tokenizer.next() {
            Some($p) => {},
            token    => panic!("WAT"),
        }
    )
}

#[derive(Debug)]
pub enum Token {
    Comma,
    Colon,
    BracketOn,
    BracketOff,
    BraceOn,
    BraceOff,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

struct Tokenizer<'a> {
    source: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            source: source.chars().peekable(),
        }
    }

    fn read_label(&mut self, first: char) -> String {
        let mut label = first.to_string();

        while let Some(&ch) = self.source.peek() {
            match ch {
                'a'...'z' => {
                    label.push(ch);
                    self.source.next();
                },
                _ => break,
            }
        }

        return label;
    }

    fn read_string(&mut self, first: char) -> String {
        let mut value = String::new();
        let mut escape = false;

        while let Some(ch) = self.source.next() {
            if ch == first && escape == false {
                return value;
            }
            match ch {
                '\\' => {
                    if escape {
                        escape = false;
                        value.push(ch);
                    } else {
                        escape = true;
                    }
                },
                _ => {
                    value.push(ch);
                    escape = false;
                },
            }
        }

        return value;
    }

    fn read_number(&mut self, first: char) -> f64 {
        let mut value = first.to_string();
        let mut period = false;

        while let Some(&ch) = self.source.peek() {
            match ch {
                '0'...'9' => {
                    value.push(ch);
                    self.source.next();
                },
                '.' => {
                    if !period {
                        period = true;
                        value.push(ch);
                        self.source.next();
                    } else {
                        return value.parse::<f64>().unwrap();
                    }
                },
                _ => return value.parse::<f64>().unwrap(),
            }
        }

        value.parse::<f64>().unwrap()
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        'lex: while let Some(ch) = self.source.next() {
            return Some(match ch {
                ',' => Token::Comma,
                ':' => Token::Colon,
                '[' => Token::BracketOn,
                ']' => Token::BracketOff,
                '{' => Token::BraceOn,
                '}' => Token::BraceOff,
                '"' => {
                    Token::String(self.read_string(ch))
                },
                '0'...'9' => Token::Number(self.read_number(ch)),
                'a'...'z' => {
                    let label = self.read_label(ch);
                    match label.as_ref() {
                        "true"  => Token::Boolean(true),
                        "false" => Token::Boolean(false),
                        "null"  => Token::Null,
                        _       => panic!("Invalid label `{:?}`", label)
                    }
                },
                _  => {
                    if ch.is_whitespace() {
                        continue 'lex;
                    } else {
                        panic!("Invalid character `{:?}`", ch);
                    }
                }
            });
        }
        return None;
    }
}

macro_rules! expect {
    ($parser:ident, $p:pat => $value:ident) => (
        match try!($parser.consume()) {
            $p    => $value,
            token => return Err(JsonError::unexpected_token(token))
        }
    );
    ($parser:ident, $token:pat) => ({
        match try!($parser.consume()) {
            $token => {}
            token  => return Err(JsonError::unexpected_token(token))
        }
    })
}

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(source),
        }
    }

    fn consume(&mut self) -> JsonResult<Token> {
        match self.tokenizer.next() {
            Some(token) => Ok(token),
            None        => Err(JsonError::custom("Unexpected end of JSON"))
        }
    }

    #[must_use]
    fn ensure_end(&mut self) -> JsonResult<()> {
        match self.tokenizer.next() {
            Some(token) => Err(JsonError::unexpected_token(token)),
            None        => Ok(())
        }
    }

    fn array(&mut self) -> JsonResult<JsonValue> {
        let mut array = Vec::new();

        match try!(self.consume()) {
            Token::BracketOff => return Ok(array.into()),
            token             => array.push(try!(self.value_from(token))),
        }

        loop {
            match try!(self.consume()) {
                Token::Comma => {
                    array.push(try!(self.value()));
                    continue
                },
                Token::BracketOff => break,
                token => return Err(JsonError::unexpected_token(token))
            }
        }

        Ok(array.into())
    }

    fn object(&mut self) -> JsonResult<JsonValue> {
        let mut object = BTreeMap::new();

        match try!(self.consume()) {
            Token::BraceOff    => return Ok(object.into()),
            Token::String(key) => {
                expect!(self, Token::Colon);
                let value = try!(self.value());
                object.insert(key, value);
            },
            token => return Err(JsonError::unexpected_token(token))
        }

        loop {
            match try!(self.consume()) {
                Token::Comma => {
                    let key = expect!(self,
                        Token::String(key) => key
                    );
                    expect!(self, Token::Colon);
                    let value = try!(self.value());
                    object.insert(key, value);
                    continue
                },
                Token::BraceOff => break,
                token => return Err(JsonError::unexpected_token(token))
            }
        }

        Ok(object.into())
    }

    fn value_from(&mut self, token: Token) -> JsonResult<JsonValue> {
        Ok(match token {
            Token::String(value)  => JsonValue::String(value),
            Token::Number(value)  => JsonValue::Number(value),
            Token::Boolean(value) => JsonValue::Boolean(value),
            Token::Null           => JsonValue::Null,
            Token::BracketOn      => return self.array(),
            Token::BraceOn        => return self.object(),
            token => return Err(JsonError::unexpected_token(token))
        })
    }

    fn value(&mut self) -> JsonResult<JsonValue> {
        let token = try!(self.consume());
        self.value_from(token)
    }
}

pub fn parse(source: &str) -> JsonResult<JsonValue> {
    let mut parser = Parser::new(source);

    let value = try!(parser.value());

    try!(parser.ensure_end());

    Ok(value)
}
