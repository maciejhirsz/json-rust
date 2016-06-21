use std::char;
use std::str;
use std::str::Bytes;
use std::iter::{ Peekable, Iterator };
use std::collections::BTreeMap;
use { JsonValue, JsonError, JsonResult };

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
    source: Peekable<Bytes<'a>>,
    buffer: Vec<u8>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            source: source.bytes().peekable(),
            buffer: Vec::new()
        }
    }

    fn expect(&mut self) -> JsonResult<u8> {
        self.source.next().ok_or(JsonError::UnexpectedEndOfJson)
    }

    fn read_char_as_number(&mut self) -> JsonResult<u32> {
        Ok(match try!(self.expect()) {
            b'0'        => 0,
            b'1'        => 1,
            b'2'        => 2,
            b'3'        => 3,
            b'4'        => 4,
            b'5'        => 5,
            b'6'        => 6,
            b'7'        => 7,
            b'8'        => 8,
            b'9'        => 9,
            b'a' | b'A' => 10,
            b'b' | b'B' => 11,
            b'c' | b'C' => 12,
            b'd' | b'D' => 13,
            b'e' | b'E' => 14,
            b'f' | b'F' => 15,
            ch          => return Err(JsonError::UnexpectedCharacter(ch)),
        })
    }

    fn read_label(&mut self, first: u8) -> &[u8] {
        self.buffer.clear();
        self.buffer.push(first);

        while let Some(&ch) = self.source.peek() {
            match ch {
                b'a' ... b'z' => {
                    self.buffer.push(ch);
                    self.source.next();
                },
                _ => break,
            }
        }

        &self.buffer
    }

    fn read_codepoint(&mut self) -> JsonResult<()> {
        let codepoint = try!(self.read_char_as_number()) << 12
                      | try!(self.read_char_as_number()) << 8
                      | try!(self.read_char_as_number()) << 4
                      | try!(self.read_char_as_number());

        let ch = try!(
            char::from_u32(codepoint).ok_or(JsonError::UnableToReadStringValue)
        );

        let mut buffer = String::new();
        buffer.push(ch);

        self.buffer.extend_from_slice(buffer.as_bytes());
        Ok(())
    }

    fn read_string(&mut self, first: u8) -> JsonResult<String> {
        self.buffer.clear();
        let mut escape = false;

        while let Some(ch) = self.source.next() {
            if ch == first && !escape {
                break;
            }

            if escape {
                let ch = match ch {
                    b'b' => 0x8,
                    b'f' => 0xC,
                    b't' => b'\t',
                    b'r' => b'\r',
                    b'n' => b'\n',
                    b'u' => {
                        try!(self.read_codepoint());
                        escape = false;
                        continue;
                    },
                    _    => ch,
                };

                self.buffer.push(ch);
            } else if ch == b'\\' {
                escape = true;
                continue;
            } else {
                self.buffer.push(ch);
            }

            escape = false;
        }

        str::from_utf8(&self.buffer).ok()
            .and_then(|slice| Some(slice.to_string()))
            .ok_or(JsonError::UnableToReadStringValue)
    }

    fn read_digits_to_buffer(&mut self) {
        while let Some(&ch) = self.source.peek() {
            match ch {
                b'0' ... b'9' => {
                    self.buffer.push(ch);
                    self.source.next();
                },
                _ => break
            }
        }
    }

    fn read_number(&mut self, first: u8) -> JsonResult<f64> {
        self.buffer.clear();
        self.buffer.push(first);
        self.read_digits_to_buffer();

        let mut period = false;

        while let Some(&ch) = self.source.peek() {
            match ch {
                b'.' => {
                    if period {
                        return Err(JsonError::UnexpectedCharacter(ch));
                    }
                    period = true;
                    self.buffer.push(ch);
                    self.source.next();
                    self.read_digits_to_buffer();
                },
                b'e' | b'E' => {
                    self.buffer.push(ch);
                    self.source.next();
                    match self.source.peek() {
                        Some(&b'-') | Some(&b'+') => {
                            self.buffer.push(self.source.next().unwrap());
                        },
                        _ => {}
                    }
                    self.read_digits_to_buffer();
                    break;
                },
                _ => break
            }
        }

        str::from_utf8(&self.buffer).ok()
            .and_then(|slice| slice.parse::<f64>().ok())
            .ok_or(JsonError::UnableToReadStringValue)
    }

    fn next(&mut self) -> JsonResult<Token> {
        while let Some(ch) = self.source.next() {
            return Ok(match ch {
                b',' => Token::Comma,
                b':' => Token::Colon,
                b'[' => Token::BracketOn,
                b']' => Token::BracketOff,
                b'{' => Token::BraceOn,
                b'}' => Token::BraceOff,
                b'"' => Token::String(try!(self.read_string(ch))),
                b'0' ... b'9' | b'-' => Token::Number(try!(self.read_number(ch))),
                b'a' ... b'z' => {
                    match self.read_label(ch) {
                        b"true"  => Token::Boolean(true),
                        b"false" => Token::Boolean(false),
                        b"null"  => Token::Null,
                        label    => {
                            return Err(JsonError::UnexpectedToken(
                                String::from_utf8(label.into())
                                    .unwrap_or("unknown".into())
                            ));
                        }
                    }
                },
                b' ' | b'\r' | b'\n' | b'\t' | 0xA0 => continue,
                _ => return Err(JsonError::UnexpectedCharacter(ch))
            });
        }
        Err(JsonError::UnexpectedEndOfJson)
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
        self.tokenizer.next()
    }

    #[must_use]
    fn ensure_end(&mut self) -> JsonResult<()> {
        match self.tokenizer.next() {
            Ok(token) => Err(JsonError::unexpected_token(token)),
            Err(JsonError::UnexpectedEndOfJson) => Ok(()),
            Err(error)                          => Err(error)
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
