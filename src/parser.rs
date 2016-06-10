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
        match $parser.consume() {
            $p    => $value,
            token => panic!("Unexpected token {:?}", token)
        }
    );
    ($parser:ident, $token:pat) => ({
        match $parser.consume() {
            $token => {}
            token  => panic!("Unexpected token {:?}", token)
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

    fn consume(&mut self) -> Token {
        self.tokenizer.next().expect("Unexpected end of JSON")
    }


    fn array(&mut self) -> JsonValue {
        let mut array = Vec::new();

        match self.consume() {
            Token::BracketOff => return array.into(),
            token             => array.push(self.value_from(token)),
        }

        loop {
            match self.consume() {
                Token::Comma => {
                    array.push(self.value());
                    continue
                },
                Token::BracketOff => break,
                token => panic!("Unexpected token {:?}", token)
            }
        }

        array.into()
    }

    fn object(&mut self) -> JsonValue {
        let mut object = BTreeMap::new();

        match self.consume() {
            Token::BraceOff    => return object.into(),
            Token::String(key) => {
                expect!(self, Token::Colon);
                let value = self.value();
                object.insert(key, value);
            },
            token => panic!("Unexpected token {:?}", token),
        }

        loop {
            match self.consume() {
                Token::Comma => {
                    let key = expect!(self,
                        Token::String(key) => key
                    );
                    expect!(self, Token::Colon);
                    let value = self.value();
                    object.insert(key, value);
                    continue
                },
                Token::BraceOff => break,
                token => panic!("Unexpected token {:?}", token)
            }
        }

        object.into()
    }

    fn value_from(&mut self, token: Token) -> JsonValue {
        match token {
            Token::String(value)  => JsonValue::String(value),
            Token::Number(value)  => JsonValue::Number(value),
            Token::Boolean(value) => JsonValue::Boolean(value),
            Token::Null           => JsonValue::Null,
            Token::BracketOn      => self.array(),
            Token::BraceOn        => self.object(),
            token => panic!("Unexpected token {:?}", token)
        }
    }

    fn value(&mut self) -> JsonValue {
        let token = self.consume();
        self.value_from(token)
    }
}

pub fn parse(source: &str) -> JsonValue {
    let mut parser = Parser::new(source);

    parser.value()
}
