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

macro_rules! expect_char {
    ($tok:ident, $ch:pat) => {
        match $tok.source.next() {
            Some($ch) => {},
            Some(ch)  => return Err(JsonError::unexpected_character(ch)),
            None      => return Err(JsonError::UnexpectedEndOfJson)
        }
    }
}

struct Tokenizer<'a> {
    source: Peekable<Bytes<'a>>,
    buffer: Vec<u8>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            source: source.bytes().peekable(),
            buffer: Vec::with_capacity(500)
        }
    }

    #[inline(always)]
    fn expect(&mut self) -> JsonResult<u8> {
        self.source.next().ok_or(JsonError::UnexpectedEndOfJson)
    }

    fn read_char_as_hexnumber(&mut self) -> JsonResult<u32> {
        let ch = try!(self.expect());
        Ok(match ch {
            b'0' ... b'9' => (ch - b'0') as u32,
            b'a' ... b'f' => (ch + 10 - b'a') as u32,
            b'A' ... b'F' => (ch + 10 - b'A') as u32,
            ch            => return Err(JsonError::unexpected_character(ch)),
        })
    }

    fn read_codepoint(&mut self) -> JsonResult<()> {
        let codepoint = try!(self.read_char_as_hexnumber()) << 12
                      | try!(self.read_char_as_hexnumber()) << 8
                      | try!(self.read_char_as_hexnumber()) << 4
                      | try!(self.read_char_as_hexnumber());

        let ch = try!(
            char::from_u32(codepoint).ok_or(JsonError::FailedUtf8Parsing)
        );

        let mut buffer = String::new();
        buffer.push(ch);

        self.buffer.extend_from_slice(buffer.as_bytes());
        Ok(())
    }

    fn read_escaped_char(&mut self) -> JsonResult<()> {
        let ch = try!(self.expect());
        let ch = match ch {
            b'b' => 0x8,
            b'f' => 0xC,
            b't' => b'\t',
            b'r' => b'\r',
            b'n' => b'\n',
            b'u' => {
                try!(self.read_codepoint());
                return Ok(());
            },
            _   => ch
        };
        self.buffer.push(ch);

        Ok(())
    }

    fn read_string(&mut self) -> JsonResult<String> {
        self.buffer.clear();

        loop {
            let ch = try!(self.expect());
            match ch {
                b'"'  => break,
                b'\\' => try!(self.read_escaped_char()),
                _     => self.buffer.push(ch)
            }
        }

        String::from_utf8(self.buffer.clone())
        .or(Err(JsonError::FailedUtf8Parsing))
    }

    fn read_number(&mut self, first: u8) -> JsonResult<f64> {
        let mut num = if first == b'-' { 0 } else { (first - b'0') as u64 };

        while let Some(&ch) = self.source.peek() {
            match ch {
                b'0' ... b'9' => {
                    num = num * 10 + (ch - b'0') as u64;
                },
                _ => break
            }
            self.source.next();
        }

        match self.source.peek() {
            Some(&b'.') | Some(&b'e') | Some(&b'E') => {},
            _ => return Ok(if first == b'-' {
                (num as f64) * -1.0
            } else {
                num as f64
            })
        }

        let mut num = num as f64;

        if let Some(&b'.') = self.source.peek() {
            self.source.next();

            let mut precision = -1;
            while let Some(&ch) = self.source.peek() {
                match ch {
                    b'0' ... b'9' => {
                        num += ((ch - b'0') as f64) * 10f64.powi(precision);
                        precision -= 1;
                    },
                    _ => break
                }
                self.source.next();
            }
        }

        match self.source.peek() {
            Some(&b'e') | Some(&b'E') => {
                self.source.next();

                let mut e = 0;
                let sign = match self.source.peek() {
                    Some(&b'-') => {
                        self.source.next();
                        -1
                    },
                    Some(&b'+') => {
                        self.source.next();
                        1
                    },
                    _ => 1
                };

                while let Some(&ch) = self.source.peek() {
                    match ch {
                        b'0' ... b'9' => e = e * 10 + (ch - b'0') as i32,
                        _ => break
                    }
                    self.source.next();
                }

                num *= 10f64.powi(e * sign);
            },
            _ => {}
        }

        Ok(if first == b'-' { num * -1.0 } else { num })
    }

    fn next(&mut self) -> JsonResult<Token> {
        loop {
            let ch = try!(self.expect());
            return Ok(match ch {
                b',' => Token::Comma,
                b':' => Token::Colon,
                b'[' => Token::BracketOn,
                b']' => Token::BracketOff,
                b'{' => Token::BraceOn,
                b'}' => Token::BraceOff,
                b'"' => Token::String(try!(self.read_string())),
                b'0' ... b'9' | b'-' => Token::Number(try!(self.read_number(ch))),
                b't' => {
                    expect_char!(self, b'r');
                    expect_char!(self, b'u');
                    expect_char!(self, b'e');
                    Token::Boolean(true)
                },
                b'f' => {
                    expect_char!(self, b'a');
                    expect_char!(self, b'l');
                    expect_char!(self, b's');
                    expect_char!(self, b'e');
                    Token::Boolean(false)
                },
                b'n' => {
                    expect_char!(self, b'u');
                    expect_char!(self, b'l');
                    expect_char!(self, b'l');
                    Token::Null
                },
                // whitespace
                9 ... 13 | 32 | 133 | 160 => continue,
                _ => return Err(JsonError::unexpected_character(ch))
            });
        }
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
            Token::BracketOff => return Ok(JsonValue::Array(array)),
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

        Ok(JsonValue::Array(array))
    }

    fn object(&mut self) -> JsonResult<JsonValue> {
        let mut object = BTreeMap::new();

        match try!(self.consume()) {
            Token::BraceOff    => return Ok(JsonValue::Object(object)),
            Token::String(key) => {
                expect!(self, Token::Colon);
                object.insert(key, try!(self.value()));
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
                    object.insert(key, try!(self.value()));
                    continue
                },
                Token::BraceOff => break,
                token => return Err(JsonError::unexpected_token(token))
            }
        }

        Ok(JsonValue::Object(object))
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
